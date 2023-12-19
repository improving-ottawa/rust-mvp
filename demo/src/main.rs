use uuid::Uuid;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use device::{Device, Id, Name};
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    let ip = local_ip_address::local_ip().unwrap();

    let port = 8787;
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("temperature_sensor");

    let sensor = TemperatureSensor::new(id, name);
    let listener = sensor.bind(ip, port, "_sensor");

    std::thread::spawn(move || {
        sensor.respond(listener);
    });

    let port = 9898;
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("temperature_actuator");

    let actuator = TemperatureActuator::new(id, name);
    let listener = actuator.bind(ip, port, "_actuator");

    actuator.respond(listener)
}
