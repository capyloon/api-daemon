# Service Interface Description Language

SIDL is a high level language that let you define the public interface of a service. A service is described in terms of interfaces that expose functions and events, with typed parameters and return types.

SIDL is language and transport agnostic: mapping to different languages and transport mechanisms (eg. WebSocket, Unix pipes, Bluetooth) is possible but is not described in this document. In general, binding developers should try to expose an API as idiomatic as possible for the target.

There is no explicit notion of client and server roles. Any participant in a session can provide services to the peer it is connected to and use the peer services. A service can also maintain active sessions with several peers, and broadcast events to a subset or all of them.

Transport layers are responsible for implementing the appropriate authentication and authorization layers as needed.

# A sample service

To explain the concepts and syntax, we will walk through this sample service:

```
// A simple enumeration.
enum DictType {
    First
    Second
}

// Dictionaries are bundles of properties.
dictionary SomeDictionary {
    type: DictType
    #[js_name="dict_name"]
    name: str
}

// Interfaces group functions and events.
interface TopLevelInterface {
    value: DictType

    fn do_something(param1: str, param2: int) -> binary, bool

    fn create_dict(id: str) -> SomeDictionary

    even data_changed -> float
}

// A service definition, implementing a "root" interface.
service TestService: TopLevelInterface
```

# Defining a service

From the example:
```
service TestService: TopLevelInterface
```

A service declaration starts by the keyword `service` followed by the service name, a `:` character and the name of the interface implemented by the service. Camelcase is the preferred naming convention.

In our case, a service named `TestService` will implement the `TopLevelInterface` interface when instanciated. You can define as many services as you like a single sidl file, and libraries are expected to provide ways to get back instances of services.

A service instance can be a singleton or a fully new instance. This is an implementation choice that is left to the service implementer and should not be observable from the peer.

# Interfaces

We just saw that a service is actually just providing the implementation of an interface. Interfaces in sidl are used to specify functions or data access implemented by an object and events that can be emitted. This means that different objects can implement the same interface, even when being used by the same service: the peer getting using the interface doesn't have to worry about the concrete implementation of the interface.

# Functions

We have a couple of functions in the TopLevelInterface. The simplest one is:
```
fn create_dict(id: str) -> SomeDictionary
```
A function declaration starts with the keyword `fn` followed by the function name. You then need to specify the list of parameters (which can be empty) and the return type.

Parameters are declared by the parameter name, a `:` and the parameter type. Multiple parameters are separated by a `,`.

The default return type is `void` but when you need to return another type, this is done by adding `->` and the return type.

# Events

Events are first class built in features in SIDL, similar to data sources. They are declared with the `event` keyword followed by the event name, and optionally by the return type if its not void. Event return types are used to specify the kind of data emitted by the event.

Each event must be part of an interface declaration, and is tied to a specific instance of the interface.

In our example:
```
even data_changed -> float
```
the `data_changed` event will emit a floating point number when it fires.

The SIDL language itself doesn't define how a peer starts and stops to listen to an event. This is let to the implementation of the core messaging and to the language bindings.

# Types

We've seen several types used when discussing functions and events. Every time a type is expected, you can use one of the following:
- a builtin primitive type.
- an interface.
- a dictionary.
- an enumeration.

Each type also has an arity, which is unary by default but can also be:
- optional (0 or 1 occurence) if you prepend `?` to the type.
- one or more if you prepend `+`to the type.
- zero or more if you prepend `*` to the type.

## Builtin primitive types

This is the set of types that are directly available:

- void: an "empty" type.
- bool: a boolean value.
- int: a signed integer.
- float: a floating point number.
- str: a string. The encoding is defined by the language bindings (eg. utf-8 in Rust, utf-16 in Javascript).
- binary: an array of bytes.

## Interface types

When you use an interface as a type, this means that you expect an object implementing this interface.

# Dictionaries

Dictionaries are similar to `structs` in languages like C. They are a way to bundle together members of varying types.

They are declared with the `dictionary` keyword followed by the dictionary name, and `{`. Then comes the list of members, each one being the member name, `:` and the member type. Finally, close the dictionary with `}`. Each member name must be unique.

# Enumerations

Enumerations in SIDL are similar to C ones, and let you express a typed, finite set of values.

They are declated with the `enum` keyword followed by the the enumeration name, and `{`. Then comes the list of possible values, and finally `}`. Each value must be unique.

# Return types

Function return types can convey two values: a success value, and an error value. This provide first class support for consistent error management.

From a syntax point of view, a full declaration is made of the success type, a `,`, and the error type. To simplify writing sidl files, the default return type is actually `<void, void>` if none is specified. It is also possible to only specify the success type, in which case the error type will also be `void`.

# Imports

To enable sharing of interfaces, dictionaries and enumerations definitions, it is possible to break up your service definitions in different files and import them where you want to use them.

The syntax used is composed of the `import` keyword followed by the path to the file that will be imported as a litteral string (ie. a string delimited by `"`).

# Comments

Comments start with `//` and end at the end of the line.

# Annotations

Most declarations can be prepended by an annotation declaration. These declarations are meant to be used as hints by tools making use of the sidl files, like code generators.

Annotations are declared by the sequence `#[` followed by the annotation content which is free form and closed by `]`.

