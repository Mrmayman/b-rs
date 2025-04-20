#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use std::{
    ffi::{CString, c_char, c_double, c_int, c_long, c_uchar},
    ptr,
};

use crate::stb::Clex;
unsafe extern "C" {
    fn strtod(_: *const c_char, _: *mut *const c_char) -> c_double;
    fn strtol(_: *const c_char, _: *mut *const c_char, _: c_int) -> c_long;
}

#[derive(Clone)]
#[repr(C)]
pub struct StbLexer {
    pub input: CString,
    pub eof: *const c_char,

    pub parse_point: *const c_char,
    pub string_storage: *mut c_char,
    pub string_storage_len: c_int,
    pub where_firstchar: *const c_char,
    pub where_lastchar: *const c_char,
    pub token: c_long,
    pub real_number: c_double,
    pub int_number: c_long,
    pub string: *mut c_char,
    pub string_len: c_int,
}

impl StbLexer {
    pub fn new(input: CString, string_store: *mut c_char, store_length: c_int) -> Self {
        let parse_point = input.as_ptr();
        let eof = unsafe { input.as_ptr().add(input.count_bytes()) };
        Self {
            input,
            eof,
            parse_point,
            string_storage: string_store,
            string_storage_len: store_length,
            where_firstchar: ptr::null_mut(),
            where_lastchar: ptr::null_mut(),
            token: 0,
            real_number: 0.0,
            int_number: 0,
            string: ptr::null_mut(),
            string_len: 0,
        }
    }

    pub unsafe fn get_location(&self) -> StbLexLocation {
        unsafe { self.get_location_at(self.where_firstchar) }
    }

    pub unsafe fn get_location_at(&self, where_location: *const i8) -> StbLexLocation {
        let mut line_number = 1;
        let mut line_offset = 0;

        let mut p: *const c_char = self.input.as_ptr();
        unsafe {
            while !p.is_null() && p < where_location {
                if *p as u8 == b'\n' || *p as u8 == b'\r' {
                    if *p as c_int + *p.offset(1) as c_int == '\r' as i32 + '\n' as i32 {
                        p = p.offset(1);
                    }

                    line_number += 1;
                    line_offset = 0;
                } else {
                    line_offset += 1;
                }
                p = p.offset(1);
            }
        }

        StbLexLocation {
            line_number,
            line_offset,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StbLexLocation {
    pub line_number: c_int,
    pub line_offset: c_int,
}

unsafe fn stb__clex_token(
    mut lexer: *mut StbLexer,
    mut token: c_int,
    mut start: *const c_char,
    mut end: *const c_char,
) -> c_int {
    unsafe {
        (*lexer).token = token as c_long;
        (*lexer).where_firstchar = start;
        (*lexer).where_lastchar = end;
        (*lexer).parse_point = end.offset(1);
    }
    1
}
unsafe fn stb__clex_eof(mut lexer: *mut StbLexer) -> c_int {
    unsafe {
        (*lexer).token = Clex::Eof as c_int as c_long;
    }
    0
}

unsafe fn stb__clex_parse_suffixes(
    mut lexer: *mut StbLexer,
    mut tokenid: c_long,
    mut start: *const c_char,
    mut cur: *const c_char,
    mut suffixes: *const c_char,
) -> c_int {
    suffixes = suffixes;
    return unsafe { stb__clex_token(lexer, tokenid as c_int, start, cur.offset(-(1))) };
}
unsafe fn stb__clex_parse_char(mut p: *const c_char, mut q: *mut *const c_char) -> Option<char> {
    unsafe {
        if *p as c_int == '\\' as i32 {
            *q = p.offset(2);
            match *p.offset(1) as u8 {
                b'\\' => return Some('\\'),
                b'\'' => return Some('\''),
                b'"' => return Some('"'),
                b't' => return Some('\t'),
                b'f' => return Some('\u{c}'),
                b'n' => return Some('\n'),
                b'r' => return Some('\r'),
                b'0' => return Some('\0'),  // TODO: octal constants
                b'x' | b'X' => return None, // TODO: hex constants
                b'u' => return None,        // TODO: unicode constants
                _ => {}
            }
        }
        *q = p.offset(1);
        return Some(*p as u8 as char);
    }
}
unsafe fn stb__clex_parse_string(
    mut lexer: *mut StbLexer,
    mut p: *const c_char,
    mut type_0: c_int,
) -> c_int {
    unsafe {
        let mut start: *const c_char = p;
        let fresh0 = p;
        p = p.offset(1);
        let mut delim: c_char = *fresh0;
        let mut out: *mut c_char = (*lexer).string_storage;
        let mut outend: *mut c_char =
            ((*lexer).string_storage).offset((*lexer).string_storage_len as isize);
        while *p as c_int != delim as c_int {
            let mut n: c_int = 0;
            if *p as c_int == '\\' as i32 {
                let mut q: *const c_char = 0 as *mut c_char;
                if stb__clex_parse_char(p, &mut q).is_none() {
                    return stb__clex_token(lexer, Clex::ParseError as c_int, start, q);
                };
                p = q;
            } else {
                let fresh1 = p;
                p = p.offset(1);
                n = *fresh1 as c_uchar as c_int;
            }
            if out.offset(1) > outend {
                return stb__clex_token(lexer, Clex::ParseError as c_int, start, p);
            }
            let fresh2 = out;
            out = out.offset(1);
            *fresh2 = n as c_char;
        }
        *out = 0 as c_char;
        (*lexer).string = (*lexer).string_storage;
        (*lexer).string_len = out.offset_from((*lexer).string_storage) as c_long as c_int;
        return stb__clex_token(lexer, type_0, start, p);
    }
}
#[unsafe(no_mangle)]
pub unsafe fn stb_c_lexer_get_token(mut lexer: *mut StbLexer) -> c_int {
    const BLOCK_SINGLE_CHAR: u64 = 17616027239959249775;

    let mut current_block: u64;
    unsafe {
        let mut p: *const c_char = (*lexer).parse_point;
        loop {
            while p != (*lexer).eof && (*p as u8 as char).is_whitespace() {
                p = p.offset(1);
            }
            if p != (*lexer).eof
                && *p.offset(0) as c_int == '/' as i32
                && *p.offset(1) as c_int == '/' as i32
            {
                while p != (*lexer).eof && *p as c_int != '\r' as i32 && *p as c_int != '\n' as i32
                {
                    p = p.offset(1);
                }
            } else if p != (*lexer).eof
                && *p.offset(0) as c_int == '/' as i32
                && *p.offset(1) as c_int == '*' as i32
            {
                let mut start: *const c_char = p;
                p = p.offset(2);
                while p != (*lexer).eof
                    && (*p.offset(0) as c_int != '*' as i32 || *p.offset(1) as c_int != '/' as i32)
                {
                    p = p.offset(1);
                }
                if p == (*lexer).eof {
                    return stb__clex_token(
                        lexer,
                        Clex::ParseError as c_int,
                        start,
                        p.offset(-(1)),
                    );
                }
                p = p.offset(2);
            } else {
                if !(p != (*lexer).eof && *p.offset(0) as c_int == '#' as i32) {
                    break;
                }
                while p != (*lexer).eof && *p as c_int != '\r' as i32 && *p as c_int != '\n' as i32
                {
                    p = p.offset(1);
                }
            }
        }

        if p == (*lexer).eof {
            return stb__clex_eof(lexer);
        }

        match *p as u8 {
            b'+' => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as u8 == b'+' {
                        return stb__clex_token(lexer, Clex::Plusplus as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Pluseq as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            b'-' => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as u8 == b'-' {
                        return stb__clex_token(lexer, Clex::Minusminus as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Minuseq as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'>' {
                        return stb__clex_token(lexer, Clex::Arrow as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            b'&' => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as c_int == '&' as i32 {
                        return stb__clex_token(lexer, Clex::Andand as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Andeq as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            b'|' => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as c_int == '|' as i32 {
                        return stb__clex_token(lexer, Clex::Oror as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Oreq as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            61 => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Eq as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            33 => {
                if p.offset(1) != (*lexer).eof && *p.offset(1) as c_int == '=' as i32 {
                    return stb__clex_token(lexer, Clex::Noteq as c_int, p, p.offset(1));
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            94 => {
                if p.offset(1) != (*lexer).eof && *p.offset(1) as c_int == '=' as i32 {
                    return stb__clex_token(lexer, Clex::Xoreq as c_int, p, p.offset(1));
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            37 => {
                if p.offset(1) != (*lexer).eof && *p.offset(1) as c_int == '=' as i32 {
                    return stb__clex_token(lexer, Clex::Modeq as c_int, p, p.offset(1));
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            42 => {
                if p.offset(1) != (*lexer).eof && *p.offset(1) as c_int == '=' as i32 {
                    return stb__clex_token(lexer, Clex::Muleq as c_int, p, p.offset(1));
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            47 => {
                if p.offset(1) != (*lexer).eof && *p.offset(1) as c_int == '=' as i32 {
                    return stb__clex_token(lexer, Clex::Diveq as c_int, p, p.offset(1));
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            60 => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Lesseq as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'<' {
                        if p.offset(2) != (*lexer).eof && *p.offset(2) as c_int == '=' as i32 {
                            return stb__clex_token(lexer, Clex::Shleq as c_int, p, p.offset(2));
                        }
                        return stb__clex_token(lexer, Clex::Shl as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            62 => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as u8 == b'=' {
                        return stb__clex_token(lexer, Clex::Greatereq as c_int, p, p.offset(1));
                    }
                    if *p.offset(1) as u8 == b'>' {
                        if p.offset(2) != (*lexer).eof && *p.offset(2) as c_int == '=' as i32 {
                            return stb__clex_token(lexer, Clex::Shreq as c_int, p, p.offset(2));
                        }
                        return stb__clex_token(lexer, Clex::Shr as c_int, p, p.offset(1));
                    }
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
            34 => return stb__clex_parse_string(lexer, p, Clex::Dqstring as c_int),
            39 => {
                let mut start_0: *const c_char = p;
                (*lexer).int_number = stb__clex_parse_char(p.offset(1), &mut p)
                    .map(|n| n as i64)
                    .unwrap_or(-1);
                if (*lexer).int_number < 0 as c_long {
                    return stb__clex_token(lexer, Clex::ParseError as c_int, start_0, start_0);
                }
                if p == (*lexer).eof || *p as c_int != '\'' as i32 {
                    return stb__clex_token(lexer, Clex::ParseError as c_int, start_0, p);
                }
                return stb__clex_token(lexer, Clex::Charlit as c_int, start_0, p.offset(1));
            }
            48 => {
                if p.offset(1) != (*lexer).eof {
                    if *p.offset(1) as c_int == 'x' as i32 || *p.offset(1) as c_int == 'X' as i32 {
                        let mut q: *const c_char = 0 as *mut c_char;
                        (*lexer).int_number = strtol(p, &mut q as *mut *const c_char, 16 as c_int);
                        if q == p.offset(2) {
                            return stb__clex_token(
                                lexer,
                                Clex::ParseError as c_int,
                                p.offset(-(2)),
                                p.offset(-(1)),
                            );
                        }
                        return stb__clex_parse_suffixes(
                            lexer,
                            Clex::Intlit as c_int as c_long,
                            p,
                            q,
                            b"\0" as *const u8 as *const c_char,
                        );
                    }
                }
                current_block = 8569828448656383210;
            }
            b'1'..=b'9' => {
                current_block = 8569828448656383210;
            }
            _ => {
                if *p as c_int >= 'a' as i32 && *p as c_int <= 'z' as i32
                    || *p as c_int >= 'A' as i32 && *p as c_int <= 'Z' as i32
                    || *p as c_int == '_' as i32
                    || *p as c_uchar as c_int >= 128 as c_int
                    || *p as c_int == '$' as i32
                {
                    let mut n: c_int = 0;
                    (*lexer).string = (*lexer).string_storage;
                    loop {
                        if n + 1 as c_int >= (*lexer).string_storage_len {
                            return stb__clex_token(
                                lexer,
                                Clex::ParseError as c_int,
                                p,
                                p.offset(n as isize),
                            );
                        }
                        *((*lexer).string).offset(n as isize) = *p.offset(n as isize);
                        n += 1;
                        if !(*p.offset(n as isize) as c_int >= 'a' as i32
                            && *p.offset(n as isize) as c_int <= 'z' as i32
                            || *p.offset(n as isize) as c_int >= 'A' as i32
                                && *p.offset(n as isize) as c_int <= 'Z' as i32
                            || *p.offset(n as isize) as c_int >= '0' as i32
                                && *p.offset(n as isize) as c_int <= '9' as i32
                            || *p.offset(n as isize) as c_int == '_' as i32
                            || *p.offset(n as isize) as c_uchar as c_int >= 128 as c_int
                            || *p.offset(n as isize) as c_int == '$' as i32)
                        {
                            break;
                        }
                    }
                    *((*lexer).string).offset(n as isize) = 0 as c_char;
                    (*lexer).string_len = n;
                    return stb__clex_token(
                        lexer,
                        Clex::Id as c_int,
                        p,
                        p.offset(n as isize).offset(-(1)),
                    );
                }
                current_block = BLOCK_SINGLE_CHAR;
            }
        }
        match current_block {
            BLOCK_SINGLE_CHAR => return stb__clex_token(lexer, *p as c_int, p, p),
            _ => {
                let mut q_0: *const c_char = p;
                while q_0 != (*lexer).eof && (*q_0 as u8 >= b'0' && *q_0 as u8 <= b'9') {
                    q_0 = q_0.offset(1);
                }
                if q_0 != (*lexer).eof {
                    if *q_0 as u8 == b'.' || *q_0 as u8 == b'e' || *q_0 as u8 == b'E' {
                        (*lexer).real_number = strtod(p, &mut q_0 as *mut *const c_char);
                        return stb__clex_parse_suffixes(
                            lexer,
                            Clex::Floatlit as c_int as c_long,
                            p,
                            q_0,
                            b"\0" as *const u8 as *const c_char,
                        );
                    }
                }
                if *p.offset(0) as c_int == '0' as i32 {
                    let mut q_1: *const c_char = p;
                    (*lexer).int_number = strtol(p, &mut q_1 as *mut *const c_char, 8 as c_int);
                    return stb__clex_parse_suffixes(
                        lexer,
                        Clex::Intlit as c_int as c_long,
                        p,
                        q_1,
                        b"\0" as *const u8 as *const c_char,
                    );
                }
                let mut q_2: *const c_char = p;
                (*lexer).int_number = strtol(p, &mut q_2 as *mut *const c_char, 10);
                return stb__clex_parse_suffixes(
                    lexer,
                    Clex::Intlit as c_int as c_long,
                    p,
                    q_2,
                    b"\0" as *const u8 as *const c_char,
                );
            }
        };
    }
}
