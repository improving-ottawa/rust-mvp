use actuator::{Actuator, Command};
use device::{Device, Id, Name};
use serde::{Deserialize, Serialize};

pub struct TemperatureActuator {
    id: Id,
    name: Name,
}

#[derive(Serialize, Deserialize)]
pub enum TemperatureActuatorCommand {
    SetMaxTemperature(f32),
}

impl Command for TemperatureActuatorCommand {}

impl Device for TemperatureActuator {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

impl Actuator for TemperatureActuator {
    fn act(&self, sensor: Id, command: String) {
        match serde_json::from_str::<TemperatureActuatorCommand>(&command) {
            Ok(command_enum) => match command_enum {
                TemperatureActuatorCommand::SetMaxTemperature(temp) => {
                    // TODO How do we handle this since it has no state?
                    // TODO Does this simply now call the Environment?
                    println!("Handling SetMaxTemperature: {} for Id: {}", temp, sensor);
                }
            },
            Err(e) => {
                println!("Error deserializing command: {:?}", e);
            }
        }
    }
}

impl TemperatureActuator {
    pub fn new(id: Id, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, name }
    }
}
