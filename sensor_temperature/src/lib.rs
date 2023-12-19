use datum::{Datum, DatumUnit};
use device::{Device, Id, Name};
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
}

impl Device for TemperatureSensor {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

// remove coverage exception ASAP
// coverage: off
impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, Some(DatumUnit::DegreesC))
    }
}
// coverage: on

impl TemperatureSensor {
    pub fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor { id, name }
    }
}

#[cfg(test)]
mod sensor_temperature_tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let id = Id::new("my id");
        let name = Name::new("my name");
        let sensor = TemperatureSensor::new(id.clone(), name.clone());
        assert_eq!(sensor.get_name(), &name);
    }

    #[test]
    fn test_get_id() {
        let id = Id::new("my id");
        let name = Name::new("my name");
        let sensor = TemperatureSensor::new(id.clone(), name.clone());
        assert_eq!(sensor.get_id(), &id);
    }
}
