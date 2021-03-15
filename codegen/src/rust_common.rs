use crate::ast_utils::*;
use crate::helpers::*;
use heck::CamelCase;
use log::error;
use sidl_parser::ast::{
    Ast, Callback, ConcreteType, Dictionary, Enumeration, FullConcreteType, Interface, Service,
};
use std::io::Write;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] ::std::io::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

// Returns the Rust type for resolve/reject calls.
// If it's an interface, expect a Tracked object.
// In that case, we check if it's shared and has a custom trait.
fn rust_type_for_resolve_reject(ast: &Ast, full_type: &FullConcreteType) -> String {
    if let ConcreteType::Interface(name) = &full_type.typ {
        let info = TrackedInterfaceInfo::by_name(ast, name);
        info.type_representation()
    } else {
        rust_type(full_type)
    }
}

pub trait SanitizedKeyword {
   fn sanitized_keyword(&self) -> String;
}

impl SanitizedKeyword for str {
    fn sanitized_keyword(&self) -> String {
        static KEYWORDS: [&str; 39] = [
            "as",
            "async",
            "await",
            "break",
            "const",
            "continue",
            "crate",
            "dyn",
            "else",
            "enum",
            "extern",
            "false",
            "fn",
            "for",
            "if",
            "impl",
            "in",
            "let",
            "loop",
            "match",
            "mod",
            "move",
            "mut",
            "pub",
            "ref",
            "return",
            "Self",
            "self",
            "static",
            "struct",
            "super",
            "trait",
            "true",
            "type",
            "union",
            "unsafe",
            "use",
            "where",
            "while",
        ];

        if KEYWORDS.iter().any(|v| v == &self) {
            format!("r#{}", self)
        } else {
            self.to_string()
        }
    }
}

pub struct EnumWriter;

impl EnumWriter {
    pub fn declare<'a, W: Write>(enumeration: &'a Enumeration, sink: &'a mut W) -> Result<()> {
        // whether we need to derive more modules.
        let mut more_derived: String = Default::default();
        if let Some(annotation) = &enumeration.annotation {
            for derive in annotation.get_values("derive") {
                more_derived += ", ";
                more_derived += derive;
            }
        }

        // TODO: figure out if we have to serialize and deserialize based on usage.
        writeln!(
            sink,
            "#[derive(Clone, PartialEq, Deserialize, Serialize, Debug{})]",
            more_derived
        )?;
        writeln!(sink, "pub enum {} {{", enumeration.name)?;

        for member in &enumeration.members {
            writeln!(sink, "    {}, // #{}", member.name.sanitized_keyword(), member.order)?;
        }
        writeln!(sink, "}}")?;
        writeln!(sink, "impl Copy for {} {{}}\n", enumeration.name)?;

        Ok(())
    }
}

pub struct DictWriter;

impl DictWriter {
    pub fn declare<'a, W: Write>(dict: &'a Dictionary, sink: &'a mut W) -> Result<()> {
        // TODO: figure out if we have to serialize and deserialize based on usage.
        writeln!(sink, "#[derive(Clone, Deserialize, Serialize, Debug)]")?;
        writeln!(sink, "pub struct {} {{", dict.name)?;

        for member in &dict.members {
            writeln!(sink, "    pub {}: {},", member.name.sanitized_keyword(), rust_type(&member.typ))?;
        }
        writeln!(sink, "}}\n")?;

        Ok(())
    }
}

pub struct Codegen {
    ast: Ast,
    toplevel_name: String,
    fingerprint: String,
}

impl Codegen {
    pub fn new(ast: Ast, name: &str) -> Self {
        let fingerprint = get_fingerprint(&ast);
        Self {
            ast: normalize_rust_case(&ast, &RustCaseNormalizer),
            toplevel_name: name.into(),
            fingerprint,
        }
    }

    // Interfaces are mapped to traits.
    pub fn generate_interface<'a, W: Write>(
        &self,
        interface: &Interface,
        sink: &'a mut W,
    ) -> Result<()> {
        writeln!(sink, "// Annotation is {:?}", interface.annotation)?;
        let traits = match &interface.annotation {
            Some(annotation) => {
                let mut res = vec![];
                if !annotation.has("rust:not-tracked") {
                    res.push("SimpleObjectTracker");
                }
                res
            }
            // No annotation at all, default to tracking.
            None => vec!["SimpleObjectTracker"],
        };

        let other_traits = if traits.is_empty() {
            "".to_owned()
        } else {
            format!(": {}", traits.join("+"))
        };

        writeln!(
            sink,
            "pub trait {}Methods {} {{",
            interface.name, other_traits
        )?;

        // Methods
        for method in interface.methods.values() {
            MethodWriter::declare(&interface.name, &method, sink)?;
            writeln!(sink, ";")?;
        }

        // Members
        for member in &interface.members {
            // member is a (String = name, (Option<Annotation>, ConcreteType, Arity))
            let ctype = member.1;
            let (mtype, _rtype) = rust_type_with_reqresp(&ctype.typ);

            // Getter: get_xxx -> mtype
            // These are infallible.
            writeln!(
                sink,
                "fn get_{}(&mut self, responder: &{}Get{}Responder);",
                member.0,
                interface.name,
                member.0.to_camel_case()
            )?;

            // Setter: set_xxx(mtype)
            // These are infallible.
            writeln!(sink, "fn set_{}(&mut self, value: {});", member.0, mtype)?;
        }

        writeln!(sink, "}}\n")?;

        if !interface.events.is_empty() {
            // Events: we create a struct usable as an event dispatcher.
            writeln!(sink, "#[derive(Clone)]")?;
            writeln!(sink, "pub struct {}EventDispatcher {{", interface.name)?;
            writeln!(sink, "helper: SessionSupport,")?;
            writeln!(sink, "object_id: TrackerId,")?;

            writeln!(sink, "}}\n")?;

            writeln!(sink, "impl {}EventDispatcher {{", interface.name)?;

            writeln!(
                sink,
                "pub fn from(helper: SessionSupport, object_id: TrackerId) -> Self {{"
            )?;
            writeln!(sink, " Self {{ helper, object_id }} }}\n")?;

            for (index, event) in interface.events.values().enumerate() {
                let ctype = &event.returns;
                let (rtype, _itype) = rust_type_with_reqresp(&ctype);

                if rtype != "()" {
                    writeln!(
                        sink,
                        "pub fn dispatch_{}(&self, value: {}) {{",
                        event.name, rtype,
                    )?;
                } else {
                    writeln!(sink, "pub fn dispatch_{}(&self) {{", event.name,)?;
                }
                writeln!(
                    sink,
                    "let service_id = self.helper.session_tracker_id().service();"
                )?;
                writeln!(sink,
                    "if is_event_in_map(&self.helper.event_map(), service_id, self.object_id, {}) {{",
                    index
                )?;

                let event_name = event.name.to_camel_case();
                let enum_value = if ctype.typ != ConcreteType::Void {
                    format!(
                        "{}ToClient::{}{}Event(value)",
                        self.toplevel_name, interface.name, event_name
                    )
                } else {
                    format!(
                        "{}ToClient::{}{}Event",
                        self.toplevel_name, interface.name, event_name
                    )
                };

                writeln!(sink, "let message = BaseMessage {{ service: service_id, object: self.object_id, kind: BaseMessageKind::Event, content: vec![] }};")?;
                writeln!(
                    sink,
                    "self.helper.serialize_message(&message, &{});",
                    enum_value
                )?;
                writeln!(sink, "}}")?;
                writeln!(sink, "}}\n")?;
            }
            writeln!(sink, "}}\n")?;

            // Events: we create a struct usable as an event broadcaster.
            writeln!(sink, "#[derive(Default)]")?;
            writeln!(sink, "pub struct {}EventBroadcaster {{", interface.name)?;
            writeln!(sink, "id: DispatcherId,")?;
            writeln!(
                sink,
                "dispatchers: HashMap<DispatcherId, {}EventDispatcher>,",
                interface.name
            )?;
            writeln!(sink, "}}\n")?;

            writeln!(sink, "impl {}EventBroadcaster {{", interface.name)?;
            writeln!(
                sink,
                r#"pub fn add(&mut self, dispatcher: &{}EventDispatcher) -> DispatcherId {{
                self.id += 1;
                self.dispatchers.insert(self.id, dispatcher.clone());
                self.id
            }}
            pub fn remove(&mut self, dispatcher_id: DispatcherId) {{
                self.dispatchers.remove(&dispatcher_id);
            }}
            pub fn log(&self) {{
                log::info!("  Registered dispatchers: {{}}", self.dispatchers.len());
            }}
            "#,
                interface.name
            )?;

            for event in interface.events.values() {
                let ctype = &event.returns;
                let event_name = &event.name;
                let (rtype, _itype) = rust_type_with_reqresp(&ctype);

                if rtype != "()" {
                    writeln!(
                        sink,
                        "pub fn broadcast_{}(&self, value: {}) {{",
                        event_name, rtype,
                    )?;
                    if needs_clone(&ctype) {
                        writeln!(
                            sink,
                            r#"for dispatcher in &self.dispatchers {{
                        dispatcher.1.dispatch_{}(value.clone());
                    }}"#,
                            event_name
                        )?;
                    } else {
                        writeln!(
                            sink,
                            r#"for dispatcher in &self.dispatchers {{
                        dispatcher.1.dispatch_{}(value);
                    }}"#,
                            event_name
                        )?;
                    }
                } else {
                    writeln!(sink, "pub fn broadcast_{}(&self) {{", event_name)?;
                    writeln!(
                        sink,
                        r#"for dispatcher in &self.dispatchers {{
                        dispatcher.1.dispatch_{}();
                    }}"#,
                        event_name
                    )?;
                }
                writeln!(sink, "}}\n")?;
            }

            writeln!(sink, "}}\n")?;
        }
        Ok(())
    }

    pub fn generate_responders<'a, W: Write>(
        &self,
        service: &Service,
        sink: &'a mut W,
    ) -> Result<()> {
        for interface in self.ast.interfaces.values() {
            let enum_name = format!("{}ToClient", service.name);

            // Method responders.
            for method in &interface.methods {
                let method = method.1;
                let camel_name = format!("{}{}", interface.name, method.name.to_camel_case());

                writeln!(sink, "#[derive(Clone)]")?;
                writeln!(sink, "pub struct {}Responder {{", camel_name)?;
                writeln!(sink, "pub transport: SessionSupport,")?;
                writeln!(sink, "pub base_message: BaseMessage,")?;
                writeln!(sink, "}}\n")?;

                writeln!(sink, "impl common::traits::CommonResponder for {}Responder {{", camel_name)?;
                sink.write_all(b"fn get_transport(&self) -> &SessionSupport {
                    &self.transport
                }\n")?;

                sink.write_all(b"fn get_base_message(&self) -> &BaseMessage {
                    &self.base_message
                }\n")?;

                writeln!(sink, "}}\n")?; // End of `impl CommonResponder`

                writeln!(sink, "impl {}Responder {{", camel_name)?;

                // Resolve with the success response.
                write!(sink, "pub fn resolve(&self")?;
                let stype = rust_type_for_resolve_reject(&self.ast, &method.returns.success);
                if stype != "()" {
                    write!(sink, ", value: {}", stype)?;
                }
                writeln!(sink, ") {{")?;
                write!(
                    sink,
                    "self.transport.serialize_message(&self.base_message, &{}::{}Success",
                    enum_name, camel_name
                )?;
                if stype != "()" {
                    let is_interface =
                        if let ConcreteType::Interface(name) = &method.returns.success.typ {
                            Some(name.clone())
                        } else {
                            None
                        };
                    write!(
                        sink,
                        "(value{})",
                        match is_interface {
                            None => "",
                            Some(name) => {
                                let info = TrackedInterfaceInfo::by_name(&self.ast, &name);
                                if info.shared() {
                                    ".lock().id()"
                                } else {
                                    ".id()"
                                }
                            }
                        }
                    )?;
                }
                writeln!(sink, ");")?;
                writeln!(sink, "}}\n")?;

                // Reject with the error response.
                write!(sink, "pub fn reject(&self")?;
                let stype = rust_type_for_resolve_reject(&self.ast, &method.returns.error);
                if stype != "()" {
                    write!(sink, ", value: {}", stype)?;
                }
                writeln!(sink, ") {{")?;
                write!(
                    sink,
                    "self.transport.serialize_message(&self.base_message, &{}::{}Error",
                    enum_name, camel_name
                )?;
                if stype != "()" {
                    let is_interface =
                        if let ConcreteType::Interface(name) = &method.returns.error.typ {
                            Some(name.clone())
                        } else {
                            None
                        };
                    write!(
                        sink,
                        "(value{})",
                        match is_interface {
                            None => "",
                            Some(name) => {
                                let info = TrackedInterfaceInfo::by_name(&self.ast, &name);
                                if info.shared() {
                                    ".lock().id()"
                                } else {
                                    ".id()"
                                }
                            }
                        }
                    )?;
                }
                writeln!(sink, ");")?;
                writeln!(sink, "}}\n")?;

                writeln!(sink, "}}\n")?; // End `impl XyzResponder`
            }

            // Getter responder. They are simpler since getter are infallible.
            for member in &interface.members {
                let name = member.0;
                let camel_name = name.to_camel_case();

                writeln!(sink, "#[derive(Clone)]")?;
                writeln!(
                    sink,
                    "pub struct {}Get{}Responder {{",
                    interface.name, camel_name
                )?;
                writeln!(sink, "pub transport: SessionSupport,")?;
                writeln!(sink, "pub base_message: BaseMessage,")?;
                writeln!(sink, "}}\n")?;

                writeln!(sink, "impl {}Get{}Responder {{", interface.name, camel_name)?;

                // Resolve with the member type.
                let repr = member.1;
                write!(sink, "pub fn resolve(&self")?;
                let stype = rust_type_for_resolve_reject(&self.ast, &repr.typ);
                if stype != "()" {
                    write!(sink, ", value: {}", stype)?;
                }
                writeln!(sink, ") {{")?;
                write!(
                    sink,
                    "self.transport.serialize_message(&self.base_message, &{}::{}Get{}",
                    enum_name, interface.name, camel_name
                )?;
                if stype != "()" {
                    let is_interface = if let ConcreteType::Interface(name) = &repr.typ.typ {
                        Some(name.clone())
                    } else {
                        None
                    };
                    write!(
                        sink,
                        "(value{})",
                        match is_interface {
                            None => "",
                            Some(name) => {
                                let info = TrackedInterfaceInfo::by_name(&self.ast, &name);
                                if info.shared() {
                                    ".lock().id()"
                                } else {
                                    ".id()"
                                }
                            }
                        }
                    )?;
                }
                writeln!(sink, ");")?;
                writeln!(sink, "}}\n")?;

                writeln!(sink, "}}\n")?;
            }
        }

        Ok(())
    }

    // Generates the proxy object used to deal with client-side objects
    fn generate_proxy<'a, W: Write>(
        &self,
        service_name: &str,
        callback: &Callback,
        sink: &'a mut W,
    ) -> Result<()> {
        writeln!(sink, "#[derive(Clone)]")?;
        writeln!(sink, "pub struct {}Proxy {{", callback.name)?;
        writeln!(
            sink,
            "helper: SessionSupport, object_id: ObjectRef, service_id: TrackerId,"
        )?;

        // For each method, keep track of the return value receiver in a map request_id -> receiver.
        for method in callback.methods.values() {
            let success = rust_type_for_param(&method.returns.success);
            let error = rust_type_for_param(&method.returns.error);
            writeln!(
                sink,
                "pub(crate) {}_requests: ProxyRequest<{}, {}>,",
                method.name, success, error
            )?;
        }

        writeln!(sink, "}}\n")?;

        writeln!(sink, "impl {}Proxy {{", callback.name)?;
        writeln!(sink, "pub fn new(object_id: ObjectRef, service_id: TrackerId, helper: &SessionSupport) -> Self {{")?;
        writeln!(
            sink,
            "Self {{ object_id, service_id, helper: helper.clone(),"
        )?;
        // For each method, keep track of the return value receiver in a map request_id -> receiver.
        for method in callback.methods.values() {
            writeln!(
                sink,
                "{}_requests: Shared::adopt(HashMap::new()),",
                method.name
            )?;
        }
        writeln!(sink, "}}\n")?;
        writeln!(sink, "}}")?;

        // Generate methods.
        for method in callback.methods.values() {
            let camel_name = method.name.to_camel_case();

            write!(sink, "pub fn {}(&mut self, ", method.name)?;

            for param in &method.params {
                let stype = rust_type_for_param(&param.typ);
                write!(sink, "{}: {},", param.name, stype)?;
            }

            // Get the return type to build the Sender/Receiver type.
            let success = rust_type_for_param(&method.returns.success);
            let error = rust_type_for_param(&method.returns.error);

            writeln!(sink, ") -> Receiver<Result<{},{}>> {{", success, error)?;
            sink.write_all(
                b"
            let (sender, receiver) = channel();
            let request = self.helper.id_factory().lock().next_id();
            let message = BaseMessage {
                service: self.service_id,
                object: self.object_id.into(),
                kind: BaseMessageKind::Request(request),
                content: vec![],
            };\n",
            )?;
            // Keep track of the sender for this request's response.
            writeln!(
                sink,
                "{{ let mut lock = self.{}_requests.lock(); lock.insert(request, sender); }}",
                method.name
            )?;
            // Create the payload.
            writeln!(
                sink,
                "let value = {}ToClient::{}{}",
                service_name, callback.name, camel_name
            )?;
            if !method.params.is_empty() {
                writeln!(sink, "(")?;
                for param in &method.params {
                    writeln!(sink, "{},", param.name)?;
                }
                writeln!(sink, ")")?;
            }
            writeln!(sink, ";")?;
            writeln!(sink, "self.helper.serialize_message(&message, &value);")?;
            writeln!(sink, "receiver")?;

            writeln!(sink, "}}\n")?;
        }

        writeln!(sink, "}}\n")?;
        Ok(())
    }

    pub fn generate<W: Write>(ast: Ast, sink: &mut W) -> Result<()> {
        if ast.services.len() > 1 {
            panic!("Multiple services in a single SIDL are not supported.");
        }

        let name = ast.services[0].name.clone();
        let mut codegen = Codegen::new(ast, &name);
        codegen.top_level(sink)
    }

    pub fn generate_gecko<W: Write>(ast: Ast, sink: &mut W) -> Result<()> {
        if ast.services.len() > 1 {
            panic!("Multiple services in a single SIDL are not supported.");
        }

        let name = ast.services[0].name.clone();
        let mut codegen = Codegen::new(ast, &name);
        codegen.gecko_client(sink)
    }

    fn gecko_client<W: Write>(&mut self, sink: &mut W) -> Result<()> {
        sink.write_all(
            b"/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is generated. Do not edit.
// @generated

#[allow(unused_imports)]
use crate::common::{JsonValue, SystemTime, ObjectRef};
use serde::{Deserialize, Serialize};
\n\n",
        )?;

        // Include the service fingerprint.
        writeln!(
            sink,
            "pub static SERVICE_FINGERPRINT: &str = \"{}\";\n",
            self.fingerprint,
        )?;

        // Include enums
        for a_enum in self.ast.enumerations.values() {
            EnumWriter::declare(&a_enum, sink)?;
        }

        // Include dictionnaries.
        for dict in self.ast.dictionaries.values() {
            DictWriter::declare(&dict, sink)?;
        }

        // Get all the possible requests and responses.
        let (reqs, resps) = get_all_reqs_resps(&self.ast);

        // Generate an enum with all the possible messages received from the client.
        let mut index = 0;
        writeln!(sink, "#[derive(Debug, Deserialize, Serialize)]")?;
        writeln!(sink, "pub enum {}FromClient {{", self.toplevel_name)?;
        for req in &reqs {
            writeln!(sink, "{}, // {}", req, index)?;
            index += 1;
        }
        writeln!(sink, "}}\n")?;

        // Generate an enum with all the possible messages send to the client.
        index = 0;
        writeln!(sink, "#[derive(Debug, Deserialize)]")?;
        writeln!(sink, "pub enum {}ToClient {{", self.toplevel_name)?;
        for resp in &resps {
            writeln!(sink, "{}, // {}", resp, index)?;
            index += 1;
        }
        writeln!(sink, "}}\n")?;

        Ok(())
    }

    fn top_level<W: Write>(&mut self, sink: &mut W) -> Result<()> {
        let need_object_tracker =
            self.ast
                .interfaces
                .values()
                .any(|interface| match &interface.annotation {
                    Some(annotation) => !annotation.has("rust:not-tracked"),
                    None => true,
                });

        let has_tracked_interfaces = {
            let interface = self
                .ast
                .interfaces
                .get(&self.ast.services[0].interface)
                .unwrap();
            interface.methods.values().any(|method| {
                if let ConcreteType::Interface(_) = &method.returns.success.typ {
                    return true;
                }
                if let ConcreteType::Interface(_) = &method.returns.error.typ {
                    return true;
                }
                false
            })
        };

        sink.write_all(
            b"// This file is generated. Do not edit.
// @generated\n
#[allow(unused_imports)]
use common::core::{BaseMessage, BaseMessageKind};
#[allow(unused_imports)]
use common::{JsonValue, SystemTime, is_event_in_map};
#[allow(unused_imports)]
use common::traits::{DispatcherId, OriginAttributes, SessionSupport, Shared, TrackerId};
#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::sync::mpsc::{Sender, Receiver, channel};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ObjectRef(TrackerId);
impl From<TrackerId> for ObjectRef {
    fn from(val: TrackerId) -> Self {
        Self(val)
    }
}
impl From<ObjectRef> for TrackerId {
    fn from(val: ObjectRef) -> Self {
        val.0
    }
}

#[allow(dead_code)]
type ProxyRequest<T, E> = Shared<HashMap<u64, Sender<Result<T, E>>>>;
\n\n",
        )?;

        // Check if any interface uses a "rust:shared" annotation, and if so emit appropriate `use` statements.
        if self
            .ast
            .interfaces
            .values()
            .any(|interface| match &interface.annotation {
                Some(annotation) => annotation.has("rust:shared"),
                None => false,
            })
        {
            sink.write_all(b"use std::sync::Arc; use parking_lot::Mutex;\n")?;
        }

        // Add custom "use" declarations from the service.
        if let Some(annotation) = &self.ast.services[0].annotation {
            for decl in annotation.get_values("rust:use") {
                writeln!(sink, "use {};", decl)?;
            }
        }

        if need_object_tracker {
            sink.write_all(b"use common::traits::SimpleObjectTracker;\n")?;
        }

        if has_tracked_interfaces {
            sink.write_all(b"#[allow(unused_imports)] use std::rc::Rc;\n")?;
        }

        // Generate interface traits. They can be shared by the service and
        // client implementations.
        for interface in &self.ast.interfaces {
            let interface = interface.1;
            self.generate_interface(&interface, sink)?;
        }

        // Generate enums.
        for a_enum in &self.ast.enumerations {
            EnumWriter::declare(&a_enum.1, sink)?;
        }

        // Generate stucts from dictionaries.
        for a_dict in &self.ast.dictionaries {
            DictWriter::declare(&a_dict.1, sink)?;
        }

        for callback in self.ast.callbacks.values() {
            self.generate_proxy(&self.ast.services[0].name, &callback, sink)?;
        }

        // Generate service responders.
        for service in &self.ast.services {
            self.generate_responders(&service, sink)?;
        }

        // Get all the possible requests and responses.
        let (reqs, resps) = get_all_reqs_resps(&self.ast);

        // Generate an enum with all the possible messages received from the client.
        let mut index = 0;
        writeln!(sink, "#[derive(Debug, Deserialize, Serialize)]")?;
        writeln!(sink, "pub enum {}FromClient {{", self.toplevel_name)?;
        for req in &reqs {
            writeln!(sink, "{}, // {}", req, index)?;
            index += 1;
        }
        writeln!(sink, "}}\n")?;

        // Generate an enum with all the possible messages send to the client.
        index = 0;
        writeln!(sink, "#[derive(Serialize)]")?;
        writeln!(sink, "pub enum {}ToClient {{", self.toplevel_name)?;
        for resp in &resps {
            writeln!(sink, "{}, // {}", resp, index)?;
            index += 1;
        }
        writeln!(sink, "}}\n")?;

        Ok(())
    }
}
