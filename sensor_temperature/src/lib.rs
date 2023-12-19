use datum::{Datum, DatumUnit};
use sensor::{Id, Name, Sensor};

pub struct TemperatureSensor {
    id: String,
    name: String,
}

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        Datum::new_now(25.0, Some(DatumUnit::DegreesC))
    }

    fn get_name(&self) -> Name {
        Name::new(self.name.as_str())
    }

    fn get_id(&self) -> Id {
        Id::new(self.id.as_str())
    }
}

impl TemperatureSensor {
    pub fn new(id: &str, name: &str) -> TemperatureSensor {
        TemperatureSensor {
            id: String::from(id),
            name: String::from(name),
        }
    }
}
