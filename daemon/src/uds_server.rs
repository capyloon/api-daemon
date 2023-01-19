/// Unix domain server endpoint.
/// It shares the same session implementation as the WebSocket endpoint.
use crate::api_server::TelemetrySender;
use crate::global_context::GlobalContext;
use crate::session::Session;
use common::frame::Frame;
use common::traits::{IdFactory, MessageKind, MessageSender, StdSender};
use log::{error, info};
use nix::sys::stat::{fchmodat, FchmodatFlags, Mode};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::thread;

fn handle_client(
    mut stream: UnixStream,
    context: GlobalContext,
    session_id: u32,
    #[allow(unused_variables)] telemetry: TelemetrySender,
) {
    // We will need a stream handle copy in our sending thread.
    let mut base_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed to clone uds stream: {}", err);
            return;
        }
    };

    // Our channel to receive data from the session.
    let (sender, receiver) = mpsc::channel();
    let sender2 = sender.clone();

    // Create a new session attached to this stream.
    let mut session = Session::uds(
        session_id,
        &context.config,
        MessageSender::new(Box::new(StdSender::new(&sender))),
        context.tokens_manager,
        context.session_context,
        context.remote_service_manager,
    );

    // Launch a thread to receive data from the session.
    thread::spawn(move || {
        let stream_closer = match base_stream.try_clone() {
            Ok(stream) => stream,
            Err(err) => {
                error!("Failed to clone uds stream: {}", err);
                return;
            }
        };

        loop {
            match receiver.recv() {
                Ok(MessageKind::Data(_tracker_id, data)) => {
                    // TODO: check if we need a BufWriter for performance reasons.
                    if let Err(err) = Frame::write_to(&data, &mut base_stream) {
                        error!("Failed to send data: {}", err);
                    }
                }
                Ok(MessageKind::ChildDaemonCrash(name, exit_code, pid)) => {
                    error!(
                        "Child daemon `{}` (pid {}) died with exit code {}, closing uds connection",
                        name, pid, exit_code
                    );

                    #[cfg(feature = "device-telemetry")]
                    telemetry.send(&format!("child-{}", name), exit_code, pid);

                    break;
                }
                Ok(MessageKind::Close) => {
                    info!("Closing uds connection.");
                    break;
                }
                Err(err) => {
                    error!("Failed to receive message on uds thread: {}", err);
                    break;
                }
            }
        }
        // We are done, let's close the socket.
        stream_closer
            .shutdown(Shutdown::Both)
            .expect("Failed to close uds stream");
    });

    // Receive messages and forward them to the session.
    loop {
        // Since the session does the decoding to distinguish between the
        // handshake and regular messages, we don't try to interpret the
        // data here.
        // TODO: check if we need a BufReader for performance reasons.
        match Frame::read_from(&mut stream) {
            Ok(data) => {
                session.on_message(&data);
            }
            Err(err) => {
                // This is not really an error since it will happen when the
                // stream is shutdown from the client side.
                info!("Failed to read frame: {}, closing uds session.", err);
                break;
            }
        }
    }

    // Make sure the stream is closed and the reading thread stops.
    let _ = sender2.send(MessageKind::Close);
}

pub fn start(run_context: &GlobalContext, telemetry: TelemetrySender) {
    if run_context.config.general.socket_path.is_none() {
        info!("No socket path configured.");
        return;
    }

    let path = run_context.config.general.socket_path.as_ref().unwrap();

    // Make sure the listener doesn't already exist.
    let _ = ::std::fs::remove_file(path);

    match UnixListener::bind(path) {
        Ok(listener) => {
            // Temporary fix for https://bugzilla.kaiostech.com/show_bug.cgi?id=98577
            // until we use a proper way to allow access to the uds socket from content
            // processes.
            if let Err(err) = fchmodat(
                None,
                path.as_str(),
                Mode::all(),
                FchmodatFlags::FollowSymlink,
            ) {
                error!("Failed to chmod uds socket at {} : {}", path, err);
            }

            let mut session_id_factory = IdFactory::new(0);
            for stream in listener.incoming() {
                let session_id = session_id_factory.next_id();
                match stream {
                    Ok(stream) => {
                        let context = run_context.clone();
                        #[allow(clippy::unit_arg)]
                        thread::spawn(move || {
                            handle_client(stream, context, session_id as u32, telemetry)
                        });
                    }
                    Err(err) => {
                        error!("Failure getting uds client: {}", err);
                        break;
                    }
                }
            }
        }
        Err(err) => error!("Failed to bind socket at {} : {}", path, err),
    }
}

#[cfg(test)]
mod test {
    use super::start;
    use crate::global_context::GlobalContext;
    use common::frame::Frame;
    use common::traits::OriginAttributes;
    use std::collections::HashSet;
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;
    use std::thread;

    fn start_uds(id: &str) -> String {
        let mut config = crate::config::Config::test_on_port(8080);
        let path = format!("/tmp/api-daemon-socket-{}", id);
        config.general.socket_path = Some(path.clone());
        let uds_context = GlobalContext::new(&config);

        // Register a handshake token for the tests.
        uds_context.tokens_manager.lock().register(
            &format!("test-token-{}", id),
            OriginAttributes::new("test-identity", HashSet::new()),
        );

        // Start a uds endpoint.
        let _uds_handle = thread::Builder::new()
            .name("uds server".into())
            .spawn(move || {
                start(&uds_context, ());
            })
            .expect("Failed to start uds server thread");

        // Let the uds time to start up.
        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        path
    }

    #[test]
    fn uds_has_service() {
        use bincode::Options;
        use common::core::*;

        // Connect to the uds endpoint.
        let mut stream = UnixStream::connect(start_uds("has-service")).unwrap();

        // Build a `has_service` call.
        let request = CoreRequest::HasService(HasServiceRequest {
            name: "SettingsManager".into(),
        });

        // Wrap it in a base message and send it.
        let base = BaseMessage {
            service: 0,
            object: 0,
            kind: BaseMessageKind::Request(1),
            content: common::get_bincode().serialize(&request).unwrap(),
        };

        Frame::serialize_to(&base, &mut stream).unwrap();

        // Read the base message in response.
        let base: BaseMessage = Frame::deserialize_from(&mut stream).unwrap();
        assert_eq!(base.response(), 1);
        // Decode the response
        let response: CoreResponse = common::deserialize_bincode(&base.content).unwrap();
        match response {
            CoreResponse::HasService(r) => assert!(r.success),
            _ => panic!("Cannot deserialize HasService response"),
        }

        // Close the session.
        stream.shutdown(Shutdown::Both).unwrap();
    }
}
