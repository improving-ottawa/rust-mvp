use std::env;

use actix_web::{App, HttpServer};
use libmdns::Responder;

use sensor_temperature::get_datum;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sensor_port = env::var("PORT").unwrap_or(String::from("8080"));
    let sensor_name = env::var("NAME").unwrap_or(String::from("unnamed"));

    let sensor_addr = format!("0.0.0.0:{}", sensor_port);

    let responder = Responder::new().unwrap();
    let _service = responder.register(
        "_sensor._tcp".to_owned(),
        sensor_name.to_owned(),
        sensor_port.parse().unwrap(),
        &["path=/datum"],
    );

    HttpServer::new(|| App::new().service(get_datum))
        .bind(sensor_addr)?
        .run()
        .await
}