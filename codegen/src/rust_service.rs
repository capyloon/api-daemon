/// Rust service code generator.
use crate::ast_utils::*;
use heck::CamelCase;
use sidl_parser::ast::{Ast, ConcreteType, Service};
use std::collections::HashSet;
use std::io::Write;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] ::std::io::Error),
    #[error("Common error")]
    Common(#[from] crate::rust_common::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

pub struct Codegen {
    ast: Ast,
    toplevel_name: String,
}

impl Codegen {
    pub fn new(ast: Ast, name: &str) -> Self {
        Self {
            ast,
            toplevel_name: name.into(),
        }
    }

    fn has_tracked_interfaces(&self, service: &Service) -> bool {
        let interface = self.ast.interfaces.get(&service.interface).unwrap();
        for method in &interface.methods {
            let method = method.1;
            if let ConcreteType::Interface(_) = &method.returns.success.typ {
                return true;
            }
            if let ConcreteType::Interface(_) = &method.returns.error.typ {
                return true;
            }
        }

        false
    }

    pub fn find_tracked_interfaces(
        &self,
        interface: &sidl_parser::ast::Interface,
        set: &mut HashSet<TrackedInterfaceInfo>,
    ) {
        for method in &interface.methods {
            let method = method.1;
            if let ConcreteType::Interface(name) = &method.returns.success.typ {
                set.insert(TrackedInterfaceInfo::by_name(&self.ast, name));
                self.find_tracked_interfaces(self.ast.interfaces.get(name).unwrap(), set);
            }
            if let ConcreteType::Interface(name) = &method.returns.error.typ {
                set.insert(TrackedInterfaceInfo::by_name(&self.ast, name));
                self.find_tracked_interfaces(self.ast.interfaces.get(name).unwrap(), set);
            }
        }
    }

    // Services are mapped to trait that implement both their defining interface and the
    // core common one.
    pub fn generate_service<'a, W: Write>(
        &mut self,
        service: &Service,
        sink: &'a mut W,
    ) -> Result<()> {
        // Find all interfaces that will be tracked objects for this service.
        // These are all success/error returned interfaces, with a recursive descent.
        let mut tracked_interfaces = HashSet::new();
        let interface = self.ast.interfaces.get(&service.interface).unwrap();
        self.find_tracked_interfaces(&interface, &mut tracked_interfaces);

        // Check the "rust:shared-tracker" annotation to decide if we will use
        // Arc<Mutex<TrackerType>> or TrackerType.
        let use_shared_tracker = if let Some(annotation) = &service.annotation {
            annotation.has("rust:shared-tracker")
        } else {
            false
        };

        // Check the "rust:shared-proxy-tracker" annotation to decide if we will use
        // Arc<Mutex<ProxyTracker>> or ProxyTracker.
        let use_shared_proxy_tracker = if let Some(annotation) = &service.annotation {
            annotation.has("rust:shared-proxy-tracker")
        } else {
            false
        };

        if use_shared_proxy_tracker {
            sink.write_all(b"use std::sync::Arc; use parking_lot::Mutex;")?;
        }

        let service_name = &service.name;

        writeln!(
            sink,
            "pub static SERVICE_NAME: &str = \"{}\";\n",
            service_name
        )?;

        writeln!(
            sink,
            "pub static SERVICE_FINGERPRINT: &str = \"{}\";\n",
            crate::helpers::get_fingerprint(&self.ast)
        )?;

        // Generate a function to check if the caller has the required permission to create that service.
        // The permission name is defined by adding a service annotation : #[permission=permission-name]
        // If no permission restriction is set, all callers can create the service.
        if let Some(annotation) = &service.annotation {
            let permissions = annotation.get_values("permission");
            sink.write_all(b"#[allow(unused_variables)] pub fn check_service_permission(origin_attributes: &OriginAttributes) -> bool {\n")?;
            // We only support setting one permission for now.
            if permissions.len() > 1 {
                panic!(
                    "Only one service permission allowed, but found {:?}",
                    permissions
                );
            } else if permissions.is_empty() {
                // We can have other service annotations, but not the permission one.
                sink.write_all(b"true\n")?;
            } else {
                sink.write_all(b"let identity = origin_attributes.identity();
                                if identity == \"uds\" {
                                    // Grant all permissions to uds sessions.
                                    true
                                } else {\n")?;
                writeln!(
                    sink,
                    "origin_attributes.has_permission(\"{}\") }}",
                    permissions[0]
                )?;
            }
            sink.write_all(b"}\n")?;
        } else {
            // No annotation.
            sink.write_all(
                b"pub fn check_service_permission(_: &OriginAttributes) -> bool { true }\n",
            )?;
        }

        // Generate the enum for tracked (produced objects)
        if !tracked_interfaces.is_empty() {
            writeln!(sink, "// Will track: {:?}", tracked_interfaces)?;
            writeln!(sink, "pub enum {}TrackedObject {{", service_name)?;
            for tracked in &tracked_interfaces {
                write!(
                    sink,
                    "{}({}),",
                    tracked.interface_name(),
                    tracked.type_representation()
                )?;
            }
            writeln!(sink, "}}")?;
            writeln!(
                sink,
                "pub type {}TrackerType = ObjectTracker<{}TrackedObject, TrackerId>;\n",
                service_name, service_name
            )?;
        }

        // Generate the enum for tracked callback objects when needed.
        if !self.ast.callbacks.is_empty() {
            writeln!(sink, "pub enum {}Proxy {{", service_name)?;
            for callback in self.ast.callbacks.values() {
                writeln!(sink, "{}({}Proxy),", callback.name, callback.name)?;
            }
            writeln!(sink, "}}")?;
            writeln!(
                sink,
                "pub type {}ProxyTracker = HashMap<ObjectRef, {}Proxy>;",
                service_name, service_name
            )?;
            writeln!(sink, "\n")?;
        }

        writeln!(
            sink,
            "pub trait {} : {}Methods {{",
            service_name, service.interface
        )?;

        if !tracked_interfaces.is_empty() {
            if use_shared_tracker {
                writeln!(
                    sink,
                    "fn get_tracker(&mut self) -> Arc<Mutex<{}TrackerType>>;\n",
                    service_name
                )?;
            } else {
                writeln!(
                    sink,
                    "fn get_tracker(&mut self) -> &mut {}TrackerType;\n",
                    service_name
                )?;
            }
        }

        // Only need a proxy tracker when there are callback objects.
        if !self.ast.callbacks.is_empty() {
            if use_shared_proxy_tracker {
                writeln!(
                    sink,
                    "fn get_proxy_tracker(&mut self) -> Arc<Mutex<{}ProxyTracker>>;\n",
                    service_name
                )?;
            } else {
                writeln!(
                    sink,
                    "fn get_proxy_tracker(&mut self) -> &mut {}ProxyTracker;\n",
                    service_name
                )?;
            }

            writeln!(sink, "fn maybe_add_proxy<F>(&mut self, object_ref: ObjectRef, builder: F) where F: FnOnce() -> {}Proxy,", service_name)?;
            if use_shared_proxy_tracker {
                sink.write_all(
                    b"{
                        let tracker = self.get_proxy_tracker();
                        let mut lock = tracker.lock();
                        lock.entry(object_ref).or_insert_with(builder);
                    }\n\n",
                )?;
            } else {
                sink.write_all(
                    b"{
                        let tracker = self.get_proxy_tracker();
                        tracker.entry(object_ref).or_insert_with(builder);
                    }\n\n",
                )?;
            }
        }

        // Generate the main dispatcher function, calling the methods from the Methods traits and sending
        // back results.
        // TODO: use annotations for special behavior like synchronous methods.
        sink.write_all(
            b"/// Called once we have checked that BaseMessage was targetted at this service.\n",
        )?;
        writeln!(
            sink,
            "fn dispatch_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {{"
        )?;
        writeln!(sink, "use self::{}FromClient as Req;", self.toplevel_name)?;
        writeln!(sink, "let req: Result<{}FromClient, common::BincodeError> = common::deserialize_bincode(&message.content);", self.toplevel_name)?;
        writeln!(sink, "match req  {{")?;
        writeln!(sink, "Ok(req) =>  {{")?;
        writeln!(
            sink,
            "let mut base_message =  BaseMessage::empty_from(message);"
        )?;
        writeln!(sink, "if let BaseMessageKind::Request(_) = message.kind {{")?;
        writeln!(
            sink,
            "base_message.kind = BaseMessageKind::Response(message.request());"
        )?;
        writeln!(sink, "}}")?;
        writeln!(sink, "let transport = transport.clone();")?;
        writeln!(sink, "match req {{")?;
        // Check all request signatures from methods, getters and setters.
        let interface = self.ast.interfaces.get(&service.interface).unwrap();
        for method in &interface.methods {
            let method = method.1;
            let mut req_name = format!("{}{}", interface.name, method.name.to_camel_case());
            let mut params = String::new();
            let mut variant_params = String::new();
            let mut bootstrap = String::new();
            if !method.params.is_empty() {
                req_name.push('(');
                for param in &method.params {
                    // If the parameter is a callback, use a proxy instead of the raw parameter
                    // which is the object id.
                    if let ConcreteType::Callback(name) = &param.typ.typ {
                        bootstrap.push_str(&format!("let object_ref = {};", param.name));
                        bootstrap.push_str("self.maybe_add_proxy(object_ref, || {");
                        bootstrap.push_str(&format!(
                            "{}Proxy::{}({}Proxy::new({}, message.service, &transport))}});",
                            service_name, name, name, param.name,
                        ));

                        params.push_str("object_ref");
                    } else {
                        params.push_str(&format!("{}, ", param.name));
                    }
                    variant_params.push_str(&format!("{}, ", param.name));
                }
                req_name.push_str(&variant_params);
                req_name.push(')');
            }
            writeln!(
                sink,
                "Req::{} => {{ {} self.{}(&{}{}Responder {{ transport, base_message }}, {}); }}",
                req_name,
                bootstrap,
                method.name,
                interface.name,
                method.name.to_camel_case(),
                params
            )?;
        }

        for member in &interface.members {
            let name = member.0;
            writeln!(
                sink,
                "Req::{}Set{}(val) => self.set_{}(val),",
                interface.name,
                name.to_camel_case(),
                name
            )?;

            writeln!(sink,
                "Req::{}Get{} => {{ self.get_{}(&{}Get{}Responder {{ transport, base_message }}); }}",
                interface.name,
                name.to_camel_case(),
                name,
                interface.name,
                name.to_camel_case(),
            )?;
        }

        for tracked in &tracked_interfaces {
            let interface = self.ast.interfaces.get(&tracked.interface_name()).unwrap();

            // Methods on tracked objects.
            for method in &interface.methods {
                let method = method.1;
                let mut req_name = format!("{}{}", interface.name, method.name.to_camel_case());
                let mut params = String::new();
                let mut variant_params = String::new();
                let mut bootstrap = String::new();
                if !method.params.is_empty() {
                    req_name.push('(');
                    for param in &method.params {
                        // If the parameter is an interface, use a proxy instead of the raw parameter
                        // which is the object id.
                        if let ConcreteType::Interface(name) = &param.typ.typ {
                            bootstrap.push_str(&format!("let object_ref = {};", param.name));
                            bootstrap.push_str("self.maybe_add_proxy(object_ref, || {");
                            bootstrap.push_str(&format!(
                                "{}Proxy::{}({}Proxy::new({}, &transport))}});",
                                service_name, name, name, param.name
                            ));

                            params.push_str("object_ref");
                        } else {
                            params.push_str(&format!("{}, ", param.name));
                        }
                        variant_params.push_str(&format!("{}, ", param.name));
                    }
                    req_name.push_str(&variant_params);
                    req_name.push(')');
                }
                writeln!(sink, "Req::{} => {{", req_name)?;
                // Get the object from the tracker, and call the method on the object if possible.
                writeln!(sink, "let tracker = self.get_tracker();")?;
                if use_shared_tracker {
                    writeln!(sink, "let mut tracker = tracker.lock();")?;
                }
                writeln!(
                    sink,
                    "if let Some({}TrackedObject::{}(ctxt)) = tracker.get_mut(message.object) {{",
                    service_name,
                    tracked.interface_name()
                )?;
                if tracked.shared() {
                    writeln!(sink, "let mut mut_ctxt = ctxt.lock();")?;
                } else {
                    writeln!(sink, "let mut_ctxt = Rc::get_mut(ctxt).unwrap();")?;
                }
                writeln!(
                    sink,
                    "mut_ctxt.{}(&{}{}Responder {{ transport, base_message }}, {});",
                    method.name,
                    interface.name,
                    method.name.to_camel_case(),
                    params
                )?;
                writeln!(sink, "}}")?;

                writeln!(sink, "}}")?; // End of Req::() =>
            }

            // Members on tracked objects.
            for member in &interface.members {
                let name = member.0;
                let camel_name = name.to_camel_case();

                // Setter
                writeln!(sink, "Req::{}Set{}(val) => {{", interface.name, camel_name,)?;
                writeln!(sink, "let tracker = self.get_tracker();")?;
                if use_shared_tracker {
                    writeln!(sink, "let mut tracker = tracker.lock();")?;
                }
                writeln!(
                    sink,
                    "if let Some({}TrackedObject::{}(ctxt)) = tracker.get_mut(message.object) {{",
                    service_name,
                    tracked.interface_name()
                )?;
                if tracked.shared() {
                    writeln!(sink, "let mut mut_ctxt = ctxt.lock();")?;
                } else {
                    writeln!(sink, "let mut_ctxt = Rc::get_mut(ctxt).unwrap();")?;
                }
                writeln!(sink, "mut_ctxt.set_{}(val);", name)?;
                writeln!(sink, "}} else {{")?;
                writeln!(sink, "error!(\"Expected {}\");", tracked.interface_name())?;
                writeln!(sink, "}}")?;

                writeln!(sink, "}}")?; // End of Req::() =>

                // Getter
                writeln!(sink, "Req::{}Get{} => {{", interface.name, camel_name)?;
                writeln!(sink, "let tracker = self.get_tracker();")?;
                if use_shared_tracker {
                    writeln!(sink, "let mut tracker = tracker.lock();")?;
                }
                writeln!(
                    sink,
                    "if let Some({}TrackedObject::{}(ctxt)) = tracker.get_mut(message.object) {{",
                    service_name,
                    tracked.interface_name()
                )?;
                if tracked.shared() {
                    writeln!(sink, "let mut mut_ctxt = ctxt.lock();")?;
                } else {
                    writeln!(sink, "let mut_ctxt = Rc::get_mut(ctxt).unwrap();")?;
                }
                writeln!(
                    sink,
                    "mut_ctxt.get_{}(&{}Get{}Responder {{ transport, base_message }});",
                    name, interface.name, camel_name
                )?;
                writeln!(sink, "}} else {{")?;
                writeln!(sink, "error!(\"Expected {}\");", tracked.interface_name())?;
                writeln!(sink, "}}")?;

                writeln!(sink, "}}")?; // End of Req::() =>
            }
        }

        // Messages from client side callback proxies. These are BaseMessageKind::Response because
        // of the call semantics, even if they are received here.
        for callback in self.ast.callbacks.values() {
            // Methods on client side objects. We care about the returned values instead
            // of the method parameters!
            for method in callback.methods.values() {
                let returned = &method.returns;

                // Success return value.
                let mut req_name =
                    format!("{}{}Success", callback.name, method.name.to_camel_case());
                if returned.success.typ != ConcreteType::Void {
                    req_name.push_str("(value)");
                }

                writeln!(sink, "Req::{} => {{", req_name)?;
                // 1. If this is returning a callback, add it to the proxy tracker.
                if let ConcreteType::Callback(callback) = &returned.success.typ {
                    writeln!(sink, "self.maybe_add_proxy(value, || {{")?;
                    writeln!(
                        sink,
                        "{}Proxy::{}({}Proxy::new(",
                        service.name, callback, callback
                    )?;
                    writeln!(sink, "value,")?;
                    writeln!(sink, "message.service,")?;
                    writeln!(sink, "&transport,")?;
                    writeln!(sink, "))")?;
                    writeln!(sink, "}});")?;
                }

                // 2. Get the appropriate proxy object.
                writeln!(sink, "let object_ref = ObjectRef::from(message.object);")?;
                writeln!(sink, "let tracker = self.get_proxy_tracker();")?;
                if use_shared_proxy_tracker {
                    writeln!(sink, "let mut tracker = tracker.lock();")?;
                }
                writeln!(
                    sink,
                    "if let Some({}Proxy::{}(proxy))  = tracker.get_mut(&object_ref) {{",
                    service.name, callback.name
                )?;
                // 3. Get the request id.
                writeln!(
                    sink,
                    "if let BaseMessageKind::Response(request) = message.kind {{"
                )?;

                // 4. Get the sender for this call.
                writeln!(
                    sink,
                    "let mut lock = proxy.{}_requests.lock();",
                    method.name
                )?;
                writeln!(sink, "if let Some(sender) = lock.get(&request) {{")?;
                // 5. send an Ok() response.
                let param_type = if returned.success.typ != ConcreteType::Void {
                    "value"
                } else {
                    "()"
                };
                // TODO: manage error from send()
                writeln!(sink, "let _ = sender.send(Ok({}));", param_type)?;
                // 6. Remove this sender since we can't receive multiple messages for the same request.
                writeln!(sink, "lock.remove(&request);")?;
                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Failed to get sender for request #{{}} from {}_requests\", request);",
                    method.name
                )?;
                writeln!(sink, "}}")?; // End of `if let Some(sender)`

                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Message is not a request: #{{:?}}\", message.kind);"
                )?;
                writeln!(sink, "}}")?; // End of `if let BaseMessageKind::Response(request)`

                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Failed to get {}Proxy::{} for object #{{}}\", message.object);",
                    service.name, callback.name
                )?;
                writeln!(sink, "}}")?; // End of `if let Some( Proxy )`

                writeln!(sink, "}}")?; // End of `Req::() => ...`

                // Error return value.
                // TODO: refactor to share code with the success case
                let mut req_name = format!("{}{}Error", callback.name, method.name.to_camel_case());
                if returned.error.typ != ConcreteType::Void {
                    req_name.push_str("(value)");
                }
                writeln!(sink, "Req::{} => {{", req_name)?;
                // 1. If this is returning a callback, add it to the proxy tracker.
                if let ConcreteType::Callback(callback) = &returned.error.typ {
                    writeln!(sink, "self.maybe_add_proxy(value, || {{")?;
                    writeln!(
                        sink,
                        "{}Proxy::{}({}Proxy::new(",
                        service.name, callback, callback
                    )?;
                    writeln!(sink, "value,")?;
                    writeln!(sink, "message.service,")?;
                    writeln!(sink, "&transport,")?;
                    writeln!(sink, "))")?;
                    writeln!(sink, "}});")?;
                }

                // 2. Get the appropriate proxy object.
                writeln!(sink, "let object_ref = ObjectRef::from(message.object);")?;
                writeln!(sink, "let tracker = self.get_proxy_tracker();")?;
                if use_shared_proxy_tracker {
                    writeln!(sink, "let mut tracker = tracker.lock();")?;
                }
                writeln!(
                    sink,
                    "if let Some({}Proxy::{}(proxy))  = tracker.get_mut(&object_ref) {{",
                    service.name, callback.name
                )?;

                // 3. Get the request id.
                writeln!(
                    sink,
                    "if let BaseMessageKind::Response(request) = message.kind {{"
                )?;

                // 4. Get the sender for this call.
                writeln!(
                    sink,
                    "let mut lock = proxy.{}_requests.lock();",
                    method.name
                )?;
                writeln!(sink, "if let Some(sender) = lock.get(&request) {{")?;
                // 5. send an Err() response.
                let param_type = if returned.error.typ != ConcreteType::Void {
                    "value"
                } else {
                    "()"
                };
                // TODO: manage error from send()
                writeln!(sink, "let _ = sender.send(Err({}));", param_type)?;
                // 6. Remove this sender since we can't receive multiple messages for the same request.
                writeln!(sink, "lock.remove(&request);")?;
                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Failed to get sender for request #{{}} from {}_requests\", request);",
                    method.name
                )?;
                writeln!(sink, "}}")?; // End of `if let Some(sender)`

                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Message is not a request: #{{:?}}\", message.kind);"
                )?;
                writeln!(sink, "}}")?; // End of `if let BaseMessageKind::Response(request)`

                writeln!(sink, "}} else {{")?;
                writeln!(
                    sink,
                    "error!(\"Failed to get {}Proxy::{} for object #{{}}\", message.object);",
                    service.name, callback.name
                )?;
                writeln!(sink, "}}")?; // End of `if let Some( Proxy )`

                writeln!(sink, "}}")?; // `End of Req::() => ...`
            }
        }

        writeln!(sink, "}}")?;
        writeln!(sink, "}}")?;
        writeln!(sink, "Err(err) => {{")?;
        writeln!(sink, "error!(\"Unable to process request for service #{{}} object #{{}}: {{:?}}\", message.service, message.object, err);")?;
        writeln!(sink, "error!(\"content is {{:?}}\", message.content);")?;
        writeln!(sink, "}}")?;
        writeln!(sink, "}}")?;
        writeln!(sink, "}}")?;
        writeln!(sink, "}}\n")?;
        Ok(())
    }

    pub fn generate<W: Write>(ast: Ast, sink: &mut W) -> Result<()> {
        if ast.services.len() > 1 {
            panic!("Multiple services in a single SIDL are not supported.");
        }

        let name = ast.services[0].name.clone();
        let mut codegen = Codegen::new(normalize_rust_case(&ast, &RustCaseNormalizer), &name);
        codegen.top_level(sink)
    }

    pub fn top_level<W: Write>(&mut self, sink: &mut W) -> Result<()> {
        sink.write_all(
            b"// This file is generated. Do not edit.
// @generated\n\n
use super::common::*;
use common::core::{BaseMessage, BaseMessageKind};
#[allow(unused_imports)]
use common::traits::{DispatcherId, ObjectTrackerMethods, OriginAttributes, SharedEventMap, SessionSupport, SessionTrackerId, SimpleObjectTracker, TrackerId};
#[allow(unused_imports)]
use common::{JsonValue, SystemTime, is_event_in_map};
use log::error;
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::rc::Rc;
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

        if self.has_tracked_interfaces(&self.ast.services[0]) {
            sink.write_all(b"use common::object_tracker::ObjectTracker;\n")?;
        }

        let ast = self.ast.clone();

        // Generate service trait.
        for service in &ast.services {
            self.generate_service(&service, sink)?;
        }

        Ok(())
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

        interface MyType {
            event test1
        }

        interface Nothing {
        }

        callback SomeObject {
            fn call_me(maybe: int) -> str
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

        Codegen::generate(ast, &mut ::std::io::stdout()).expect("Failed to generate rust code!");
    }
}
