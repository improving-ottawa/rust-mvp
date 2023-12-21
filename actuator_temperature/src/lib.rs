use actuator::{Actuator, Command};
use device::{Device, Id, Name};
use serde::{Serialize, Deserialize};

pub struct TemperatureActuator {
    id: Id,
    name: Name,
}

#[derive(Serialize, Deserialize)]
pub enum TemperatureActuatorCommand {
    SetMaxTemperature(f32)
}

impl Command for TemperatureActuatorCommand {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}


impl Device for TemperatureActuator {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

#[allow(unused_variables)] // remove ASAP
impl Actuator for TemperatureActuator {
    fn act(&self, sensor: Id, command: String) {

        match serde_json::from_str::<TemperatureActuatorCommand>(&command) {
            Ok(command_enum) => {
                match command_enum {
                    TemperatureActuatorCommand::SetMaxTemperature(temp) => {
                        println!("Handling SetMaxTemperature: {}", temp);
                    }
                }

            }
            Err(e) => {
                println!("Error serializing command to TemperatureActuatorCommand");
            }
        }
    }
}

impl TemperatureActuator {
    pub fn new(id: Id, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, name }
    }
}
