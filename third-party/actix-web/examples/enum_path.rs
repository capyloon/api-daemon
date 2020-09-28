use actix_web::{get, middleware, web, App, HttpServer};
use serde::Deserialize;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum MyThing {
    String(String),
    Int(u32),
}

#[get("/res/{name}")]
async fn index(name: web::Path<MyThing>) -> String {
    println!("REQ: {:?}", name);

    "Hello".to_owned()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .workers(1)
    .run()
    .await
}
