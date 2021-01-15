use crate::channel::{Receiver, Sender};
use actix_web::{dev::Factory, web, App, FromRequest, HttpServer, Responder, Result};
use futures::{executor::block_on, Future};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    rc::Rc,
};

pub struct TestServer {
    instance: Rc<actix_web::dev::Server>,
    pub requests: Rc<Receiver>,
    socket: Rc<SocketAddr>,
}

impl TestServer {
    pub fn stop(&self) {
        block_on(self.instance.stop(false));
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.socket.to_string())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.stop()
    }
}

pub fn new<A, F, R, T, U>(addr: A, func: F) -> Result<TestServer, anyhow::Error>
where
    A: ToSocketAddrs + 'static + Send + Copy,
    F: Factory<T, R, U> + 'static + Send + Copy,
    T: FromRequest + 'static,
    R: Future<Output = U> + 'static,
    U: Responder + 'static,
{
    let (tx, rx) = crossbeam_channel::unbounded();
    let (tx_req, rx_req) = crossbeam_channel::unbounded();

    let _ = ::std::thread::spawn(move || {
        let sys = actix_rt::System::new("test-server");
        let server = HttpServer::new(move || {
            App::new()
                .wrap(Sender::new(tx_req.clone()))
                .default_service(web::route().to(func))
        })
        .bind(addr)
        .expect("Failed to bind!");

        let sockets = server.addrs();
        let instance = server.shutdown_timeout(1).run();
        let _ = tx.send((instance, sockets));

        sys.run()
    });

    let (server, sockets) = rx.recv()?;
    let socket = sockets
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Failed to get socket addr!"))?;

    Ok(TestServer {
        instance: Rc::new(server),
        requests: Rc::new(Receiver {
            rx: Rc::new(rx_req),
        }),
        socket: Rc::new(*socket),
    })
}
