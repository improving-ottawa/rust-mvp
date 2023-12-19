use actix_web::web::Data;
use actix_web::{get, web, App, HttpServer};
use libmdns::Responder;
use uuid::Uuid;

use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sensor_id = Uuid::new_v4().to_string();
    let sensor_name = "temperature_sensor";

    let port: u16 = 13579;

    let host = local_ip_address::local_ip().unwrap();
    let host = host.clone().to_string();

    let address = format!("{}:{}", host, port);

    let responder = Responder::new().unwrap();

    let endpoint = "/datum";
    let txt = format!("path={}", endpoint);

    let _service = responder.register(
        "_sensor._tcp".to_owned(),
        String::from(sensor_name),
        port,
        &[txt.as_str()],
    );

    #[get("/datum")]
    async fn get_datum(data: web::Data<TemperatureSensor>) -> String {
        let datum = &data.get_datum(); // <- get app_name
        datum.to_string() // <- response with app_name
    }

    HttpServer::new(move || {
        let sensor = TemperatureSensor::new(sensor_id.as_str(), sensor_name);

        let address = format!("{}:{}", host, port);
        println!("\nCreating new sensor {} at {}", sensor_name, address);
        println!(
            "Ask this sensor for data with: curl {}{}",
            address, endpoint
        );

        App::new().app_data(Data::new(sensor)).service(get_datum)
    })
    .bind(address)?
    .workers(1) // sensors should be single-threaded
    .run()
    .await
}
