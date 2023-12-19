use std::env;
use actix_web::{App, get, HttpResponse, HttpServer, Responder as ActixResponder};
use libmdns::Responder;
use uuid::Uuid;
use datum::{Datum, DatumUnit, DatumValue};
use sensor::{Id, Name, Sensor};


struct TemperatureSensor {

}

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        Datum::new_now(
            DatumValue::Float(25.0),
            Some(DatumUnit::DegreesC)
        )
    }

    fn get_name(&self) -> Name {
        Name::new("TemperatureSensor")
    }

    fn get_id(&self) -> Id {
        Id::new(Uuid::new_v4().to_string().as_str())
    }
}


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

    HttpServer::new(|| {
        App::new().service(get_datum)
    })
        .bind(sensor_addr)?
        .run()
        .await
}

#[get("/datum")]
async fn get_datum() -> impl ActixResponder {
    HttpResponse::Ok().body("Sensor Datum")
}