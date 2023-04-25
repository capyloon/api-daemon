// Shared helpers for the Rust codegen.

use heck::ToUpperCamelCase;
use sidl_parser::ast::{Arity, Ast, ConcreteType, FullConcreteType, Method};
use std::io::Write;

// Returns the Rust type and the type usable for requests and responses.
pub fn rust_type_with_reqresp(full_type: &FullConcreteType) -> (String, String) {
    let type1 = rust_type(full_type);
    let type2 = if let ConcreteType::Interface(_name) = &full_type.typ {
        if full_type.arity == Arity::ZeroOrMore || full_type.arity == Arity::OneOrMore {
            "Vec<u32>".to_owned()
        } else {
            "u32".to_owned()
        }
    } else {
        type1.clone()
    };
    (type1, type2)
}

// Returns true if the matching Rust type needs explicit cloning (ie. doesn't implement Copy)
pub fn needs_clone(full_type: &FullConcreteType) -> bool {
    if full_type.arity != Arity::Unary {
        return true;
    }

    match full_type.typ {
        ConcreteType::Void
        | ConcreteType::Bool
        | ConcreteType::Int
        | ConcreteType::Float
        | ConcreteType::Enumeration(_) => false,
        ConcreteType::Str
        | ConcreteType::Binary
        | ConcreteType::Json
        | ConcreteType::Date
        | ConcreteType::Blob
        | ConcreteType::Url
        | ConcreteType::Dictionary(_)
        | ConcreteType::Interface(_)
        | ConcreteType::Callback(_) => true,
    }
}

pub fn rust_type(full_type: &FullConcreteType) -> String {
    let mut res = String::new();
    match full_type.arity {
        Arity::Optional => res.push_str("Option<"),
        Arity::OneOrMore => res.push_str("Vec<"),
        Arity::ZeroOrMore => res.push_str("Option<Vec<"),
        Arity::Unary => {}
    }

    match full_type.typ {
        ConcreteType::Void => res.push_str("()"),
        ConcreteType::Bool => res.push_str("bool"),
        ConcreteType::Int => res.push_str("i64"),
        ConcreteType::Float => res.push_str("f64"),
        ConcreteType::Str => res.push_str("String"),
        ConcreteType::Binary => res.push_str("Vec<u8>"),
        ConcreteType::Json => res.push_str("JsonValue"),
        ConcreteType::Date => res.push_str("SystemTime"),
        ConcreteType::Blob => res.push_str("Blob"),
        ConcreteType::Url => res.push_str("Url"),
        ConcreteType::Dictionary(ref utype)
        | ConcreteType::Enumeration(ref utype)
        | ConcreteType::Interface(ref utype) => {
            res.push_str(utype);
        }
        ConcreteType::Callback(_) => res.push_str("ObjectRef"),
    }

    match full_type.arity {
        Arity::Optional | Arity::OneOrMore => res.push('>'),
        Arity::ZeroOrMore => res.push_str(">>"),
        Arity::Unary => {}
    }
    res
}

pub fn rust_type_for_param(full_type: &FullConcreteType) -> String {
    let mut res = String::new();
    let is_unary = full_type.arity == Arity::Unary;
    match full_type.arity {
        Arity::Optional => res.push_str("Option<"),
        Arity::OneOrMore => res.push_str("Vec<"),
        Arity::ZeroOrMore => res.push_str("Option<Vec<"),
        Arity::Unary => {}
    }

    match full_type.typ {
        ConcreteType::Void => res.push_str("()"),
        ConcreteType::Bool => res.push_str("bool"),
        ConcreteType::Int => res.push_str("i64"),
        ConcreteType::Float => res.push_str("f64"),
        ConcreteType::Str => res.push_str(if is_unary { "&str" } else { "String" }),
        ConcreteType::Binary => res.push_str("Vec<u8>"),
        ConcreteType::Json => res.push_str("JsonValue"),
        ConcreteType::Date => res.push_str("SystemTime"),
        ConcreteType::Blob => res.push_str("Blob"),
        ConcreteType::Url => res.push_str("Url"),
        ConcreteType::Dictionary(ref utype) | ConcreteType::Enumeration(ref utype) => {
            res.push_str(utype);
        }
        ConcreteType::Callback(_) | ConcreteType::Interface(_) => {
            res.push_str("ObjectRef"); // Interfaces are mapped to the object id.
        }
    }

    match full_type.arity {
        Arity::Optional | Arity::OneOrMore => res.push('>'),
        Arity::ZeroOrMore => res.push_str(">>"),
        Arity::Unary => {}
    }
    res
}

pub fn rust_type_for_proxy_param(full_type: &FullConcreteType) -> String {
    let mut res = String::new();
    match full_type.arity {
        Arity::Optional => res.push_str("Option<"),
        Arity::OneOrMore => res.push_str("Vec<"),
        Arity::ZeroOrMore => res.push_str("Option<Vec<"),
        Arity::Unary => {}
    }

    match full_type.typ {
        ConcreteType::Void => res.push_str("()"),
        ConcreteType::Bool => res.push_str("bool"),
        ConcreteType::Int => res.push_str("i64"),
        ConcreteType::Float => res.push_str("f64"),
        ConcreteType::Str => res.push_str("String"),
        ConcreteType::Binary => res.push_str("Vec<u8>"),
        ConcreteType::Json => res.push_str("JsonValue"),
        ConcreteType::Date => res.push_str("SystemTime"),
        ConcreteType::Blob => res.push_str("Blob"),
        ConcreteType::Url => res.push_str("Url"),
        ConcreteType::Dictionary(ref utype) | ConcreteType::Enumeration(ref utype) => {
            res.push_str(utype);
        }
        ConcreteType::Callback(_) | ConcreteType::Interface(_) => {
            res.push_str("ObjectRef"); // Interfaces are mapped to the object id.
        }
    }

    match full_type.arity {
        Arity::Optional | Arity::OneOrMore => res.push('>'),
        Arity::ZeroOrMore => res.push_str(">>"),
        Arity::Unary => {}
    }
    res
}

pub struct MethodWriter;

impl MethodWriter {
    // Returns Request, ResponseSuccess, ResponseError
    pub fn get_req_resps(method: &Method) -> (String, String, String) {
        // To not create additional structs, requests are tuples matching method parameters.
        let mut req_tuple = "(".to_owned();
        // Process parameters list
        for param in &method.params {
            let (_stype, itype) = rust_type_with_reqresp(&param.typ);
            req_tuple.push_str(&format!("{},", itype));
        }
        req_tuple.push(')');
        let (_stype, mut stype) = rust_type_with_reqresp(&method.returns.success);
        let (_etype, mut etype) = rust_type_with_reqresp(&method.returns.error);

        // Check types to not generate empty enum variants.
        if req_tuple == "()" {
            req_tuple = "".into();
        }

        if stype == "()" {
            stype = "".into();
        } else {
            stype = format!("({})", stype);
        }

        if etype == "()" {
            etype = "".into();
        } else {
            etype = format!("({})", etype);
        }

        let camel_name = method.name.to_upper_camel_case();
        (
            format!("{}{}", camel_name, req_tuple),
            format!("{}Success{}", camel_name, stype),
            format!("{}Error{}", camel_name, etype),
        )
    }

    pub fn declare<'a, W: Write>(
        interface_name: &str,
        method: &'a Method,
        sink: &'a mut W,
    ) -> Result<(), ::std::io::Error> {
        write!(
            sink,
            "    fn {}(&mut self, responder: {}{}Responder, ",
            method.name,
            interface_name,
            method.name.to_upper_camel_case()
        )?;
        // Write parameters list
        for param in &method.params {
            let stype = rust_type_for_param(&param.typ);
            write!(sink, "{}: {},", param.name, stype)?;
        }

        writeln!(sink, ")")?;

        Ok(())
    }
}

pub fn get_all_reqs_resps(ast: &Ast) -> (Vec<String>, Vec<String>) {
    // Get all the possible requests and responses.
    let mut reqs = vec![];
    let mut resps = vec![];

    // Helper closure to populate the requests and responses.
    let mut add_to_reqs_resps = |is_req, value: String| {
        if is_req {
            reqs.push(value);
        } else {
            resps.push(value);
        }
    };

    for interface in ast.interfaces.values() {
        for method in interface.methods.values() {
            let (req, success_resp, error_resp) = MethodWriter::get_req_resps(method);

            add_to_reqs_resps(true, format!("{}{}", interface.name, req));
            add_to_reqs_resps(false, format!("{}{}", interface.name, success_resp));
            add_to_reqs_resps(false, format!("{}{}", interface.name, error_resp));
        }

        // Members
        for member in &interface.members {
            // member is a (String = name, (Option<Annotation>, ConcreteType, Arity))
            let ctype = member.1;
            let (mtype, rtype) = rust_type_with_reqresp(&ctype.typ);

            // Getter: get_xxx -> mtype
            // These are infallible.
            let member_name = member.0.to_upper_camel_case();
            add_to_reqs_resps(true, format!("{}Get{}", interface.name, member_name));
            add_to_reqs_resps(
                false,
                format!("{}Get{}({})", interface.name, member_name, rtype),
            );

            // Setter: set_xxx(mtype)
            add_to_reqs_resps(
                true,
                format!("{}Set{}({})", interface.name, member_name, mtype),
            );
        }

        // Events
        for event in interface.events.values() {
            let ctype = &event.returns;
            let (_rtype, itype) = rust_type_with_reqresp(ctype);
            let event_name = event.name.to_upper_camel_case();
            if ctype.typ != ConcreteType::Void {
                add_to_reqs_resps(
                    false,
                    format!("{}{}Event({})", interface.name, event_name, itype),
                );
            } else {
                add_to_reqs_resps(false, format!("{}{}Event", interface.name, event_name));
            }
        }
    }

    for callback in ast.callbacks.values() {
        for method in callback.methods.values() {
            let (req, success_resp, error_resp) = MethodWriter::get_req_resps(method);

            add_to_reqs_resps(false, format!("{}{}", callback.name, req));
            add_to_reqs_resps(true, format!("{}{}", callback.name, success_resp));
            add_to_reqs_resps(true, format!("{}{}", callback.name, error_resp));
        }
    }

    (reqs, resps)
}

pub fn get_fingerprint(ast: &Ast) -> String {
    use crate::ast_utils::*;
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    let ast = normalize_rust_case(ast, &RustCaseNormalizer);

    let (reqs, resps) = get_all_reqs_resps(&ast);
    let mut hasher = Sha256::new();

    // Hash all the requests and response enumerations.
    for req in reqs {
        hasher.update(req.as_bytes());
    }

    for resp in resps {
        hasher.update(resp.as_bytes());
    }

    // Hash each dictionary shape: member names and types.
    for dictionary in ast.dictionaries.values() {
        for member in &dictionary.members {
            let rtype = rust_type(&member.typ);
            hasher.update(member.name.as_bytes());
            hasher.update(rtype.as_bytes());
        }
    }

    // Hash each event shape: name and type.
    for interface in ast.interfaces.values() {
        for event in interface.events.values() {
            let rtype = rust_type(&event.returns);
            hasher.update(event.name.as_bytes());
            hasher.update(rtype.as_bytes());
        }
    }

    let result = hasher.finalize();
    let mut hex = String::new();
    for byte in result {
        write!(&mut hex, "{:x}", byte).expect("Unable to write");
    }
    hex
}
