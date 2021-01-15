/// Javascript code generator.
use crate::ast_utils::*;
use heck::CamelCase;
use sidl_parser::ast::{
    Arity, Ast, Callback, ConcreteType, Enumeration, FullConcreteType, Interface, Method, Service,
};
use std::io::Write;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] ::std::io::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

// A single type item.
#[derive(Debug)]
pub struct TypeItem {
    name: Option<String>, // Can be None for setters or response types descriptions.
    typ: FullConcreteType,
}

impl TypeItem {
    fn from(name: Option<String>, full_type: &FullConcreteType) -> Self {
        TypeItem {
            name,
            typ: full_type.clone(),
        }
    }
}

// A request with a name and types parameters.
#[derive(Debug)]
pub struct TypedRequest {
    name: String,
    types: Vec<TypeItem>,
}

impl TypedRequest {
    fn new(name: &str) -> Self {
        TypedRequest {
            name: name.into(),
            types: vec![],
        }
    }
}

// A response with a success and error type.
#[derive(Debug)]
pub struct TypedResponse {
    name: String,
    success: Option<TypeItem>,
    error: Option<TypeItem>,
}

impl TypedResponse {
    fn new(name: &str) -> Self {
        TypedResponse {
            name: name.into(),
            success: None,
            error: None,
        }
    }
}

#[derive(Debug)]
pub enum TypedMessage {
    Request(TypedRequest),
    Response(TypedResponse),
}

struct MethodWriter;

impl MethodWriter {
    // Returns Request, Response as TypedMessage
    fn get_types(method: &Method) -> Result<(TypedMessage, TypedMessage)> {
        let camel_name = method.name.to_camel_case();

        // Request with all the parameters
        let mut typed_req = TypedRequest::new(&camel_name);
        for param in &method.params {
            typed_req
                .types
                .push(TypeItem::from(Some(param.name.clone()), &param.typ));
        }

        // Success response
        let mut typed_resp = TypedResponse::new(&camel_name);
        typed_resp.success = Some(TypeItem::from(None, &method.returns.success));

        // Error response
        typed_resp.error = Some(TypeItem::from(None, &method.returns.error));
        Ok((
            TypedMessage::Request(typed_req),
            TypedMessage::Response(typed_resp),
        ))
    }

    // Returns Request, Response as TypedMessage and writes the method signature
    fn declare<'a, W: Write>(
        method: &'a Method,
        sink: &'a mut W,
    ) -> Result<(TypedMessage, TypedMessage)> {
        write!(sink, "{}(", method.name)?;
        // Write parameters list
        let mut first = true;
        for param in &method.params {
            if !first {
                write!(sink, ",")?;
            }
            first = false;
            write!(sink, "{}", param.name)?;
        }
        write!(sink, ")")?;

        MethodWriter::get_types(method)
    }
}

pub struct Codegen {
    ast: Ast,
    fingerprint: String,
    unique_id: u32,
}

fn js_type(typ: &ConcreteType) -> String {
    let mut res = String::new();
    match *typ {
        ConcreteType::Void => res.push_str("void"),
        ConcreteType::Bool => res.push_str("bool"),
        ConcreteType::Int => res.push_str("i64"),
        ConcreteType::Float => res.push_str("f64"),
        ConcreteType::Str => res.push_str("string"),
        ConcreteType::Binary => res.push_str("u8_array"),
        ConcreteType::Json => res.push_str("json"),
        ConcreteType::Date => res.push_str("date"),
        _ => unimplemented!("No js type for this concrete type: {:?}", typ),
    }
    res
}

impl Codegen {
    // path is the current "root" seeding the JS property.
    fn write_encoder_for_item<'a, W: Write>(
        &mut self,
        path: &str,
        item: &'a TypeItem,
        sink: &'a mut W,
    ) -> Result<()> {
        // The full name of the JS property.
        let mut full_path = match &item.name {
            Some(val) => format!("{}.{}", path, val),
            None => path.into(),
        };

        let postfix = match item.typ.arity {
            Arity::Unary => "",
            Arity::Optional => {
                // For optional arity, we enclose in |if (!!full_path) { ... }|
                writeln!(sink, "result = result.bool({} !== undefined);", full_path)?;
                writeln!(sink, "if ({} !== undefined) {{", full_path)?;

                "}\n"
            }
            Arity::ZeroOrMore => {
                // TODO: Check that the full_path is an array.
                writeln!(
                    sink,
                    "result = result.bool({} !== undefined && {}.length > 0);",
                    full_path, full_path
                )?;
                writeln!(
                    sink,
                    "if ({} !== undefined && {}.length > 0) {{",
                    full_path, full_path
                )?;
                writeln!(sink, "result = result.u64({}.length);", full_path)?;
                writeln!(sink, "{}.forEach(item => {{", full_path)?;

                full_path = "item".into();

                "});\n }\n"
            }
            Arity::OneOrMore => {
                // Multiple occurences are turned into arrays that are iterated over.
                // First write the array length.
                // TODO: Check that the full_path is an array.
                writeln!(sink, "result = result.u64({}.length);", full_path)?;
                writeln!(sink, "{}.forEach(item => {{", full_path)?;

                full_path = "item".into();

                "});\n"
            }
        };

        match item.typ.typ {
            ConcreteType::Dictionary(ref utype) => {
                // For dictionaries, we serialize each field in sequence.
                let dict = self.ast.dictionaries.get(utype).unwrap().clone();
                // If the full path is just "item" that means we are in an array
                // loop already, so no need to create another dictionary holder.

                sink.write_all(b"{\n")?;

                let dict_name = format!("dict{}", self.unique_id);
                self.unique_id += 1;
                if full_path != "item" {
                    writeln!(sink, "let {} = {};", dict_name, full_path)?;
                    full_path = dict_name;
                }

                // Create a new item type for each property.
                for member in &dict.members {
                    self.write_encoder_for_item(
                        &full_path,
                        &TypeItem::from(Some(member.name.clone()), &member.typ),
                        sink,
                    )?;
                }

                sink.write_all(b"}\n")?;
            }
            ConcreteType::Enumeration(_) => {
                // Enumerations are just variant tag values.
                writeln!(sink, "result = result.enum_tag({});", full_path)?;
            }
            ConcreteType::Callback(_) | ConcreteType::Interface(_) => {
                // For interfaces, we output the object id.
                writeln!(sink, "result = result.u32({}.id);", full_path)?;
            }
            _ => {
                writeln!(
                    sink,
                    "result = result.{}({});",
                    js_type(&item.typ.typ),
                    full_path
                )?;
            }
        }

        sink.write_all(postfix.as_bytes())?;

        Ok(())
    }

    fn write_decoder_for_item<'a, W: Write>(
        &self,
        item: &'a TypeItem,
        sink: &'a mut W,
        var_name: &str,
        nesting_level: u32,
    ) -> Result<()> {
        let name = item.name.clone().unwrap_or_else(|| "<no_name>".into());

        let is_nested = nesting_level != 0 && matches!(item.typ.typ, ConcreteType::Dictionary(_));

        let full_name = match &item.name {
            Some(name) => {
                if is_nested {
                    format!("{}.{}", var_name, name)
                } else {
                    var_name.to_owned()
                }
            }
            None => var_name.to_owned(),
        };

        writeln!(sink, "// decoding {}", name)?;

        // Manage the item arity.
        match item.typ.arity {
            Arity::Unary => {
                self.write_decoder_for_single_item(
                    item,
                    sink,
                    false, /* is_array */
                    &full_name,
                    nesting_level + 1,
                )?
            }
            Arity::Optional => {
                writeln!(sink, "if (decoder.bool()) {{")?;
                self.write_decoder_for_single_item(
                    item,
                    sink,
                    false, /* is_array */
                    &full_name,
                    nesting_level + 1,
                )?;
                writeln!(sink, "}}")?;
            }
            Arity::ZeroOrMore => {
                writeln!(sink, "if (decoder.bool()) {{")?;
                writeln!(sink, "let count = decoder.u64();")?;
                // writeln!(sink, "console.log(`zero or more: ${{count}} items`);")?;
                writeln!(
                    sink,
                    "{}{} = [];",
                    var_name,
                    item.name
                        .clone()
                        .map(|e| format!(".{}", e))
                        .unwrap_or_else(|| "".into()),
                )?;
                writeln!(sink, "for (let i = 0; i < count; i++) {{")?;
                self.write_decoder_for_single_item(
                    item,
                    sink,
                    true, /* is_array */
                    var_name,
                    nesting_level + 1,
                )?;
                writeln!(sink, "}}")?;
                writeln!(sink, "}}")?;
            }
            Arity::OneOrMore => {
                writeln!(sink, "{{")?;
                writeln!(sink, "let count = decoder.u64();")?;
                // writeln!(sink, "console.log(`one or more: ${{count}} items`);")?;
                writeln!(
                    sink,
                    "{}{} = [];",
                    var_name,
                    item.name
                        .clone()
                        .map(|e| format!(".{}", e))
                        .unwrap_or_else(|| "".into()),
                )?;
                writeln!(sink, "for (let i = 0; i < count; i++) {{")?;
                self.write_decoder_for_single_item(
                    item,
                    sink,
                    true, /* is_array */
                    var_name,
                    nesting_level + 1,
                )?;
                writeln!(sink, "}}")?;
                writeln!(sink, "}} // let count = ... scope")?;
            }
        }

        Ok(())
    }

    fn write_decoder_for_single_item<'a, W: Write>(
        &self,
        item: &'a TypeItem,
        sink: &'a mut W,
        is_array: bool,
        var_name: &str,
        nesting_level: u32,
    ) -> Result<()> {
        let name = item.name.clone().unwrap_or_else(|| "<no_name>".into());

        match item.typ.typ {
            ConcreteType::Dictionary(ref utype) => {
                if is_array {
                    // need to create a local object inside the for loop.
                    let local_var = format!("_{}", var_name);
                    writeln!(sink, "let {} = {{}};", local_var)?;
                    // For dictionaries, we read each field in sequence.
                    let dict = self.ast.dictionaries.get(utype).unwrap();
                    for member in &dict.members {
                        self.write_decoder_for_item(
                            &TypeItem::from(Some(member.name.clone()), &member.typ),
                            sink,
                            &local_var,
                            nesting_level + 1,
                        )?;
                    }
                    if item.name.is_some() {
                        writeln!(sink, "{}.{}.push({});", var_name, name, local_var)?;
                    } else {
                        writeln!(sink, "{}.push({});", var_name, local_var)?;
                    }
                } else {
                    // For dictionaries, we read each field in sequence.
                    let dict = self.ast.dictionaries.get(utype).unwrap();
                    writeln!(sink, "{} = {{}};", var_name)?;
                    // Create a new item type for each property.
                    for member in &dict.members {
                        self.write_decoder_for_item(
                            &TypeItem::from(Some(member.name.clone()), &member.typ),
                            sink,
                            var_name,
                            nesting_level + 1,
                        )?;
                    }
                }
            }
            ConcreteType::Enumeration(_) => {
                // Enumerations are just variant tag values.

                if is_array {
                    if item.name.is_some() {
                        writeln!(sink, "{}.{}.push(decoder.enum_tag());", var_name, name)?;
                    } else {
                        writeln!(sink, "{}.push(decoder.enum_tag());", var_name)?;
                    }
                } else if item.name.is_some() {
                    writeln!(sink, "{}.{} = decoder.enum_tag();", var_name, name)?;
                } else {
                    writeln!(sink, "{} = decoder.enum_tag();", var_name)?;
                }
            }
            ConcreteType::Interface(ref name) => {
                if is_array {
                    write!(
                        sink,
                        "{}.push(new {}Session(decoder.u32(), service_id, session));",
                        var_name,
                        name.to_camel_case()
                    )?;
                } else {
                    write!(
                        sink,
                        "{} = new {}Session(decoder.u32(), service_id, session);",
                        var_name,
                        name.to_camel_case()
                    )?;
                }
            }
            _ => {
                if is_array {
                    writeln!(
                        sink,
                        "{}{}.push(decoder.{}());",
                        var_name,
                        item.name
                            .clone()
                            .map(|e| format!(".{}", e))
                            .unwrap_or_else(|| "".into()),
                        js_type(&item.typ.typ),
                    )?;
                } else {
                    writeln!(
                        sink,
                        "{}{} = decoder.{}();",
                        var_name,
                        item.name
                            .clone()
                            .map(|e| format!(".{}", e))
                            .unwrap_or_else(|| "".into()),
                        js_type(&item.typ.typ),
                    )?;
                }
            }
        }

        Ok(())
    }

    // Returns the updated index for requests and responses.
    pub fn generate_messages_for_interface<'a, W: Write>(
        &mut self,
        name: &str,
        messages: &[TypedMessage],
        req_index: usize,
        resp_index: usize,
        sink: &'a mut W,
    ) -> Result<(usize, usize)> {
        writeln!(sink, "const {}Messages = {{", name)?;

        // For each request, provide an encoder.
        let mut req_index = req_index;
        let mut resp_index = resp_index;
        for message in messages.iter() {
            match message {
                TypedMessage::Request(req) => {
                    writeln!(sink, "{}Request: {{", req.name)?;
                    writeln!(sink, "encode: (data) => {{")?;
                    writeln!(sink, "let encoder = new Encoder();")?;
                    writeln!(sink, "let result = encoder.enum_tag({});", req_index)?;
                    req_index += 1;
                    // generate the encoder for the request payload.
                    for ptype in &req.types {
                        self.write_encoder_for_item("data", &ptype, sink)?;
                    }

                    writeln!(sink, "return result.value();")?;
                    writeln!(sink, "}}")?;
                    writeln!(sink, "}},")?;
                }
                TypedMessage::Response(resp) => {
                    // For each response, provide a decoder.
                    writeln!(sink, "{}Response: {{", resp.name)?;
                    writeln!(sink, "decode: (buffer , service_id, session) => {{")?;
                    writeln!(sink, "let decoder = new Decoder(buffer);")?;
                    writeln!(sink, "let variant = decoder.enum_tag();")?;

                    if let Some(item) = &resp.success {
                        // Methods and getter responses.
                        writeln!(sink, "if (variant == {}) {{", resp_index)?;
                        writeln!(sink, "// Success")?;

                        writeln!(sink, "let result = null;")?;
                        self.write_decoder_for_item(&item, sink, "result", 0)?;
                        writeln!(sink, "return {{ success: result }}")?;
                        writeln!(sink, "}}")?;

                        resp_index += 1;
                    } else {
                        panic!("Unexpected empty success response for {}", resp.name);
                    }
                    if let Some(item) = &resp.error {
                        // Methods error responses.
                        writeln!(sink, "else if (variant == {}) {{", resp_index)?;
                        writeln!(sink, "// Error")?;

                        writeln!(sink, "let result = null;")?;
                        self.write_decoder_for_item(&item, sink, "result", 0)?;
                        writeln!(sink, "return {{ error: result }}")?;
                        writeln!(sink, "}}")?;
                        resp_index += 1;
                    }

                    writeln!(sink, "else {{")?;
                    writeln!(
                        sink,
                        "console.error(`{}Response::decode: Unexpected variant ${{variant}}`);",
                        resp.name
                    )?;
                    writeln!(sink, "return null;")?;
                    writeln!(sink, "}}")?;

                    writeln!(sink, "}}")?;
                    writeln!(sink, "}},")?;
                }
            }
        }

        writeln!(sink, "}}\n")?;

        Ok((req_index, resp_index))
    }

    // Interfaces are mapped to a class extending SessionObject.
    // Returns the updated index for requests and responses.
    pub fn generate_interface<'a, W: Write>(
        &mut self,
        interface: &Interface,
        req_index: usize,
        resp_index: usize,
        sink: &'a mut W,
    ) -> Result<(usize, usize)> {
        let mut messages: Vec<TypedMessage> = vec![];

        writeln!(
            sink,
            "class {}Session extends SessionObject {{",
            interface.name
        )?;
        writeln!(sink, "constructor(object_id, service_id, session) {{")?;
        writeln!(
            sink,
            "super(object_id , session, service_id, {}Messages);",
            interface.name
        )?;
        writeln!(sink, "this.service_id = service_id;")?;

        if !interface.events.is_empty() {
            writeln!(sink, "session.track_events(service_id, object_id, this);")?;
        }

        writeln!(sink, "}}")?;

        let mut resp_count = resp_index;

        // Methods
        for method in interface.methods.values() {
            let (req, resp) = MethodWriter::declare(&method, sink)?;
            writeln!(sink, "{{")?;

            writeln!(
                sink,
                "return this.call_method(\"{}\", {{",
                method.name.to_camel_case()
            )?;
            let mut first = true;
            for param in &method.params {
                if !first {
                    write!(sink, ",")?;
                }
                first = false;
                write!(sink, "{}: {}", param.name, param.name)?;
            }
            writeln!(sink, "}});")?;
            writeln!(sink, "}}")?;

            messages.push(req);
            messages.push(resp);
            resp_count += 2; // Adding 2 because there are success and error responses.
        }

        // Members
        for member in &interface.members {
            let member = member.1;
            let camel_name = member.name.to_camel_case();

            // Getter: get xxx()
            // These are infallible.
            writeln!(sink, "get {}() {{", member.name)?;
            writeln!(sink, "return this.call_method(\"Get{}\");", camel_name)?;
            writeln!(sink, "}}")?;

            let typed_getter = TypedRequest::new(&format!("Get{}", camel_name));
            // No parameter sent for the request.
            messages.push(TypedMessage::Request(typed_getter));

            // Using the member type for the getter response.
            let mut typed_getter = TypedResponse::new(&format!("Get{}", camel_name));
            typed_getter.success = Some(TypeItem::from(None, &member.typ));
            messages.push(TypedMessage::Response(typed_getter));
            resp_count += 1;

            // Setter: set xxx(value)
            // These are infallible.
            writeln!(sink, "set {}(value) {{", member.name)?;
            writeln!(
                sink,
                "return this.call_method_oneway(\"Set{}\", {{ value }});",
                camel_name
            )?;
            writeln!(sink, "}}")?;

            let mut typed_setter = TypedRequest::new(&format!("Set{}", camel_name));
            // Using the member type for the setter request.
            typed_setter
                .types
                .push(TypeItem::from(Some("value".into()), &member.typ));
            messages.push(TypedMessage::Request(typed_setter));
        }

        // Generate the event dispatching code if this interface has events.
        if !interface.events.is_empty() {
            writeln!(sink, "on_event(event) {{")?;
            writeln!(
                sink,
                "// console.log(`{}Session message: ${{event}}`);",
                interface.name
            )?;
            writeln!(sink, "let decoder = new Decoder(event);")?;
            writeln!(sink, "let variant = decoder.enum_tag();")?;
            for (i, event) in interface.events.iter().enumerate() {
                let event = event.1;
                writeln!(sink, "// Event #{}: {}", i + resp_count, event.name)?;
                if i != 0 {
                    write!(sink, "else ")?;
                }
                writeln!(sink, "if (variant == {}) {{", i + resp_count)?;
                writeln!(sink, "let result = null;")?;
                let rtype = TypeItem::from(None, &event.returns);
                self.write_decoder_for_item(&rtype, sink, "result", 0)?;

                writeln!(sink, "this.dispatchEvent({}, result);", i)?;
                writeln!(sink, "}}")?;
            }
            writeln!(
                sink,
                "else {{\n console.error(`Unable to process variant #${{variant}}`); }}"
            )?;
            writeln!(sink, "}}")?; // End on_event(event) ...
        }

        writeln!(sink, "}}\n")?; // End class {}Session ...

        // Create constants for the events name.
        for (i, event) in interface.events.iter().enumerate() {
            writeln!(
                sink,
                "{}Session.prototype.{}_EVENT = {};",
                interface.name, event.1.name, i
            )?;
        }

        // Debug
        write!(sink, "/*\n\n")?;
        write!(sink, "Messages: {:?}\n\n", messages)?;
        write!(sink, "*/\n\n")?;

        let res = self.generate_messages_for_interface(
            &interface.name,
            &messages,
            req_index,
            resp_index,
            sink,
        )?;

        // Increment the response index by the number of events since they are part
        // of the response set.
        Ok((res.0, res.1 + interface.events.len()))
    }

    // Callbacks are mapped to a class extending SessionObject.
    // Returns the updated index for requests and responses.
    pub fn generate_callback<'a, W: Write>(
        &mut self,
        callback: &Callback,
        req_index: usize,
        resp_index: usize,
        sink: &'a mut W,
    ) -> Result<(usize, usize)> {
        writeln!(
            sink,
            "export class {}Base extends SessionObject {{",
            callback.name
        )?;
        writeln!(sink, "constructor(service_id, session) {{")?;
        writeln!(sink, "super(session.next_id , session, service_id, null);")?;
        writeln!(sink, "session.track(this);")?;
        writeln!(sink, "this.service_id = service_id;")?;
        writeln!(sink, "}}")?;

        // Callback base classes don't implement methods themselves, since
        // they are expected to be implemented by the user.
        // The base class implements a dispatcher to call these methods
        // when a message is received and manages the response sending when
        // the method promise resolves or rejects.

        // The message dispatcher
        writeln!(
            sink,
            r#"on_message(message) {{
            // console.log(`Message for {} ${{this.display()}}: %o`, message);"#,
            callback.name
        )?;

        writeln!(
            sink,
            r#"let decoder = new Decoder(message.content);
        let variant = decoder.enum_tag();
        // console.log(`Starting at index {}`);
        // console.log(`we got variant ${{variant}}`);
        // Dispatch based on message.content which is the real payload.
        "#,
            resp_index
        )?;

        let mut resp_index = resp_index;
        let mut req_index = req_index;

        writeln!(sink, "switch (variant) {{")?;
        for method in callback.methods.values() {
            let (req, resp) = MethodWriter::get_types(&method)?;
            let req = match req {
                TypedMessage::Request(req) => req,
                _ => panic!("Expected TypedMessage::Request!"),
            };
            let resp = match resp {
                TypedMessage::Response(resp) => resp,
                _ => panic!("Expected TypedMessage::Response!"),
            };

            writeln!(sink, "case {}: {{", resp_index)?;
            writeln!(
                sink,
                "// console.log(`Extracting parameters for {}(...)`);",
                method.name
            )?;
            writeln!(
                sink,
                "if (this.{} && this.{} instanceof Function) {{",
                method.name, method.name
            )?;

            // Decode the parameters, storing them in a temp struct.
            writeln!(sink, "let result = {{}};")?;
            for item in &req.types {
                self.write_decoder_for_item(&item, sink, "result", 0)?;
            }

            writeln!(sink, "let output = this.{}(", method.name)?;
            let mut first = true;
            for item in &req.types {
                if !first {
                    write!(sink, ",")?;
                }
                first = false;
                match item.typ.typ {
                    ConcreteType::Dictionary(_)
                    | ConcreteType::Interface(_)
                    | ConcreteType::Enumeration(_) => writeln!(sink, "result")?,
                    _ => writeln!(sink, "result.{}", item.name.clone().unwrap())?,
                }
            }
            writeln!(sink, ");")?;

            // output is a Promise, decode the resolved or rejected value and
            // send it back.
            writeln!(sink, "output.then(")?;

            // Success case.
            writeln!(
                sink,
                "success => {{ // console.log(`{}.{} success: ${{success}}`);",
                callback.name, method.name
            )?;
            writeln!(sink, "let encoder = new Encoder();")?;
            writeln!(sink, "let result = encoder.enum_tag({});", req_index)?;
            req_index += 1;
            // generate the encoder for the success payload.
            self.write_encoder_for_item("success", &resp.success.unwrap(), sink)?;
            // Send the message.
            writeln!(sink, "message.content = result.value();")?;
            writeln!(sink, "this.send_callback_message(message);")?;
            writeln!(sink, "}},")?;

            // Error case.
            writeln!(
                sink,
                "error => {{ // console.error(`{}.{} error: ${{error}}`);",
                callback.name, method.name
            )?;
            writeln!(sink, "let encoder = new Encoder();")?;
            writeln!(sink, "let result = encoder.enum_tag({});", req_index)?;
            req_index += 1;
            // generate the encoder for the error payload.
            self.write_encoder_for_item("error", &resp.error.unwrap(), sink)?;
            // Send the message.
            writeln!(sink, "message.content = result.value();")?;
            writeln!(sink, "this.send_callback_message(message);")?;
            writeln!(sink, "}}")?;

            writeln!(sink, ");")?; // End of output.then(

            writeln!(sink, "}}")?; // End if (this...)

            writeln!(sink, "break; }}")?; // End case $variant...

            resp_index += 1;
            // messages.push(resp);
            // messages.push(req);
        }
        writeln!(
            sink,
            "default: console.error(`Unexpected variant: ${{variant}}`);"
        )?;
        writeln!(sink, "}}")?; // End switch (variant)

        writeln!(sink, "}}")?; // End on_message(..)

        writeln!(sink, "}}\n")?; // End class {}Base ...

        Ok((req_index, resp_index))
    }

    // Services are mapped to trait that implement both their defining interface and the
    // core common one.
    pub fn generate_service<'a, W: Write>(&self, service: &Service, sink: &'a mut W) -> Result<()> {
        // Write the main entry point used to instanciate the service.
        write!(
            sink,
            r#"export const {} = {{
            get: (session) => {{
                return Services.get("{}", "{}", session).then((service_id) => {{
                    // object_id is always 0 for the service itself.
                    return new {}Session(0, service_id, session);
                }});
            }},
        }};"#,
            service.name,
            service.name,
            self.fingerprint,
            service.interface
        )?;
        writeln!(sink)?;

        Ok(())
    }

    fn generate_enumeration<'a, W: Write>(
        &self,
        enumeration: &Enumeration,
        sink: &'a mut W,
    ) -> Result<()> {
        writeln!(sink, "export const {} = {{", enumeration.name)?;
        for member in &enumeration.members {
            writeln!(sink, "{}:{},", member.name, member.order)?;
        }
        writeln!(sink, "}}\n")?;
        Ok(())
    }

    pub fn generate<W: Write>(&mut self, sink: &mut W) -> Result<()> {
        sink.write_all(
            b"// This file is generated. Do not edit.
              // @generated\n\n
              import Services from '../../../../common/client/src/services';
              import SessionObject from '../../../../common/client/src/sessionobject';
              import {Encoder, Decoder} from '../../../../common/client/src/bincode.js';\n\n",
        )?;

        // Generate enums representations.
        for item in &self.ast.enumerations {
            self.generate_enumeration(&item.1, sink)?;
        }

        let mut req_index = 0;
        let mut resp_index = 0;
        // Generate session objects for each interface.
        for interface in self.ast.interfaces.clone().values() {
            // We need to keep track of the current request and response indexes because
            // they need to match the variant index of the Rust side.
            let (new_req, new_resp) =
                self.generate_interface(&interface, req_index, resp_index, sink)?;
            req_index = new_req;
            resp_index = new_resp;
        }

        // Generate session objects for each callback.
        for callback in self.ast.callbacks.clone().values() {
            // We need to keep track of the current request and response indexes because
            // they need to match the variant index of the Rust side.
            let (new_req, new_resp) =
                self.generate_callback(&callback, req_index, resp_index, sink)?;
            req_index = new_req;
            resp_index = new_resp;
        }

        // Generate service wrapper.
        for service in &self.ast.services {
            self.generate_service(&service, sink)?;
        }

        Ok(())
    }

    pub fn new(ast: Ast) -> Codegen {
        let fingerprint = crate::helpers::get_fingerprint(&ast);
        Codegen {
            ast: normalize_rust_case(&ast, &JavascriptCaseNormalizer),
            fingerprint,
            unique_id: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Codegen;

    #[test]
    fn test_generate_rust() {
        const CONTENT: &'static str = r#"
        interface Kind {
            event test1
            event test2
            event test3
            data: binary+
        }

        callback SomeObject {
            fn call_me(maybe: int) -> str
        }

        interface MyType {
            event test1
        }

        interface Nothing {
        }

        #[service annotation]
        interface TestServiceInterface {

            #[rust_name=do_it]
            fn doIt(what: binary?, which: SomeObject) -> Kind
        }

        service TestService: TestServiceInterface
        "#;

        use sidl_parser::ast::Ast;

        let ast = Ast::parse_str("test", CONTENT, None).unwrap();
        let mut generator = Codegen::new(ast);

        generator
            .generate(&mut ::std::io::stdout())
            .expect("Failed to generate Javascript code!");
    }
}
