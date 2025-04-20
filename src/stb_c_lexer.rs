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
    ffi::{c_char, c_double, c_int, c_long, c_uchar, c_uint},
    ptr,
};
unsafe extern "C" {
    fn strtod(_: *const c_char, _: *mut *const c_char) -> c_double;
    fn strtol(_: *const c_char, _: *mut *const c_char, _: c_int) -> c_long;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StbLexer {
    pub input_stream: *const c_char,
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
    pub fn new(
        input_stream: *const c_char,
        input_stream_end: *const c_char,
        string_store: *mut c_char,
        store_length: c_int,
    ) -> Self {
        Self {
            input_stream,
            eof: input_stream_end,
            parse_point: input_stream,
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
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stb_lex_location {
    pub line_number: c_int,
    pub line_offset: c_int,
}
pub const CLEX_intlit: CLEX = 258;
pub const CLEX_floatlit: CLEX = 259;
pub const CLEX_parse_error: CLEX = 257;
pub const CLEX_charlit: CLEX = 263;
pub const CLEX_dqstring: CLEX = 261;
pub const CLEX_shr: CLEX = 271;
pub const CLEX_shreq: CLEX = 285;
pub const CLEX_greatereq: CLEX = 267;
pub const CLEX_shl: CLEX = 270;
pub const CLEX_shleq: CLEX = 284;
pub const CLEX_lesseq: CLEX = 266;
pub const CLEX_diveq: CLEX = 277;
pub const CLEX_muleq: CLEX = 276;
pub const CLEX_modeq: CLEX = 278;
pub const CLEX_xoreq: CLEX = 281;
pub const CLEX_noteq: CLEX = 265;
pub const CLEX_eq: CLEX = 264;
pub const CLEX_oreq: CLEX = 280;
pub const CLEX_oror: CLEX = 269;
pub const CLEX_andeq: CLEX = 279;
pub const CLEX_andand: CLEX = 268;
pub const CLEX_arrow: CLEX = 282;
pub const CLEX_minuseq: CLEX = 275;
pub const CLEX_minusminus: CLEX = 273;
pub const CLEX_pluseq: CLEX = 274;
pub const CLEX_plusplus: CLEX = 272;
pub const CLEX_id: CLEX = 260;
pub const CLEX_eof: CLEX = 256;
pub type CLEX = c_uint;
pub const CLEX_first_unused_token: CLEX = 286;
pub const CLEX_eqarrow: CLEX = 283;
pub const CLEX_sqstring: CLEX = 262;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stb_c_lexer_get_location(
    mut lexer: *const StbLexer,
    mut where_0: *const c_char,
    mut loc: *mut stb_lex_location,
) {
    unsafe {
        let mut p: *const c_char = (*lexer).input_stream;
        let mut line_number: c_int = 1 as c_int;
        let mut char_offset: c_int = 0 as c_int;
        while *p as c_int != 0 && p < where_0 as *mut c_char {
            if *p as c_int == '\n' as i32 || *p as c_int == '\r' as i32 {
                p = p.offset(
                    (if *p.offset(0 as c_int as isize) as c_int
                        + *p.offset(1 as c_int as isize) as c_int
                        == '\r' as i32 + '\n' as i32
                    {
                        2 as c_int
                    } else {
                        1 as c_int
                    }) as isize,
                );
                line_number += 1 as c_int;
                char_offset = 0 as c_int;
            } else {
                p = p.offset(1);
                char_offset += 1;
            }
        }
        (*loc).line_number = line_number;
        (*loc).line_offset = char_offset;
    }
}
unsafe extern "C" fn stb__clex_token(
    mut lexer: *mut StbLexer,
    mut token: c_int,
    mut start: *const c_char,
    mut end: *const c_char,
) -> c_int {
    unsafe {
        (*lexer).token = token as c_long;
        (*lexer).where_firstchar = start;
        (*lexer).where_lastchar = end;
        (*lexer).parse_point = end.offset(1 as c_int as isize);
    }
    1
}
unsafe extern "C" fn stb__clex_eof(mut lexer: *mut StbLexer) -> c_int {
    unsafe {
        (*lexer).token = CLEX_eof as c_int as c_long;
    }
    0
}
unsafe extern "C" fn stb__clex_iswhite(mut x: c_int) -> c_int {
    return (x == ' ' as i32
        || x == '\t' as i32
        || x == '\r' as i32
        || x == '\n' as i32
        || x == '\u{c}' as i32) as c_int;
}
unsafe extern "C" fn stb__clex_parse_suffixes(
    mut lexer: *mut StbLexer,
    mut tokenid: c_long,
    mut start: *const c_char,
    mut cur: *const c_char,
    mut suffixes: *const c_char,
) -> c_int {
    suffixes = suffixes;
    return unsafe {
        stb__clex_token(
            lexer,
            tokenid as c_int,
            start,
            cur.offset(-(1 as c_int as isize)),
        )
    };
}
unsafe extern "C" fn stb__clex_parse_char(
    mut p: *const c_char,
    mut q: *mut *const c_char,
) -> c_int {
    unsafe {
        if *p as c_int == '\\' as i32 {
            *q = p.offset(2 as c_int as isize);
            match *p.offset(1 as c_int as isize) as c_int {
                92 => return '\\' as i32,
                39 => return '\'' as i32,
                34 => return '"' as i32,
                116 => return '\t' as i32,
                102 => return '\u{c}' as i32,
                110 => return '\n' as i32,
                114 => return '\r' as i32,
                48 => return '\0' as i32,
                120 | 88 => return -(1 as c_int),
                117 => return -(1 as c_int),
                _ => {}
            }
        }
        *q = p.offset(1 as c_int as isize);
        return *p as c_uchar as c_int;
    }
}
unsafe extern "C" fn stb__clex_parse_string(
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
                n = stb__clex_parse_char(p, &mut q);
                if n < 0 as c_int {
                    return stb__clex_token(lexer, CLEX_parse_error as c_int, start, q);
                }
                p = q;
            } else {
                let fresh1 = p;
                p = p.offset(1);
                n = *fresh1 as c_uchar as c_int;
            }
            if out.offset(1 as c_int as isize) > outend {
                return stb__clex_token(lexer, CLEX_parse_error as c_int, start, p);
            }
            let fresh2 = out;
            out = out.offset(1);
            *fresh2 = n as c_char;
        }
        *out = 0 as c_int as c_char;
        (*lexer).string = (*lexer).string_storage;
        (*lexer).string_len = out.offset_from((*lexer).string_storage) as c_long as c_int;
        return stb__clex_token(lexer, type_0, start, p);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stb_c_lexer_get_token(mut lexer: *mut StbLexer) -> c_int {
    let mut current_block: u64;
    unsafe {
        let mut p: *const c_char = (*lexer).parse_point;
        loop {
            while p != (*lexer).eof && stb__clex_iswhite(*p as c_int) != 0 {
                p = p.offset(1);
            }
            if p != (*lexer).eof
                && *p.offset(0 as c_int as isize) as c_int == '/' as i32
                && *p.offset(1 as c_int as isize) as c_int == '/' as i32
            {
                while p != (*lexer).eof && *p as c_int != '\r' as i32 && *p as c_int != '\n' as i32
                {
                    p = p.offset(1);
                }
            } else if p != (*lexer).eof
                && *p.offset(0 as c_int as isize) as c_int == '/' as i32
                && *p.offset(1 as c_int as isize) as c_int == '*' as i32
            {
                let mut start: *const c_char = p;
                p = p.offset(2 as c_int as isize);
                while p != (*lexer).eof
                    && (*p.offset(0 as c_int as isize) as c_int != '*' as i32
                        || *p.offset(1 as c_int as isize) as c_int != '/' as i32)
                {
                    p = p.offset(1);
                }
                if p == (*lexer).eof {
                    return stb__clex_token(
                        lexer,
                        CLEX_parse_error as c_int,
                        start,
                        p.offset(-(1 as c_int as isize)),
                    );
                }
                p = p.offset(2 as c_int as isize);
            } else {
                if !(p != (*lexer).eof && *p.offset(0 as c_int as isize) as c_int == '#' as i32) {
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
        match *p as c_int {
            43 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '+' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_plusplus as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_pluseq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            45 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '-' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_minusminus as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_minuseq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '>' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_arrow as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            38 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '&' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_andand as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_andeq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            124 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '|' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_oror as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_oreq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            61 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_eq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            33 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof
                    && *p.offset(1 as c_int as isize) as c_int == '=' as i32
                {
                    return stb__clex_token(
                        lexer,
                        CLEX_noteq as c_int,
                        p,
                        p.offset(1 as c_int as isize),
                    );
                }
                current_block = 17616027239959249775;
            }
            94 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof
                    && *p.offset(1 as c_int as isize) as c_int == '=' as i32
                {
                    return stb__clex_token(
                        lexer,
                        CLEX_xoreq as c_int,
                        p,
                        p.offset(1 as c_int as isize),
                    );
                }
                current_block = 17616027239959249775;
            }
            37 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof
                    && *p.offset(1 as c_int as isize) as c_int == '=' as i32
                {
                    return stb__clex_token(
                        lexer,
                        CLEX_modeq as c_int,
                        p,
                        p.offset(1 as c_int as isize),
                    );
                }
                current_block = 17616027239959249775;
            }
            42 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof
                    && *p.offset(1 as c_int as isize) as c_int == '=' as i32
                {
                    return stb__clex_token(
                        lexer,
                        CLEX_muleq as c_int,
                        p,
                        p.offset(1 as c_int as isize),
                    );
                }
                current_block = 17616027239959249775;
            }
            47 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof
                    && *p.offset(1 as c_int as isize) as c_int == '=' as i32
                {
                    return stb__clex_token(
                        lexer,
                        CLEX_diveq as c_int,
                        p,
                        p.offset(1 as c_int as isize),
                    );
                }
                current_block = 17616027239959249775;
            }
            60 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_lesseq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '<' as i32 {
                        if p.offset(2 as c_int as isize) != (*lexer).eof
                            && *p.offset(2 as c_int as isize) as c_int == '=' as i32
                        {
                            return stb__clex_token(
                                lexer,
                                CLEX_shleq as c_int,
                                p,
                                p.offset(2 as c_int as isize),
                            );
                        }
                        return stb__clex_token(
                            lexer,
                            CLEX_shl as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            62 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == '=' as i32 {
                        return stb__clex_token(
                            lexer,
                            CLEX_greatereq as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                    if *p.offset(1 as c_int as isize) as c_int == '>' as i32 {
                        if p.offset(2 as c_int as isize) != (*lexer).eof
                            && *p.offset(2 as c_int as isize) as c_int == '=' as i32
                        {
                            return stb__clex_token(
                                lexer,
                                CLEX_shreq as c_int,
                                p,
                                p.offset(2 as c_int as isize),
                            );
                        }
                        return stb__clex_token(
                            lexer,
                            CLEX_shr as c_int,
                            p,
                            p.offset(1 as c_int as isize),
                        );
                    }
                }
                current_block = 17616027239959249775;
            }
            34 => return stb__clex_parse_string(lexer, p, CLEX_dqstring as c_int),
            39 => {
                let mut start_0: *const c_char = p;
                (*lexer).int_number =
                    stb__clex_parse_char(p.offset(1 as c_int as isize), &mut p) as c_long;
                if (*lexer).int_number < 0 as c_int as c_long {
                    return stb__clex_token(lexer, CLEX_parse_error as c_int, start_0, start_0);
                }
                if p == (*lexer).eof || *p as c_int != '\'' as i32 {
                    return stb__clex_token(lexer, CLEX_parse_error as c_int, start_0, p);
                }
                return stb__clex_token(
                    lexer,
                    CLEX_charlit as c_int,
                    start_0,
                    p.offset(1 as c_int as isize),
                );
            }
            48 => {
                if p.offset(1 as c_int as isize) != (*lexer).eof {
                    if *p.offset(1 as c_int as isize) as c_int == 'x' as i32
                        || *p.offset(1 as c_int as isize) as c_int == 'X' as i32
                    {
                        let mut q: *const c_char = 0 as *mut c_char;
                        (*lexer).int_number = strtol(p, &mut q as *mut *const c_char, 16 as c_int);
                        if q == p.offset(2 as c_int as isize) {
                            return stb__clex_token(
                                lexer,
                                CLEX_parse_error as c_int,
                                p.offset(-(2 as c_int as isize)),
                                p.offset(-(1 as c_int as isize)),
                            );
                        }
                        return stb__clex_parse_suffixes(
                            lexer,
                            CLEX_intlit as c_int as c_long,
                            p,
                            q,
                            b"\0" as *const u8 as *const c_char,
                        );
                    }
                }
                current_block = 8569828448656383210;
            }
            49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                current_block = 8569828448656383210;
            }
            _ => {
                if *p as c_int >= 'a' as i32 && *p as c_int <= 'z' as i32
                    || *p as c_int >= 'A' as i32 && *p as c_int <= 'Z' as i32
                    || *p as c_int == '_' as i32
                    || *p as c_uchar as c_int >= 128 as c_int
                    || *p as c_int == '$' as i32
                {
                    let mut n: c_int = 0 as c_int;
                    (*lexer).string = (*lexer).string_storage;
                    loop {
                        if n + 1 as c_int >= (*lexer).string_storage_len {
                            return stb__clex_token(
                                lexer,
                                CLEX_parse_error as c_int,
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
                    *((*lexer).string).offset(n as isize) = 0 as c_int as c_char;
                    (*lexer).string_len = n;
                    return stb__clex_token(
                        lexer,
                        CLEX_id as c_int,
                        p,
                        p.offset(n as isize).offset(-(1 as c_int as isize)),
                    );
                }
                current_block = 17616027239959249775;
            }
        }
        match current_block {
            17616027239959249775 => return stb__clex_token(lexer, *p as c_int, p, p),
            _ => {
                let mut q_0: *const c_char = p;
                while q_0 != (*lexer).eof
                    && (*q_0 as c_int >= '0' as i32 && *q_0 as c_int <= '9' as i32)
                {
                    q_0 = q_0.offset(1);
                }
                if q_0 != (*lexer).eof {
                    if *q_0 as c_int == '.' as i32
                        || *q_0 as c_int == 'e' as i32
                        || *q_0 as c_int == 'E' as i32
                    {
                        (*lexer).real_number = strtod(p, &mut q_0 as *mut *const c_char);
                        return stb__clex_parse_suffixes(
                            lexer,
                            CLEX_floatlit as c_int as c_long,
                            p,
                            q_0,
                            b"\0" as *const u8 as *const c_char,
                        );
                    }
                }
                if *p.offset(0 as c_int as isize) as c_int == '0' as i32 {
                    let mut q_1: *const c_char = p;
                    (*lexer).int_number = strtol(p, &mut q_1 as *mut *const c_char, 8 as c_int);
                    return stb__clex_parse_suffixes(
                        lexer,
                        CLEX_intlit as c_int as c_long,
                        p,
                        q_1,
                        b"\0" as *const u8 as *const c_char,
                    );
                }
                let mut q_2: *const c_char = p;
                (*lexer).int_number = strtol(p, &mut q_2 as *mut *const c_char, 10 as c_int);
                return stb__clex_parse_suffixes(
                    lexer,
                    CLEX_intlit as c_int as c_long,
                    p,
                    q_2,
                    b"\0" as *const u8 as *const c_char,
                );
            }
        };
    }
}
