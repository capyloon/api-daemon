extern crate clap;
extern crate env_logger;

#[macro_use]
extern crate log;
extern crate sidl_codegen;
extern crate sidl_parser;

use clap::{App, Arg};
use std::path::Path;

use sidl_codegen::{generate_javascript_code, generate_rust_service};

fn main() {
    env_logger::init();

    let matches = App::new("codegen")
        .version("1.0")
        .about("Generate code from sidl files")
        .arg(
            Arg::with_name("input")
                .help("Input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("rust")
                .short("r")
                .long("rust")
                .value_name("rust")
                .help("Path to the generated Rust code.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("js")
                .short("j")
                .long("js")
                .value_name("js")
                .help("Path to the generated Javascript.")
                .takes_value(true),
        )
        .get_matches();

    let input = Path::new(matches.value_of("input").unwrap());

    if let Some(path) = matches.value_of("rust") {
        if let Err(err) = generate_rust_service(input, Path::new(path)) {
            error!("{}", err);
        }
    }

    if let Some(path) = matches.value_of("js") {
        if let Err(err) = generate_javascript_code(input, Path::new(path), None) {
            error!("{}", err);
        }
    }
}
