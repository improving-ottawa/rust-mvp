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

// remove coverage exception ASAP
// coverage: off
#[allow(unused_variables)] // remove ASAP
impl Actuator for TemperatureActuator {
    fn act(sensor: Id, command: String) {
        todo!()
    }
}
// coverage: on

impl TemperatureActuator {
    pub fn new(id: Id, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, name }
    }
}

#[cfg(test)]
mod actuator_temperature_tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let id = Id::new("my id");
        let name = Name::new("my name");
        let actuator = TemperatureActuator::new(id.clone(), name.clone());
        assert_eq!(actuator.get_name(), &name);
    }

    #[test]
    fn test_get_id() {
        let id = Id::new("my id");
        let name = Name::new("my name");
        let actuator = TemperatureActuator::new(id.clone(), name.clone());
        assert_eq!(actuator.get_id(), &id);
    }
}
