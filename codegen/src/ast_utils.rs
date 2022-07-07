/// Utilities to extract some features from the AST.
use heck::*;
use sidl_parser::ast::*;
use std::collections::HashMap;

// Provides the list of interfaces used in this AST, and
// if they are consumed, produced, or both from the point
// of view of the service implementation.
// An interface is produced if it's a method result.
// An interface is consumed if it's a method parameter.
// An interface implementing a service is produced.

#[derive(PartialEq, Clone, Debug)]
pub enum InterfaceUsage {
    Produced,
    Consumed,
    Both,
}

impl InterfaceUsage {
    pub fn update(&self, value: InterfaceUsage) -> Self {
        if *self == InterfaceUsage::Both {
            // Nothing can change anymore at this point.
            return value;
        }

        if value != *self || value == InterfaceUsage::Both {
            return InterfaceUsage::Both;
        }

        (*self).clone()
    }
}

pub struct UsedInterface {
    pub interface: Interface,
    pub usage: InterfaceUsage,
}

#[derive(Default)]
pub struct InterfaceList {
    pub interfaces: Vec<UsedInterface>,
}

impl InterfaceList {
    pub fn from_ast(ast: &Ast) -> Self {
        let mut interfaces: HashMap<String, InterfaceUsage> = HashMap::new();

        macro_rules! update_map {
            ($name:expr, $value:expr) => {
                match interfaces.get(&$name) {
                    Some(value) => {
                        let final_value = value.clone().update($value);
                        interfaces.insert($name.clone(), final_value);
                    }
                    None => {
                        interfaces.insert($name.clone(), $value);
                    }
                }
            };
        }
        // Interfaces implementing services are produced.
        for service in &ast.services {
            update_map!(service.interface, InterfaceUsage::Produced);
        }

        for interface in &ast.interfaces {
            let interface = interface.1;

            // Check methods return values and parameters.
            for method in &interface.methods {
                if let ConcreteType::Interface(name) = &method.1.returns.success.typ {
                    update_map!(*name, InterfaceUsage::Produced);
                }

                if let ConcreteType::Interface(name) = &method.1.returns.error.typ {
                    update_map!(*name, InterfaceUsage::Produced);
                }

                for param in &method.1.params {
                    if let ConcreteType::Interface(name) = &param.typ.typ {
                        update_map!(*name, InterfaceUsage::Consumed);
                    }
                }
            }

            // Check event types.
            for event in &interface.events {
                if let ConcreteType::Interface(name) = &event.1.returns.typ {
                    update_map!(*name, InterfaceUsage::Produced);
                }
            }

            // Check member types.
            for member in &interface.members {
                if let ConcreteType::Interface(name) = &member.1.typ.typ {
                    update_map!(*name, InterfaceUsage::Both);
                }
            }
        }

        let list = interfaces
            .iter()
            .map(|item| UsedInterface {
                interface: ast.interfaces.get(item.0).unwrap().clone(),
                usage: (*item.1).clone(),
            })
            .collect();

        InterfaceList { interfaces: list }
    }

    pub fn get_usage_for(&self, name: &str) -> Option<InterfaceUsage> {
        self.interfaces
            .iter()
            .find(|item| item.interface.name == name)
            .map(|item| item.usage.clone())
    }

    pub fn is_consumed(&self, name: &str) -> bool {
        match self.get_usage_for(name) {
            Some(value) => value == InterfaceUsage::Consumed || value == InterfaceUsage::Both,
            None => false,
        }
    }

    pub fn is_produced(&self, name: &str) -> bool {
        match self.get_usage_for(name) {
            Some(value) => value == InterfaceUsage::Produced || value == InterfaceUsage::Both,
            None => false,
        }
    }

    // Returns an iterator over all items of this usage, giving the interface name.
    pub fn iter_for(&self, usage: InterfaceUsage) -> impl Iterator<Item = String> + '_ {
        self.interfaces.iter().filter_map(move |item| {
            if item.usage != usage {
                None
            } else {
                Some(item.interface.name.to_owned())
            }
        })
    }
}

pub trait CaseNormalizer {
    fn service_name(&self, from: &str) -> String;
    fn interface_name(&self, from: &str) -> String;
    fn enumeration_name(&self, from: &str) -> String;
    fn enumeration_member(&self, from: &str) -> String;
    fn dictionary_name(&self, from: &str) -> String;
    fn method_name(&self, from: &str) -> String;
    fn member_name(&self, from: &str) -> String;
    fn parameter_name(&self, from: &str) -> String;
    fn event_name(&self, from: &str) -> String;
}

pub struct RustCaseNormalizer;
// Rust case normalization rules:
// CamelCase for services name and interface.
// CamelCase for enumerations names
// CamelCase for dictionary names, snake_case for dictionary members
// CamelCase for interfaces and callback names
// snake_case for method names, member names, parameter names and event names
impl CaseNormalizer for RustCaseNormalizer {
    fn service_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn interface_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn enumeration_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn enumeration_member(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn dictionary_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn method_name(&self, from: &str) -> String {
        from.to_snake_case()
    }
    fn member_name(&self, from: &str) -> String {
        from.to_snake_case()
    }
    fn parameter_name(&self, from: &str) -> String {
        from.to_snake_case()
    }
    fn event_name(&self, from: &str) -> String {
        from.to_snake_case()
    }
}

pub struct JavascriptCaseNormalizer;
// Javascript normalization rules, following https://www.w3.org/TR/api-design/#casing
impl CaseNormalizer for JavascriptCaseNormalizer {
    fn service_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn interface_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn enumeration_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn enumeration_member(&self, from: &str) -> String {
        // These are like constants.
        from.to_shouty_snake_case()
    }
    fn dictionary_name(&self, from: &str) -> String {
        from.to_upper_camel_case()
    }
    fn method_name(&self, from: &str) -> String {
        from.to_lower_camel_case()
    }
    fn member_name(&self, from: &str) -> String {
        from.to_lower_camel_case()
    }
    fn parameter_name(&self, from: &str) -> String {
        from.to_lower_camel_case()
    }
    fn event_name(&self, from: &str) -> String {
        from.to_shouty_snake_case()
    }
}

// Normalizes the names of various structures according to the conventions
// that will be used in the Rust generated code.
// This reduces the use of .to_snake_case() and .to_upper_camel_case() in the
// code generator.
// Also renames methods when findind a #[rust:rename=...] annotation.
pub fn normalize_rust_case<N: CaseNormalizer>(ast: &Ast, normalizer: &N) -> Ast {
    let mut dest = Ast {
        services: ast
            .services
            .iter()
            .map(|service| Service {
                annotation: service.annotation.clone(),
                name: normalizer.service_name(&service.name),
                interface: normalizer.interface_name(&service.interface),
            })
            .collect(),
        ..Default::default()
    };

    for enumeration in &ast.enumerations {
        let name = enumeration.0;
        let enumeration = enumeration.1;
        let other = Enumeration {
            annotation: enumeration.annotation.clone(),
            name: normalizer.enumeration_name(&enumeration.name),
            members: enumeration
                .members
                .iter()
                .map(|item| {
                    let mut res = item.clone();
                    res.name = normalizer.enumeration_member(&item.name);
                    res
                })
                .collect(),
        };

        dest.enumerations.insert(name.into(), other);
    }

    for dict in &ast.dictionaries {
        let name = dict.0;
        let dict = dict.1;
        let other = Dictionary {
            annotation: dict.annotation.clone(),
            name: normalizer.dictionary_name(&dict.name),
            members: dict
                .members
                .iter()
                .map(|member| {
                    let mut res = member.clone();
                    res.name = normalizer.member_name(&member.name);
                    res
                })
                .collect(),
        };

        dest.dictionaries.insert(name.into(), other);
    }

    for interface in &ast.interfaces {
        let name = interface.0;
        let interface = interface.1;

        let mut other = interface.clone();
        other.name = normalizer.interface_name(&interface.name);
        other.methods.clear();
        for method in &interface.methods {
            let mut name = method.0.to_owned();
            let method = method.1;
            // Rename the method is needed.
            if let Some(annotation) = &method.annotation {
                let renaming = annotation.get_values("rust:rename");
                if !renaming.is_empty() {
                    name = renaming[0].into();
                }
            }
            let mut new_m = method.clone();
            new_m.name = normalizer.method_name(&name);
            new_m.params.clear();
            for param in &method.params {
                let mut new_p = param.clone();
                new_p.name = normalizer.parameter_name(&param.name);
                let typ = match new_p.typ.typ {
                    ConcreteType::Enumeration(name) => {
                        ConcreteType::Enumeration(normalizer.enumeration_name(&name))
                    }
                    ConcreteType::Dictionary(name) => {
                        ConcreteType::Dictionary(normalizer.dictionary_name(&name))
                    }
                    ConcreteType::Interface(name) => {
                        ConcreteType::Interface(normalizer.interface_name(&name))
                    }
                    _ => new_p.typ.typ,
                };
                new_p.typ = FullConcreteType {
                    typ,
                    arity: new_p.typ.arity,
                    extra: new_p.typ.extra,
                };
                new_m.params.push(new_p);
            }

            other.methods.insert(name.into(), new_m);
        }

        other.events.clear();
        for event in &interface.events {
            let name = event.0;
            let event = event.1;

            let mut new_event = event.clone();
            new_event.name = normalizer.event_name(&event.name);
            let typ = match new_event.returns.typ {
                ConcreteType::Enumeration(name) => {
                    ConcreteType::Enumeration(normalizer.enumeration_name(&name))
                }
                ConcreteType::Dictionary(name) => {
                    ConcreteType::Dictionary(normalizer.dictionary_name(&name))
                }
                ConcreteType::Interface(name) => {
                    ConcreteType::Interface(normalizer.interface_name(&name))
                }
                _ => new_event.returns.typ,
            };
            new_event.returns = FullConcreteType {
                typ,
                arity: new_event.returns.arity,
                extra: event.returns.extra.clone(),
            };

            other.events.insert(name.into(), new_event);
        }

        other.members.clear();
        for member in interface.members.values() {
            let name = normalizer.method_name(&member.name);
            let new_member =
                DictionaryMember::new(&name, member.annotation.clone(), member.typ.clone());
            other.members.insert(name, new_member);
        }

        dest.interfaces.insert(name.into(), other);
    }

    // Callbacks are restricted interfaces, so we use the same normalizing rules
    // as the ones for interfaces.
    for callback in &ast.callbacks {
        let name = callback.0;
        let callback = callback.1;

        let mut other = callback.clone();
        other.name = normalizer.interface_name(&callback.name);
        other.methods.clear();
        for method in &callback.methods {
            let name = method.0;
            let method = method.1;
            let mut new_m = method.clone();
            new_m.name = normalizer.member_name(&method.name);
            new_m.params.clear();
            for param in &method.params {
                let mut new_p = param.clone();
                new_p.name = normalizer.parameter_name(&param.name);
                let typ = match new_p.typ.typ {
                    ConcreteType::Enumeration(name) => {
                        ConcreteType::Enumeration(normalizer.enumeration_name(&name))
                    }
                    ConcreteType::Dictionary(name) => {
                        ConcreteType::Dictionary(normalizer.dictionary_name(&name))
                    }
                    ConcreteType::Interface(name) => {
                        ConcreteType::Interface(normalizer.interface_name(&name))
                    }
                    _ => new_p.typ.typ,
                };
                new_p.typ = FullConcreteType {
                    typ,
                    arity: new_p.typ.arity,
                    extra: new_p.typ.extra.clone(),
                };
                new_m.params.push(new_p);
            }

            other.methods.insert(name.into(), new_m);
        }

        dest.callbacks.insert(name.into(), other);
    }

    dest
}

// Manages how tracked objects are kept around.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TrackedInterfaceInfo {
    interface_name: String, // The interface name.
    trait_name: String,     // The name of the trait implemented by the tracked object.
    shared: bool,           // Whether this object should be available from multiple threads or not.
    multiple: bool,         // Whether arity is not single.
}

impl TrackedInterfaceInfo {
    pub fn by_name(ast: &Ast, interface_name: &str, arity: Arity) -> Self {
        let interface = ast.interfaces.get(interface_name).unwrap();

        // Use the "rust:shared" annotation to switch to a Shared<T> tracked object.
        let shared = if let Some(annotation) = &interface.annotation {
            annotation.has("rust:shared")
        } else {
            false
        };

        // Use the first "rust:trait" annotation if specified to use as the implemented trait.
        let trait_name = if let Some(annotation) = &interface.annotation {
            let custom_traits = annotation.get_values("rust:trait");
            let send_trait = if shared { " + Send" } else { "" };
            if custom_traits.is_empty() {
                format!("{}Methods {}", interface_name, send_trait)
            } else {
                format!("{} {}", custom_traits[0], send_trait)
            }
        } else {
            format!("{}Methods", interface_name)
        };

        Self {
            interface_name: interface_name.into(),
            trait_name,
            shared,
            multiple: arity == Arity::ZeroOrMore || arity == Arity::OneOrMore,
        }
    }

    pub fn type_representation(&self) -> String {
        let mult_start = if self.multiple { "<Vec<Rc" } else { "" };
        let mult_end = if self.multiple { ">>" } else { "" };

        format!(
            "{}{}<dyn {}>{}{}",
            if self.shared { "Arc<Mutex" } else { "Rc" },
            mult_start,
            self.trait_name,
            mult_end,
            if self.shared { ">" } else { "" },
        )
    }

    pub fn interface_name(&self) -> String {
        self.interface_name.clone()
    }

    pub fn shared(&self) -> bool {
        self.shared
    }

    pub fn multiple(&self) -> bool {
        self.multiple
    }
}

#[test]
fn interface_list() {
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

        interface Event2 {
            data: str
            foo: int
        }

        interface MemberType {
            what: binary
            event interface_event -> Event2
        }


        #[service annotation]
        interface TestServiceInterface {

            #[rust_name=do_it]
            fn doIt(what: binary?, which: SomeObject) -> Kind

            some_member: MemberType
        }

        service TestService: TestServiceInterface
        "#;

    use sidl_parser::ast::Ast;

    let ast = Ast::parse_str("test", CONTENT, None).unwrap();
    let list = InterfaceList::from_ast(&ast);
    assert_eq!(
        list.get_usage_for("Kind").unwrap(),
        InterfaceUsage::Produced
    );
    assert_eq!(
        list.get_usage_for("Event2").unwrap(),
        InterfaceUsage::Produced
    );
    assert_eq!(
        list.get_usage_for("MemberType").unwrap(),
        InterfaceUsage::Both
    );
}
