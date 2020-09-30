# Service layer for external APIs

External APIs in KaiOS are implemented in an external daemon that communicates with the web runtime embedder. The APIs are conceptually represented by a set of services exposed over some transport protocols.

We want this system to fullfill the following goals:

- Implementers of services and clients of these services must not have to deal with details of the transport layer and of the message serialization and deserialization.
- The APIs are all asynchronous and message based with strong typing.
- Clients can call async methods and register themselves as event listeners.
- For performance reasons, properties can be cached transparently on the client side.
- Several transports can be supported simultaneously.

## Representation of services APIs

We use a custom service definition language to define services. The language provides basic types along with custom types definition.

### Examples:

```
// The FM Radio service
Service FmRadio {

    // Search direction.
    enum SearchDirection {
        Up,
        Down,
    }

    // Parameters for the search method.
    type SearchParams {
        freq: float,
        dir: SearchDirection,
        timeout: int,
    }

    property enabled: bool;
    #[client_name=antennaAvailable]
    property antenna_available: bool;
    property freq: float;

    event frequencychange -> float;
    event enable;
    event disable;

    fn enable(float);
    fn disable();
    fn search(SearchParams);
    fn cancel_search();
    fn set_freq(float);
}

```

```
import common_types;

// The Settings service
Service Settings {
    type Setting {
        name: str,
        value: any,
    }

    type Lock {
        fn set(Setting+);
        fn get(str) -> Setting;
    }

    event settingchange -> Setting;
    event observe(str) -> Setting;

    fn create_lock() -> Lock;
}

```

### Default types

- bool
- int
- float
- str
- binary
- json
- any

Modifiers are used to change the arity of a type: `*` for zero or more, `+` for one or more and `?` for zero or one.

### Annotations

Code generation can be customized by using annotations attached to declaration. The format for annotations is `#[name1=value1,name2=value2]`.

## Service implementation

The canonical implementation language for services is Rust. All services are registered on a thread, but can spawn their own ones. This allows direct communication between services without bouncing back on the 'main' thread by passing around a mpsc channel sender.

For instance, services can listen on services changes with low overhead.

The code generator creates a Trait and all the private types for each service in its own module.

## Client side implementation

The code generator will generate JS code following best front-end approaches.

## Low level protocol format

We use the bitsparrow library to exchange data between the Rust and JS sides.