use std::{
    ffi::{CStr, CString},
    fmt::Display,
};

use crate::stb_c_lexer::{StbLexer, stb_c_lexer_get_token};

// #[allow(non_upper_case_globals)]
// #[allow(dead_code)]
// mod bindings {
//     include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
// }

pub struct Lexer {
    inner: StbLexer,
    input_path: String,
}

impl Lexer {
    pub fn new(input: &str, input_path: &str) -> Self {
        let input_stream = CString::new(input).unwrap();

        // TODO: size of identifiers and string literals is limited because of stb_c_lexer.h
        let mut string_store: [i8; 4096] = unsafe { std::mem::zeroed() };

        let inner = StbLexer::new(
            input_stream,
            string_store.as_mut_ptr(),
            string_store.len() as i32,
        );

        Self {
            inner,
            input_path: input_path.to_owned(),
        }
    }

    pub fn diag(&mut self, msg: &str) {
        self.diag_at(msg, self.inner.where_firstchar);
    }

    pub fn where_firstchar(&self) -> *const i8 {
        self.inner.where_firstchar
    }

    pub fn diag_at(&mut self, msg: &str, whr: *const i8) {
        let loc = unsafe { self.inner.get_location_at(whr) };
        eprintln!(
            "{}:{}:{}: {msg}",
            self.input_path,
            loc.line_number,
            loc.line_offset + 1
        );
    }

    pub fn get_token(&mut self) -> Option<LexToken> {
        unsafe {
            if stb_c_lexer_get_token(&mut self.inner) == 0 {
                return None;
            }
        }

        // if self.inner.token == CLEX_CLEX_floatlit as i64 {
        //     return Some(LexToken::Float(self.inner.real_number));
        // }

        // if self.inner.token == CLEX_CLEX_intlit as i64 {
        //     return Some(LexToken::Int(self.inner.int_number));
        // }

        self.get_token_inner()
    }

    pub fn read_int(&self) -> Option<i64> {
        let tk = self.get_token_inner()?;
        if let LexToken::Lex(Clex::Intlit) = tk {
            Some(self.inner.int_number)
        } else {
            None
        }
    }

    fn get_token_inner(&self) -> Option<LexToken> {
        let token = self.inner.token;
        if token < 256 {
            Some(LexToken::Char(token as u8 as char))
        } else {
            Some(LexToken::Lex(unsafe { std::mem::transmute(token) }))
        }
    }

    fn read_string(&self) -> Option<String> {
        if self.inner.string.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(self.inner.string) }
            .to_str()
            .ok()?
            .to_owned();
        return Some(s);
    }

    pub fn get_ident(&mut self) -> String {
        _ = self.get_token().unwrap();
        self.expect_ident()
    }

    pub fn expect_ident(&mut self) -> String {
        self.expect(LexToken::Lex(Clex::Id));
        self.read_string().unwrap()
    }

    pub fn expect(&mut self, tk: LexToken) {
        let Some(token) = self.get_token_inner() else {
            self.diag("ERROR: reached end-of-file while reading token");
            std::process::exit(1);
        };

        if token != tk {
            self.diag(&format!("ERROR: expected {tk}, but got {token}"));
            std::process::exit(1);
        }
    }

    pub fn get_char(&mut self, ch: char) {
        self.get_and_expect(LexToken::Char(ch));
    }

    pub fn get_and_expect(&mut self, tk: LexToken) {
        _ = self.get_token();
        self.expect(tk);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexToken {
    Char(char),
    Lex(Clex),
}

impl Display for LexToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexToken::Char(c) => write!(f, "`{c}`"),
            LexToken::Lex(tk) => write!(f, "{tk:?}"),
        }
    }
}

impl LexToken {
    pub fn is_char(&self, c: char) -> bool {
        if let Self::Char(ch) = self {
            *ch == c
        } else {
            false
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i64)]
pub enum Clex {
    Eof = 256,
    ParseError,
    Intlit,
    Floatlit,
    Id,
    Dqstring,
    Sqstring,
    Charlit,
    Eq,
    Noteq,
    Lesseq,
    Greatereq,
    Andand,
    Oror,
    Shl,
    Shr,
    Plusplus,
    Minusminus,
    Pluseq,
    Minuseq,
    Muleq,
    Diveq,
    Modeq,
    Andeq,
    Oreq,
    Xoreq,
    Arrow,
    Eqarrow,
    Shleq,
    Shreq,
    FirstUnusedToken,
}
