use std::ffi::*;

#[allow(dead_code)]
mod lexer;
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;

macro_rules! diagf {
    ($l:expr, $fpath:expr, $where:expr, $fmt:literal $($args:tt)*) => {{
        let mut loc: stb_lex_location = unsafe {std::mem::zeroed()};
        unsafe { stb_c_lexer_get_location($l, $where, &mut loc) };
        eprint!("{}:{}:{}: ", $fpath, loc.line_number, loc.line_offset + 1);
        eprintln!($fmt $($args)*);
    }};
}

fn display_token_temp(token: c_long) -> String {
    // TODO: port print_token() from stb_c_lexer.h to display more tokens
    if token < 256 {
        format!("{}", token as u8 as char)
    } else {
        format!("{token}")
    }
}

fn expect_clex(l: &stb_lexer, input_path: &str, clex: c_long) -> bool {
    if l.token != clex {
        diagf!(
            l,
            input_path,
            l.where_firstchar,
            "ERROR: expected {}, but got {}\n",
            display_token_temp(clex),
            display_token_temp((*l).token)
        );
        false
    } else {
        true
    }
}

fn get_and_expect_clex(l: &mut stb_lexer, input_path: &str, clex: i64) -> bool {
    unsafe { stb_c_lexer_get_token(l) };
    expect_clex(l, input_path, clex)
}

#[derive(Clone)]
struct AutoVar {
    name: String,
    offset: usize,
    hwere: *mut c_char,
}

fn usage(program_name: &str) {
    eprintln!("Usage: {program_name} <input.b> <output.asm>");
}

fn main() {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();

    let Some(input_path) = args.next() else {
        usage(&program_name);
        eprintln!("ERROR: no input is provided");
        std::process::exit(69);
    };

    let Some(output_path) = args.next() else {
        usage(&program_name);
        eprintln!("ERROR: no output is provided");
        std::process::exit(69);
    };

    let mut vars: Vec<AutoVar> = Vec::new();
    let mut vars_offset: usize;

    let input = std::fs::read_to_string(&input_path).unwrap();
    let input_stream = CString::new(input).unwrap();

    let mut l: stb_lexer = unsafe { std::mem::zeroed() };
    let mut string_store: [c_char; 1024] = unsafe { std::mem::zeroed() }; // TODO: size of identifiers and string literals is limited because of stb_c_lexer.h
    unsafe {
        stb_c_lexer_init(
            &mut l,
            input_stream.as_ptr(),
            input_stream.as_ptr().add(input_stream.count_bytes()),
            string_store.as_mut_ptr(),
            string_store.len() as i32,
        )
    };

    let mut output = String::new();
    output.push_str("format ELF64\n");
    output.push_str("section \".text\" executable\n");

    'func: loop {
        vars.clear();
        vars_offset = 0;

        unsafe { stb_c_lexer_get_token(&mut l) };
        if l.token == CLEX_CLEX_eof as i64 {
            break 'func;
        }

        if !expect_clex(&mut l, &input_path, CLEX_CLEX_id as i64) {
            std::process::exit(1);
        }
        let l_string = strdup(l.string);
        output.push_str(&format!("public {l_string}\n"));
        output.push_str(&format!("{l_string}:\n"));
        if !get_and_expect_clex(&mut l, &input_path, '(' as i64) {
            std::process::exit(1);
        }
        if !get_and_expect_clex(&mut l, &input_path, ')' as i64) {
            std::process::exit(1);
        }
        if !get_and_expect_clex(&mut l, &input_path, '{' as i64) {
            std::process::exit(1);
        }

        output.push_str("    push rbp\n");
        output.push_str("    mov rbp, rsp\n");

        'body: loop {
            // Statement
            unsafe { stb_c_lexer_get_token(&mut l) };
            if l.token == '}' as i64 {
                output.push_str(&format!("    add rsp, {vars_offset}\n"));
                output.push_str(&format!("    pop rbp\n"));
                output.push_str("    mov rax, 0\n");
                output.push_str("    ret\n");
                break 'body;
            }
            if !expect_clex(&mut l, &input_path, CLEX_CLEX_id as i64) {
                std::process::exit(1);
            }
            if unsafe { CStr::from_ptr(l.string) } == c"extrn" {
                if !get_and_expect_clex(&mut l, &input_path, CLEX_CLEX_id as i64) {
                    std::process::exit(1);
                }
                let cstr = unsafe { CStr::from_ptr(l.string) };
                let l_string = cstr.to_str().unwrap();
                output.push_str(&format!("    extrn {l_string}\n"));
                // TODO: support multiple extrn declarations
                // TODO: report extrn redefinition
                if !get_and_expect_clex(&mut l, &input_path, ';' as i64) {
                    std::process::exit(1);
                }
            } else if unsafe { CStr::from_ptr(l.string) } == c"auto" {
                if !get_and_expect_clex(&mut l, &input_path, CLEX_CLEX_id as i64) {
                    std::process::exit(1);
                }
                vars_offset += 8;
                let name = strdup(l.string);
                let name_where = l.where_firstchar;

                let existing_var = vars.iter().find(|n| n.name == name);

                if let Some(existing_var) = existing_var {
                    diagf!(
                        &mut l,
                        input_path,
                        name_where,
                        "ERROR: redefinition of variable `{name}`\n",
                    );
                    diagf!(
                        &mut l,
                        input_path,
                        existing_var.hwere,
                        "NOTE: the first declaration is located here\n"
                    );
                    std::process::exit(69);
                }
                vars.push(AutoVar {
                    name: name.to_owned(),
                    offset: vars_offset,
                    hwere: l.where_firstchar,
                });
                // TODO: support multiple auto declarations
                output.push_str("    sub rsp, 8\n");
                if !get_and_expect_clex(&mut l, &input_path, ';' as i64) {
                    std::process::exit(1);
                }
            } else {
                let name = strdup(l.string);
                let name_where = l.where_firstchar;

                unsafe { stb_c_lexer_get_token(&mut l) };
                if l.token == '=' as i64 {
                    let Some(var_def) = vars.iter().find(|n| n.name == name) else {
                        diagf!(
                            &mut l,
                            input_path,
                            name_where,
                            "ERROR: could not find variable `{name}`\n",
                        );
                        std::process::exit(69);
                    };

                    // NOTE: expecting only int literal here for now
                    if !get_and_expect_clex(&mut l, &input_path, CLEX_CLEX_intlit as i64) {
                        std::process::exit(1);
                    }
                    output.push_str(&format!(
                        "    mov QWORD [rbp-{}], {}\n",
                        var_def.offset, l.int_number,
                    ));
                    if !get_and_expect_clex(&mut l, &input_path, ';' as i64) {
                        std::process::exit(1);
                    }
                } else if l.token == '(' as i64 {
                    // NOTE: expecting only var read here for now

                    unsafe { stb_c_lexer_get_token(&mut l) };
                    if l.token == ')' as i64 {
                        // TODO: report calling unknown functions
                        output.push_str(&format!("    call {name}\n"));
                        if !get_and_expect_clex(&mut l, &input_path, ';' as i64) {
                            std::process::exit(1);
                        }
                    } else {
                        if !expect_clex(&mut l, &input_path, CLEX_CLEX_id as i64) {
                            std::process::exit(1);
                        }
                        let l_string = strdup(l.string);
                        let Some(var_def) = vars.iter().find(|n| n.name == l_string) else {
                            diagf!(
                                &mut l,
                                input_path,
                                name_where,
                                "ERROR: could not find variable `{name}`\n",
                            );
                            std::process::exit(69);
                        };

                        output.push_str(&format!("    mov rdi, [rbp-{}]\n", (*var_def).offset));
                        output.push_str(&format!("    call {name}\n"));

                        if !get_and_expect_clex(&mut l, &input_path, ')' as i64) {
                            std::process::exit(1);
                        }
                        if !get_and_expect_clex(&mut l, &input_path, ';' as i64) {
                            std::process::exit(1);
                        }
                    }
                } else {
                    diagf!(
                        &mut l,
                        input_path,
                        l.where_firstchar,
                        "ERROR: unexpected token `{}`\n",
                        display_token_temp(l.token)
                    );
                    std::process::exit(69);
                }
            }
        }
    }
    std::fs::write(output_path, &output).unwrap();
}

fn strdup(ptr: *mut i8) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned()
}

// TODO: B lexing is different from the C one.
//   Hack stb_c_lexer.h into stb_b_lexer.h
// TODO: Create a roadmap based on the spec.
