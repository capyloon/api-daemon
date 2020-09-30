/// Generates documentation for the Javascript client.
use crate::ast_utils::*;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use log::error;
use sidl_parser::ast::{Arity, Ast, ConcreteType, TypeExtraDecorator};
use std::io::Write;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Handlebars rendering error")]
    HandlebarsRender(#[from] RenderError),
    #[error("Handlebars error")]
    Handlebars(#[from] handlebars::TemplateError),
}

type Result<T> = std::result::Result<T, Error>;

pub fn js_type(typ: &ConcreteType, arity: &Arity) -> Vec<String> {
    let mut res = String::new();
    match arity {
        Arity::OneOrMore => res.push_str("["),
        Arity::ZeroOrMore => res.push_str("["),
        Arity::Unary | Arity::Optional => {}
    }

    match typ {
        ConcreteType::Void => res.push_str("void"),
        ConcreteType::Bool => res.push_str("boolean"),
        ConcreteType::Int => res.push_str("integer"),
        ConcreteType::Float => res.push_str("float"),
        ConcreteType::Str => res.push_str("string"),
        ConcreteType::Binary => res.push_str("arraybuffer"),
        ConcreteType::Json => res.push_str("json"),
        ConcreteType::Date => res.push_str("Date"),
        ConcreteType::Dictionary(ref utype)
        | ConcreteType::Enumeration(ref utype)
        | ConcreteType::Interface(ref utype)
        | ConcreteType::Callback(ref utype) => {
            res.push_str(&utype.to_string());
        }
    }

    match arity {
        Arity::Optional => res.push_str("?"),
        Arity::OneOrMore => res.push_str("]"),
        Arity::ZeroOrMore => res.push_str("]?"),
        Arity::Unary => {}
    }

    let link = match typ {
        ConcreteType::Void
        | ConcreteType::Bool
        | ConcreteType::Int
        | ConcreteType::Float
        | ConcreteType::Str
        | ConcreteType::Binary
        | ConcreteType::Date
        | ConcreteType::Json => "".to_owned(),
        ConcreteType::Dictionary(ref utype) => format!("#dictionary_{}", utype),
        ConcreteType::Enumeration(ref utype) => format!("#enumeration_{}", utype),
        ConcreteType::Interface(ref utype) => format!("#interface_{}", utype),
        ConcreteType::Callback(ref utype) => format!("#callback_{}", utype),
    };

    vec![res, link]
}

pub fn get_decorator() -> TypeExtraDecorator {
    std::rc::Rc::new(|typ, arity| js_type(typ, arity))
}

pub struct Codegen {
    ast: Ast,
    library_name: String,
}

impl Codegen {
    pub fn generate<'a, W: Write>(&self, sink: &'a mut W) -> Result<()> {
        if self.ast.services.len() > 1 {
            error!(
                "Only one service supported, but found {}",
                self.ast.services.len()
            );
            return Ok(());
        }

        // Include our Handlerbar template inline so we won't have runtime path issues.
        let main_template = include_str!("templates/javascript_html.handlebars");
        let style_css = include_str!("templates/style.css");

        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("javascript_documentation", main_template)?;
        handlebars.register_template_string("style_css", style_css)?;

        // Helper that gives access to the library name, needed to create code samples.
        handlebars.register_helper(
            "sidl-name",
            Box::new(
                |_h: &Helper,
                 _r: &Handlebars,
                 _: &Context,
                 _rc: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    out.write(&self.library_name)?;
                    Ok(())
                },
            ),
        );

        // Helper that creates a link or a simple text if there is no target in the second parameter.
        handlebars.register_helper(
            "maybe-link",
            Box::new(
                |h: &Helper,
                 _r: &Handlebars,
                 _: &Context,
                 _rc: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let text = h
                        .param(0)
                        .and_then(|v| v.value().as_str())
                        .ok_or_else(|| RenderError::new("param not found"))?;
                    let link = h
                        .param(1)
                        .and_then(|v| v.value().as_str())
                        .ok_or_else(|| RenderError::new("param not found"))?;
                    if link.is_empty() {
                        out.write(text)?;
                    } else {
                        out.write(&format!("<a href=\"{}\">{}</a>", link, text))?;
                    }
                    Ok(())
                },
            ),
        );

        // Helper that returns the UPPER_SNAKE_CASE variant of a string
        handlebars.register_helper(
            "upper-snake-case",
            Box::new(
                |h: &Helper,
                 _r: &Handlebars,
                 _: &Context,
                 _rc: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    use heck::ShoutySnakeCase;

                    let text = h
                        .param(0)
                        .and_then(|v| v.value().as_str())
                        .ok_or_else(|| RenderError::new("param not found"))?;

                    out.write(&text.to_shouty_snake_case())?;
                    Ok(())
                },
            ),
        );

        handlebars.render_to_write("javascript_documentation", &self.ast, sink)?;

        // Also generates the json version of the AST to help with handlebar authoring.
        let file = std::fs::File::create("service.json").unwrap();
        let _ = serde_json::to_writer_pretty(file, &self.ast);

        Ok(())
    }

    pub fn new(ast: Ast, library_name: &str) -> Codegen {
        // Augment type information in the codegen to help the handlebar template.
        Codegen {
            ast: normalize_rust_case(&ast, &JavascriptCaseNormalizer),
            library_name: library_name.into(),
        }
    }
}
