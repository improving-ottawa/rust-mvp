use std::sync::{Arc, Mutex};
use std::time::Duration;

use uuid::Uuid;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use controller::Controller;
use controller::ControllerExtension;
use device::{Device, Id, Name};
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    // in the local demo, all devices have the same ip (localhost)
    let ip = local_ip_address::local_ip().unwrap();

    // --------------------------------------------------------------------------------
    // spin up a sensor
    // --------------------------------------------------------------------------------

    let port = 8787;
    let temperature_sensor_id = Id::new(&Uuid::new_v4().to_string());
    let full_id = format!("temperature_{}", temperature_sensor_id);
    let name = Name::new(&full_id);

    let sensor = TemperatureSensor::new(temperature_sensor_id.clone(), name);
    let listener = sensor.bind(ip, port, "_sensor");

    std::thread::spawn(move || {
        sensor.respond(listener);
    });

    // --------------------------------------------------------------------------------
    // spin up an actuator
    // --------------------------------------------------------------------------------

    let port = 9898;
    let temperature_actuator_id = Id::new(&Uuid::new_v4().to_string());
    let full_id = format!("temperature_{}", temperature_actuator_id);
    let name = Name::new(&full_id);

    let actuator = TemperatureActuator::new(temperature_actuator_id.clone(), name);
    let listener = actuator.bind(ip, port, "_actuator");

    std::thread::spawn(move || {
        actuator.respond(listener);
    });

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let controller = Arc::new(Mutex::new(Controller::new()));
    controller.run().unwrap();

    loop {
        println!("Demo running...");
        std::thread::sleep(Duration::from_secs(5));
    }
}
