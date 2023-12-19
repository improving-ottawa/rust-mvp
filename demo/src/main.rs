use uuid::Uuid;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::{Device, Id, Name};
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    let ip = local_ip_address::local_ip().unwrap();

    let port = 8787;
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("temperature");

    let sensor = TemperatureSensor::new(id, name);
    let listener = sensor.bind(ip, port, "_sensor");

    std::thread::spawn(move || {
        sensor.respond(listener);
    });

    let port = 9898;
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("temperature");

    let actuator = TemperatureActuator::new(id, name);
    let listener = actuator.bind(ip, port, "_actuator");

    std::thread::spawn(move || {
        actuator.respond(listener);
    });

    // let port = 10101;
    //
    // let controller = Controller::new();
    std::thread::spawn(move || {
        Controller::discover("_sensor");
    });

    // std::thread::spawn(move || {
    Controller::discover("_actuator");
    // });
}
