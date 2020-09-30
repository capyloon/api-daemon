use log::debug;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::parser::{ParserContext, ParserError, Token, TokenKind};

#[derive(Error, Debug)]
pub enum AstError {
    #[error("Unknown type: `{0}`")]
    NoSuchType(String),
    #[error("Unexpected Top Level Identifier: `{0}`")]
    UnexpectedTopLevel(String),
    #[error("Invalid Service Type: {0:?} {1:?}")]
    InvalidServiceType(ConcreteType, Arity),
    #[error("Parser error")]
    Parser(#[from] ParserError),
    #[error("Invalid Token: `{0}`")]
    InvalidToken(String),
    #[error("Invalid Interface Parameter: `{0}`")]
    InvalidInterfaceParameter(String),
}

type Result<T> = ::std::result::Result<T, AstError>;

/// Extension point to add custom information to FullConcreteType.
pub type TypeExtraDecorator = std::rc::Rc<dyn Fn(&ConcreteType, &Arity) -> Vec<String>>;

/// The trait to implement by ast elements that can be built by
/// parsing.
trait Parseable: Sized {
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self>;
}

/// Representation of a type, which is either a built-in type or
/// a user defined one.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ConcreteType {
    Void,
    Bool,
    Int,
    Str,
    Float,
    Binary,
    Date,
    Json,
    Callback(String),
    Enumeration(String),
    Dictionary(String),
    Interface(String),
}

impl ConcreteType {
    fn builtin_from_str(typ: &str) -> Option<Self> {
        let res = {
            if typ == "void" {
                ConcreteType::Void
            } else if typ == "bool" {
                ConcreteType::Bool
            } else if typ == "int" {
                ConcreteType::Int
            } else if typ == "str" {
                ConcreteType::Str
            } else if typ == "float" {
                ConcreteType::Float
            } else if typ == "binary" {
                ConcreteType::Binary
            } else if typ == "date" {
                ConcreteType::Date
            } else if typ == "json" {
                ConcreteType::Json
            } else {
                return None;
            }
        };

        Some(res)
    }

    fn from(typ: &str, ast: &Ast) -> Option<Self> {
        match ConcreteType::builtin_from_str(typ) {
            Some(typ) => Some(typ),
            None => {
                // Not a default type, check if it's a callback, interface, enumeration or a dictionary one.
                if ast.callbacks.contains_key(typ) {
                    Some(ConcreteType::Callback(typ.to_owned()))
                } else if ast.dictionaries.contains_key(typ) {
                    Some(ConcreteType::Dictionary(typ.to_owned()))
                } else if ast.interfaces.contains_key(typ) {
                    Some(ConcreteType::Interface(typ.to_owned()))
                } else if ast.enumerations.contains_key(typ) {
                    Some(ConcreteType::Enumeration(typ.to_owned()))
                } else {
                    None
                }
            }
        }
    }
}

/// Defines the arity of a type.
/// Default to Unary.
/// `+` means `OneOrMore`
/// `*` means `ZeroOrMore`
/// `?` means `Optional`
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Arity {
    Unary,
    OneOrMore,
    ZeroOrMore,
    Optional,
}

impl Default for Arity {
    fn default() -> Self {
        Arity::Unary
    }
}

impl Arity {
    fn parse(ctxt: &mut ParserContext) -> Arity {
        // Check if there is an arity marker.
        {
            if ctxt.peek_token(TokenKind::Expected("?".to_owned())).is_ok() {
                Arity::Optional
            } else if ctxt.peek_token(TokenKind::Expected("*".to_owned())).is_ok() {
                Arity::ZeroOrMore
            } else if ctxt.peek_token(TokenKind::Expected("+".to_owned())).is_ok() {
                Arity::OneOrMore
            } else {
                Arity::default()
            }
        }
    }
}

/// Annotations are optional lists of comma separated values
/// that are used to give hints to the code generators.
#[derive(Clone, Debug, Serialize)]
pub struct Annotation {
    pub content: Vec<String>,
}

impl Annotation {
    /// Helper to get an optional annotation in the current context.
    fn parse(ctxt: &mut ParserContext) -> Result<Option<Self>> {
        // First check if there is an annotation available.
        // If so, save it to transfer to the forthcoming top level item.
        match ctxt.peek_token(TokenKind::Annotation) {
            Ok(Token::Annotation(val)) => Ok(Some(Annotation {
                content: val.split(',').map(|e| e.trim().to_owned()).collect(),
            })),
            Ok(token) => {
                // That should never happen!
                debug!("Unexpected token: {:?}", token);
                Ok(None)
            }
            Err(ParserError::PeekError) => {
                // Failed to get an annotation, moving on.
                Ok(None)
            }
            Err(err) => {
                // Something else went wrong, just report the error.
                debug!("Unexpected error: {:?}", err);
                Err(AstError::Parser(err))
            }
        }
    }

    /// Convenience method to check if a fixed annotation exists.
    pub fn has(&self, what: &str) -> bool {
        self.content.iter().any(|e| e == what)
    }

    /// Return the content of all annotations starting by what=
    pub fn get_values(&self, what: &str) -> Vec<&str> {
        let delim = format!("{}=", what);
        self.content
            .iter()
            .filter_map(|e| {
                if e.starts_with(&delim) {
                    Some(e.split('=').collect::<Vec<&str>>()[1])
                } else {
                    None
                }
            })
            .collect()
    }
}

/// A concrete type and its arity.
#[derive(Clone, Debug, Serialize)]
pub struct FullConcreteType {
    pub typ: ConcreteType,
    pub arity: Arity,
    pub extra: Vec<String>, // used to carry custom information, eg. in codegen.
}

impl FullConcreteType {
    fn new(typ: ConcreteType, arity: Arity, decorator: Option<TypeExtraDecorator>) -> Self {
        let extra = match decorator {
            Some(decorator) => decorator(&typ, &arity),
            None => vec![],
        };

        FullConcreteType { typ, arity, extra }
    }

    fn default_with_decorator(decorator: Option<TypeExtraDecorator>) -> Self {
        let extra = match decorator {
            Some(decorator) => decorator(&ConcreteType::Void, &Arity::Unary),
            None => vec![],
        };

        FullConcreteType {
            typ: ConcreteType::Void,
            arity: Arity::Unary,
            extra,
        }
    }
}

/// A return type is a success type and an error type, each with arity.
#[derive(Clone, Debug, Serialize)]
pub struct ReturnType {
    pub success: FullConcreteType,
    pub error: FullConcreteType,
}

impl ReturnType {
    fn default_with_decorator(decorator: Option<TypeExtraDecorator>) -> Self {
        Self {
            success: FullConcreteType::default_with_decorator(decorator.clone()),
            error: FullConcreteType::default_with_decorator(decorator),
        }
    }
}

/// A method parameter.
#[derive(Clone, Debug, Serialize)]
pub struct MethodParameter {
    pub name: String,
    pub typ: FullConcreteType,
}

impl MethodParameter {
    fn new(name: &str, typ: FullConcreteType) -> Self {
        MethodParameter {
            name: name.into(),
            typ,
        }
    }
}

/// A method call, with a set of parameters and a return type.
#[derive(Debug, Clone, Serialize)]
pub struct Method {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub params: Vec<MethodParameter>,
    pub returns: ReturnType,
}

// Parses a type & arity token sequence.
fn get_type_arity(ctxt: &mut ParserContext, ast: &mut Ast) -> Result<FullConcreteType> {
    let stype = ctxt.next_token(TokenKind::Identifier)?.as_str();
    let ctype = ConcreteType::from(&stype, ast);
    if ctype.is_none() {
        return Err(AstError::NoSuchType(stype));
    }

    Ok(FullConcreteType::new(
        ctype.unwrap(),
        Arity::parse(ctxt),
        ctxt.decorator.clone(),
    ))
}

// Parses a `param: type` token sequence.
fn get_param(ctxt: &mut ParserContext, ast: &mut Ast) -> Result<FullConcreteType> {
    let _ = ctxt.next_token(TokenKind::Expected(":".to_owned()))?;

    get_type_arity(ctxt, ast)
}

impl Method {
    fn has_interface_param(&self) -> Result<()> {
        for param in &self.params {
            if let ConcreteType::Interface(_) = param.typ.typ {
                return Err(AstError::InvalidInterfaceParameter(param.name.clone()));
            }
        }
        Ok(())
    }
}

impl Parseable for Method {
    // Parses a method definition:
    // fn method_name(param1: type1, param2: type2, ...) -> return_type
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected("(".to_owned()))?;

        let mut method = Method {
            annotation,
            name,
            params: vec![],
            returns: ReturnType::default_with_decorator(ctxt.decorator.clone()),
        };

        // Special case for empty parameter lists.
        let has_params = ctxt
            .peek_token(TokenKind::Expected(")".to_owned()))
            .is_err();

        // Get all the parameters
        if has_params {
            loop {
                let name = ctxt.next_token(TokenKind::Identifier)?.as_str();
                let full_type = get_param(ctxt, ast)?;
                method.params.push(MethodParameter::new(&name, full_type));
                // Search for either `,` or `)`
                let is_done = ctxt.peek_token(TokenKind::Expected(")".to_owned())).is_ok();
                if is_done {
                    break;
                }
                let _ = ctxt.next_token(TokenKind::Expected(",".to_owned()))?;
            }
        }

        // Now check if we have a non-void return type.
        let has_return_type = ctxt
            .peek_token(TokenKind::Expected("->".to_owned()))
            .is_ok();
        if has_return_type {
            // We expect success_type[, error_type]
            let mut return_type = ReturnType::default_with_decorator(ctxt.decorator.clone());

            // Check the success type.
            return_type.success = get_type_arity(ctxt, ast)?;

            let has_error_return = ctxt.peek_token(TokenKind::Expected(",".to_owned())).is_ok();
            if has_error_return {
                // Check the error type.
                return_type.error = get_type_arity(ctxt, ast)?;
            }

            method.returns = return_type;
        }

        Ok(method)
    }
}

/// A event, with a return type.
#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub returns: FullConcreteType,
}

impl Parseable for Event {
    // Parses a Event definition:
    // event event_name -> data_type
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let name = ctxt.next_token(TokenKind::Identifier)?.as_str();

        let mut event = Event {
            annotation,
            name,
            returns: FullConcreteType::default_with_decorator(ctxt.decorator.clone()),
        };

        // Now check if we have a non-void return type.
        if ctxt
            .peek_token(TokenKind::Expected("->".to_owned()))
            .is_ok()
        {
            event.returns = get_type_arity(ctxt, ast)?;
        }

        Ok(event)
    }
}

/// A callback definition, which is a set of methods.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Callback {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub methods: BTreeMap<String, Method>,
}

impl Parseable for Callback {
    // Parses a callback definition: a set of methods.
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let mut callback = Callback::default();
        callback.annotation = annotation;
        callback.name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected("{".to_owned()))?;

        // Add an empty interface to get the type recognized.
        ast.callbacks
            .insert(callback.name.clone(), Callback::default());

        loop {
            // Are we done?
            if ctxt.peek_token(TokenKind::Expected("}".to_owned())).is_ok() {
                break;
            }

            let annotation = Annotation::parse(ctxt)?;
            let id = ctxt.next_token(TokenKind::Identifier)?.as_str();
            // If the id is `fn` this is a method. If not, parse it as a member.
            if id == "fn" {
                let method = Method::parse(ctxt, ast, annotation)?;
                callback.methods.insert(method.name.clone(), method);
            } else {
                // Nothing else expected...
                return Err(AstError::InvalidToken(id));
            }
        }

        // Remove the dummy interface.
        ast.callbacks.remove(&callback.name);

        Ok(callback)
    }
}

/// An interface definition, which is a set of typed members,
/// methods and events.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Interface {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub members: BTreeMap<String, DictionaryMember>,
    pub methods: BTreeMap<String, Method>,
    pub events: BTreeMap<String, Event>,
}

impl Parseable for Interface {
    // Parses an interface definition: a set of members and methods.
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let mut interface = Interface::default();
        interface.annotation = annotation;
        interface.name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected("{".to_owned()))?;

        // Add an empty interface to get the type recognized.
        ast.interfaces
            .insert(interface.name.clone(), Interface::default());

        loop {
            // Are we done?
            if ctxt.peek_token(TokenKind::Expected("}".to_owned())).is_ok() {
                break;
            }

            let annotation = Annotation::parse(ctxt)?;
            let id = ctxt.next_token(TokenKind::Identifier)?.as_str();
            // If the id is `fn` this is a method. If not, parse it as a member.
            if id == "fn" {
                let method = Method::parse(ctxt, ast, annotation)?;
                // Interfaces methods can't have interface members (only callbacks can).
                method.has_interface_param()?;
                interface.methods.insert(method.name.clone(), method);
            } else if id == "event" {
                //event will parse a single direction message
                let event = Event::parse(ctxt, ast, annotation)?;
                interface.events.insert(event.name.clone(), event);
            } else {
                // Not a method or an event, consider this is a member.
                let full_type = get_param(ctxt, ast)?;
                interface.members.insert(
                    id.clone(),
                    DictionaryMember::new(&id, annotation, full_type),
                );
            }
        }

        // Remove the dummy interface.
        ast.interfaces.remove(&interface.name);

        Ok(interface)
    }
}

/// A dictionary member.
#[derive(Debug, Clone, Serialize)]
pub struct DictionaryMember {
    pub name: String,
    pub annotation: Option<Annotation>,
    pub typ: FullConcreteType,
}

impl DictionaryMember {
    pub fn new(name: &str, annotation: Option<Annotation>, typ: FullConcreteType) -> Self {
        DictionaryMember {
            name: name.into(),
            annotation,
            typ,
        }
    }
}

/// A dictionary definition, which is a set of typed members.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Dictionary {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub members: Vec<DictionaryMember>,
}

impl Parseable for Dictionary {
    // Parses a dictionary definition: a set of members that can't be interfaces.
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let mut dictionary = Dictionary::default();
        dictionary.annotation = annotation;
        dictionary.name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected("{".to_owned()))?;

        // Add an empty interface to get the type recognized.
        ast.dictionaries
            .insert(dictionary.name.clone(), Dictionary::default());

        loop {
            // Are we done?
            if ctxt.peek_token(TokenKind::Expected("}".to_owned())).is_ok() {
                break;
            }

            let annotation = Annotation::parse(ctxt)?;
            let name = ctxt.next_token(TokenKind::Identifier)?.as_str();
            let typ = get_param(ctxt, ast)?;
            let member = DictionaryMember {
                name,
                annotation,
                typ,
            };
            dictionary.members.push(member);
        }

        // Remove the dummy interface.
        ast.dictionaries.remove(&dictionary.name);

        Ok(dictionary)
    }
}

/// A enumeration member.
#[derive(Debug, Clone, Serialize)]
pub struct EnumerationMember {
    pub name: String,
    pub order: usize,
    pub annotation: Option<Annotation>,
}

/// A enumeration definition, which is a predefined list of value.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Enumeration {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub members: Vec<EnumerationMember>,
}

impl Parseable for Enumeration {
    // Parses a enumeration definition: start from 0, not allow alias.
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let mut enumeration = Enumeration::default();
        enumeration.annotation = annotation;
        enumeration.name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected("{".to_owned()))?;

        // Add an empty interface to get the type recognized.
        ast.enumerations
            .insert(enumeration.name.clone(), Enumeration::default());
        let mut i = 0;
        loop {
            // Are we done?
            if ctxt.peek_token(TokenKind::Expected("}".to_owned())).is_ok() {
                break;
            }

            let annotation = Annotation::parse(ctxt)?;
            let id = ctxt.next_token(TokenKind::Identifier)?.as_str();
            let member = EnumerationMember {
                name: id,
                annotation,
                order: i,
            };
            enumeration.members.push(member);
            i += 1;
        }

        // Remove the dummy interface.
        ast.enumerations.remove(&enumeration.name);

        Ok(enumeration)
    }
}

struct Import;

impl Import {
    fn parse(ctxt: &mut ParserContext, ast: &mut Ast, _: Option<Annotation>) -> Result<()> {
        // TODO: actually import something...
        let path = ctxt.next_token(TokenKind::LitteralString)?.as_str();
        debug!("Importing {}", path);
        let mut imported_ctxt = ParserContext::from_file(path, ctxt.decorator.clone())?;
        ast.parse_from_ctxt(&mut imported_ctxt)?;

        Ok(())
    }
}

/// A service definition, mapping a service name to an interface type.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Service {
    pub annotation: Option<Annotation>,
    pub name: String,
    pub interface: String,
}

impl Parseable for Service {
    // Parses a service definition.
    fn parse(
        ctxt: &mut ParserContext,
        ast: &mut Ast,
        annotation: Option<Annotation>,
    ) -> Result<Self> {
        let mut service = Service::default();
        service.annotation = annotation;
        service.name = ctxt.next_token(TokenKind::Identifier)?.as_str();
        let _ = ctxt.next_token(TokenKind::Expected(":".to_owned()))?;
        let full_type = get_type_arity(ctxt, ast)?;
        let (ctype, arity) = (full_type.typ, full_type.arity);
        match (ctype, arity) {
            (ConcreteType::Interface(utype), Arity::Unary) => {
                service.interface = utype;
                Ok(service)
            }
            (typ, arity) => Err(AstError::InvalidServiceType(typ, arity)),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Ast {
    pub callbacks: BTreeMap<String, Callback>,
    pub interfaces: BTreeMap<String, Interface>,
    pub dictionaries: BTreeMap<String, Dictionary>,
    pub enumerations: BTreeMap<String, Enumeration>,
    pub services: Vec<Service>,
}

impl Ast {
    pub fn parse_str(
        source: &str,
        input: &str,
        decorator: Option<TypeExtraDecorator>,
    ) -> Result<Ast> {
        let mut ctxt = ParserContext::from_str(source, input, decorator)?;

        let mut ast = Ast::default();
        ast.parse_from_ctxt(&mut ctxt)?;
        Ok(ast)
    }

    pub fn parse_file<P: AsRef<Path>>(
        path: P,
        decorator: Option<TypeExtraDecorator>,
    ) -> Result<Ast> {
        let mut ctxt = ParserContext::from_file(path, decorator)?;

        let mut ast = Ast::default();
        ast.parse_from_ctxt(&mut ctxt)?;
        Ok(ast)
    }

    pub fn parse_from_ctxt(&mut self, mut ctxt: &mut ParserContext) -> Result<()> {
        // Consume tokens, and build the ast as we go.
        loop {
            let annotation = Annotation::parse(&mut ctxt)?;

            // Get the identifier of the top level construct:
            // import, interface or fn.
            let token = ctxt.next_token(TokenKind::Identifier);
            match token {
                Err(err) => match err {
                    ParserError::Eof => {
                        break;
                    }
                    _ => {
                        debug!("Failed to get identifier: {:?}", err);
                        return Err(err.into());
                    }
                },
                Ok(token) => {
                    let id = token.as_str();
                    if id == "import" {
                        Import::parse(&mut ctxt, self, annotation)?;
                    } else if id == "enum" {
                        let enumeration = Enumeration::parse(&mut ctxt, self, annotation)?;
                        self.enumerations
                            .insert(enumeration.name.clone(), enumeration);
                    } else if id == "dictionary" {
                        let dictionary = Dictionary::parse(&mut ctxt, self, annotation)?;
                        self.dictionaries
                            .insert(dictionary.name.clone(), dictionary);
                    } else if id == "service" {
                        let service = Service::parse(&mut ctxt, self, annotation)?;
                        self.services.push(service);
                    } else if id == "interface" {
                        let interface = Interface::parse(&mut ctxt, self, annotation)?;
                        self.interfaces.insert(interface.name.clone(), interface);
                    } else if id == "callback" {
                        let callback = Callback::parse(&mut ctxt, self, annotation)?;
                        self.callbacks.insert(callback.name.clone(), callback);
                    } else {
                        return Err(AstError::UnexpectedTopLevel(id));
                    }
                }
            }
        }

        Ok(())
    }
}

#[test]
fn parse_simple() {
    let content = r#"import "common_types.sidl" // test

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

    callback ClientCallback {
        fn provide_value() -> int
    }

    #[service annotation]
    interface TestInterface {
        foo: bool
        bar: int+
        baz: Kind+

        event get -> Bin

        imported: ImportedType

        #[rust:name=do_it,js:name=DoIt]
        fn doIt(what: binary?, which: Kind) -> MyType

        fn get_json() -> json

        fn use_callback(callback: ClientCallback) -> bool
    }

    service TestService: TestInterface
    "#;

    let ast = Ast::parse_str("test", content, None).unwrap();

    assert_eq!(ast.callbacks.len(), 1);

    assert_eq!(ast.interfaces.len(), 3);
    let mut member_count = 0;
    let mut event_count = 0;
    let mut method_count = 0;
    for interface in &ast.interfaces {
        member_count += interface.1.members.len();
        event_count += interface.1.events.len();
        method_count += interface.1.methods.len();
    }
    assert_eq!(member_count, 9);
    assert_eq!(event_count, 1);
    assert_eq!(method_count, 3);
    assert_eq!(ast.dictionaries.len(), 1);
    assert_eq!(ast.enumerations.len(), 1);
    assert_eq!(ast.services.len(), 1);
    assert_eq!(ast.callbacks.len(), 1);

    let interface = ast.interfaces.get("TestInterface").unwrap();
    let annot = interface.annotation.as_ref().unwrap();
    assert_eq!(annot.content.len(), 1);
    let method = interface.methods.get("doIt").unwrap();
    let annot = method.annotation.as_ref().unwrap();
    assert_eq!(annot.content.len(), 2);
    assert!(annot.has("rust:name=do_it"));
    assert!(!annot.has("js:name=do_it"));

    // TODO: add a lot more tests
}

#[test]
fn interface_parameter() {
    let content = r#"
    interface MyType {
        value: int
    }

    #[service annotation]
    interface TestInterface {

        fn bogus_method(param: MyType) -> bool
    }

    service TestService: TestInterface
    "#;

    let error = Ast::parse_str("test", content, None).err().unwrap();
    match error {
        AstError::InvalidInterfaceParameter(name) => assert_eq!(name, "param".to_owned()),
        _ => panic!("Unexpected error: {}", error),
    }
}

#[test]
fn annotation_values() {
    let content = r#"

    interface TestInterface {
        fn foo(param: int) -> bool
    }

    #[rust:use=my_traits, rust:use=other_file]
    service TestService: TestInterface
    "#;

    let ast = Ast::parse_str("test", content, None).unwrap();
    assert_eq!(ast.services.len(), 1);
    let service = &ast.services[0];
    let annotations = service.annotation.as_ref().unwrap().get_values("rust:use");
    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0], "my_traits");
    assert_eq!(annotations[1], "other_file");
}

#[test]
fn date_type() {
    let content = r#"

    interface TestInterface {
        fn foo(param: date) -> bool
    }

    service TestService: TestInterface
    "#;

    let ast = Ast::parse_str("test", content, None).unwrap();
    assert_eq!(ast.services.len(), 1);
}
