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

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, Some(DatumUnit::DegreesC))
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor { id, name }
    }
}
