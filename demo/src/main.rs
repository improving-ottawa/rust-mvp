use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    let temperature_sensor_id = Id::new(&Uuid::new_v4().to_string());
    let full_id = format!("temperature_{}", temperature_sensor_id);
    let name = Name::new(&full_id);

    let sensor = TemperatureSensor::new(temperature_sensor_id.clone(), name);
    let listener = sensor.bind(ip, port, "_sensor");

    std::thread::spawn(move || {
        sensor.respond(listener);
    });

    let port = 9898;
    let temperature_actuator_id = Id::new(&Uuid::new_v4().to_string());
    let full_id = format!("temperature_{}", temperature_actuator_id);
    let name = Name::new(&full_id);

    let actuator = TemperatureActuator::new(temperature_actuator_id.clone(), name);
    let listener = actuator.bind(ip, port, "_actuator");

    std::thread::spawn(move || {
        actuator.respond(listener);
    });

    let controller = Arc::new(Mutex::new(Controller::new()));

    // Spawn a looping thread to continuously check for newly connected devices
    let discovery_ctrl = controller.clone();
    std::thread::spawn(move || loop {
        {
            discovery_ctrl.lock().unwrap().discover("_sensor").unwrap();
            discovery_ctrl
                .lock()
                .unwrap()
                .discover("_actuator")
                .unwrap();
        }
        std::thread::sleep(Duration::from_secs(5));
    });

    // A testing loop where we lock the controller, call the api, release lock, sleep and loop
    let api_ctrl = controller.clone();
    loop {
        {
            let controller = api_ctrl.lock().expect("failed to lock");

            let address = controller.get_device_address(temperature_sensor_id.clone());

            match address {
                Ok(address) => {
                    Controller::read_sensor(address.as_str()).unwrap();
                }
                Err(msg) => println!("{}", msg),
            }
        }
        std::thread::sleep(Duration::from_secs(2));

        {
            let controller = api_ctrl.lock().expect("failed to lock");

            let address = controller.get_device_address(temperature_actuator_id.clone());

            match address {
                Ok(address) => {
                    Controller::command_actuator(address.as_str()).unwrap();
                }
                Err(msg) => println!("{}", msg),
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
