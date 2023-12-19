use uuid::Uuid;

use sensor::{Id, Name, Sensor};
use sensor_temperature::TemperatureSensor;

fn main() {
    let ip = local_ip_address::local_ip().unwrap();
    let port = 8787;
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("temperature_sensor");

    let sensor = TemperatureSensor::new(id, name);
    let listener = sensor.bind(ip, port);

    sensor.respond(listener)
}
