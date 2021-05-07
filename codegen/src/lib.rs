use log::info;
use sidl_parser::ast;
use std::fs::File;
use std::path::Path;
use thiserror::Error as ThisError;

pub mod ast_utils;
pub mod config;
pub mod doc_javascript;
pub mod helpers;
pub mod javascript;
pub mod rust_common;
pub mod rust_service;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Ast error")]
    Ast(#[from] ast::AstError),
    #[error("Rust Common Codegen error")]
    RustCommon(#[from] rust_common::Error),
    #[error("Rust Service Codegen error")]
    RustService(#[from] rust_service::Error),
    #[error("Javascript Codegen error")]
    JavascriptCode(#[from] javascript::Error),
    #[error("Javascript Documentation error")]
    JavascriptDoc(#[from] doc_javascript::Error),
    #[error("IO error")]
    Io(#[from] ::std::io::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

pub fn generate_rust_service(src: &Path, dest: &Path) -> Result<()> {
    info!("Generating full Rust code {:?} -> {:?}", src, dest);
    let ast = ast::Ast::parse_file(src, None)?;

    // May need to create directory
    std::fs::create_dir_all(dest)?;

    let mut file = File::create(dest.join("service.rs"))?;
    rust_service::Codegen::generate(ast.clone(), &mut file)?;

    let mut file = File::create(dest.join("common.rs"))?;
    rust_common::Codegen::generate(ast.clone(), &mut file)?;

    let mut file = File::create(dest.join("gecko_client.rs"))?;
    rust_common::Codegen::generate_gecko(ast, &mut file)?;

    Ok(())
}

pub fn generate_javascript_code(
    src: &Path,
    dest: &Path,
    config: Option<config::Config>,
) -> Result<()> {
    info!("Generating Javascript code {:?} -> {:?}", src, dest);
    let ast = ast::Ast::parse_file(src, None)?;

    if let Some(parent) = dest.parent() {
        // May need to create directory
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let mut generator = javascript::Codegen::new(ast);
    generator.generate(&mut file, &config.unwrap_or_default())?;
    Ok(())
}

pub fn generate_javascript_doc(src: &Path, dest: &Path, name: &str) -> Result<()> {
    info!(
        "Generating Javascript documentation {:?} -> {:?}",
        src, dest
    );
    let ast = ast::Ast::parse_file(src, Some(doc_javascript::get_decorator()))?;

    if let Some(parent) = dest.parent() {
        // May need to create directory
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(dest)?;
    let generator = doc_javascript::Codegen::new(ast, name);
    generator.generate(&mut file)?;
    Ok(())
}

#[test]
fn test_generate_rust() {
    const CONTENT: &'static str = r#"import "common_types.sidl" // test

    enum Kind {
        data
        elf
    }

    dictionary Bin {
        len: int
        body: str
    }

    interface MyType {
        #[js_name=clef]
        key: int
        value: Bin
        kind: Kind
    }

    #[service annotation]
    interface TestServiceInterface {
        foo: bool
        bar: int+
        baz: Kind+

        imported: ImportedType

        #[rust_name=do_it]
        fn doIt(what: binary?, which: Kind) -> MyType
    }

    service TestService: TestServiceInterface
    "#;

    use sidl_parser::ast::Ast;

    let ast = Ast::parse_str("test2", CONTENT, None).unwrap();

    rust_service::Codegen::generate(ast, &mut ::std::io::stdout())
        .expect("Failed to generate Rust code!");
}
