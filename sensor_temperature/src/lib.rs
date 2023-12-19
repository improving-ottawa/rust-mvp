use actix_web::{get, HttpResponse, Responder};
use uuid::Uuid;

use datum::{Datum, DatumUnit};
use sensor::{Id, Name, Sensor};

struct TemperatureSensor {}

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        Datum::new_now(25.0, Some(DatumUnit::DegreesC))
    }

    fn get_name(&self) -> Name {
        Name::new("TemperatureSensor")
    }

    fn get_id(&self) -> Id {
        Id::new(Uuid::new_v4().to_string().as_str())
    }
}

#[get("/datum")]
pub async fn get_datum() -> impl Responder {
    HttpResponse::Ok().body("Sensor Datum")
}
