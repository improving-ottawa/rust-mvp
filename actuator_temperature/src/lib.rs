use actuator::Actuator;
use device::{Device, Id, Name};

pub struct TemperatureActuator {
    id: Id,
    name: Name,
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
    fn act(sensor: Id, command: String) {
        todo!()
    }
}

impl TemperatureActuator {
    pub fn new(id: Id, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, name }
    }
}
