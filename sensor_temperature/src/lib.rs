use datum::{Datum, DatumUnit};
use sensor::{Id, Name, Sensor};

pub struct TemperatureSensor {
    id: Id,
    name: Name,
}

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, Some(DatumUnit::DegreesC))
    }

    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor { id, name }
    }
}
