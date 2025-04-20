use stb::Lexer;

mod stb;
mod stb_c_lexer;

struct AutoVar {
    name: String,
    offset: usize,
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
    let mut l = Lexer::new(&input, &input_path);

    let mut output = String::new();
    output.push_str("format ELF64\n");
    output.push_str("section \".text\" executable\n");

    loop {
        vars.clear();
        vars_offset = 0;

        let Some(_) = l.get_token() else {
            break;
        };

        let func_name = l.expect_ident();

        output.push_str(&format!("public {func_name}\n"));
        output.push_str(&format!("{func_name}:\n"));

        l.get_char('(');
        l.get_char(')');
        l.get_char('{');

        output.push_str("    push rbp\n");
        output.push_str("    mov rbp, rsp\n");

        'body: loop {
            let Some(token) = l.get_token() else {
                break;
            };

            if token.is_char('}') {
                output.push_str(&format!("    add rsp, {vars_offset}\n"));
                output.push_str(&format!("    pop rbp\n"));
                output.push_str("    mov rax, 0\n");
                output.push_str("    ret\n");
                break 'body;
            };

            let name = l.expect_ident();
            if name == "extrn" {
                let extern_fn = l.get_ident();
                output.push_str(&format!("    extrn {extern_fn}\n"));
                // TODO: support multiple extrn declarations
                // TODO: report extrn redefinition
                l.get_char(';');
            } else if name == "auto" {
                let name = l.get_ident();
                vars_offset += 8;

                if vars.iter().any(|n| n.name == name) {
                    l.diag(&format!("ERROR: redefinition of variable `{name}`"));
                    l.diag("NOTE: the first declaration is located here");
                    std::process::exit(69);
                }
                vars.push(AutoVar {
                    name: name.to_owned(),
                    offset: vars_offset,
                });
                // TODO: support multiple auto declarations
                output.push_str("    sub rsp, 8\n");
                l.get_char(';');
            } else {
                let token = l.get_token().unwrap();
                if token.is_char('=') {
                    let Some(var_def) = vars.iter().find(|n| n.name == name) else {
                        l.diag(&format!("ERROR: could not find variable `{name}`"));
                        std::process::exit(69);
                    };

                    // NOTE: expecting only int literal here for now
                    _ = l.get_token();
                    let Some(num) = l.read_int() else {
                        l.diag("ERROR: Non-integer type detected");
                        std::process::exit(69);
                    };
                    output.push_str(&format!("    mov QWORD [rbp-{}], {num}\n", var_def.offset));
                    l.get_char(';');
                } else if token.is_char('(') {
                    // NOTE: expecting only var read here for now

                    let token = l.get_token().unwrap();
                    if token.is_char(')') {
                        // TODO: report calling unknown functions
                        output.push_str(&format!("    call {name}\n"));
                        l.get_char(';');
                    } else {
                        let l_string = l.expect_ident();
                        let Some(var_def) = vars.iter().find(|n| n.name == l_string) else {
                            l.diag(&format!("ERROR: could not find variable `{name}`"));
                            std::process::exit(69);
                        };

                        output.push_str(&format!("    mov rdi, [rbp-{}]\n", (*var_def).offset));
                        output.push_str(&format!("    call {name}\n"));

                        l.get_char(')');
                        l.get_char(';');
                    }
                } else {
                    l.diag(&format!("ERROR: unexpected token `{token}`\n"));
                    std::process::exit(69);
                }
            }
        }
    }
    std::fs::write(output_path, &output).unwrap();
}

// TODO: B lexing is different from the C one.
//   Hack stb_c_lexer.h into stb_b_lexer.h
// TODO: Create a roadmap based on the spec.
