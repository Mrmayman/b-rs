use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=stb_c_lexer.h");

    let bindings = bindgen::Builder::default()
        .header("stb_c_lexer.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let header = std::fs::read_to_string("stb_c_lexer.h").unwrap();
    std::fs::write(
        "stb_c_lexer.c",
        format!("#define STB_C_LEXER_IMPLEMENTATION\n{header}"),
    )
    .unwrap();
    cc::Build::new().file("stb_c_lexer.c").compile("liblexer.a");

    println!("cargo:rustc-link-lib=lexer");
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    ); // Search path for the library
}
