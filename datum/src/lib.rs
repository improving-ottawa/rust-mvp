use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};

/// A `Datum` is a singular data point; a single measurement / observation of some `Attribute`.
///
/// It contains a typed `value`, a `unit` associated with that `value`, and a `timestamp`.
///
/// Note that it is not generically-typed (no `T` parameter). Data is communicated across HTTP / TCP
/// and is consumed by a frontend HTML app, so we will lose type safety at those interfaces. Storing
/// these data points in `Datum` structs anticipates this complication and tries to tackle it head-on.
pub struct Datum {
    pub value: DatumValue,
    pub unit: Option<DatumUnit>,
    pub timestamp: DateTime<Utc>,
}

impl Display for Datum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}@{}",
            self.value,
            self.unit.unwrap_or_default(),
            self.timestamp.to_rfc3339()
        )
    }
}

#[allow(dead_code)] // remove ASAP
impl Datum {
    fn get_as_bool(&self) -> Option<bool> {
        match self.value {
            DatumValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    fn get_as_float(&self) -> Option<f32> {
        match self.value {
            DatumValue::Float(value) => Some(value),
            _ => None,
        }
    }

    fn get_as_int(&self) -> Option<i32> {
        match self.value {
            DatumValue::Int(value) => Some(value),
            _ => None,
        }
    }

    // TODO add other 'get_as_x' methods here as necessary
}

impl From<bool> for DatumValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f32> for DatumValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for DatumValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DatumValue {
    Bool(bool),
    Float(f32),
    Int(i32),
}

impl Display for DatumValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            DatumValue::Bool(value) => value.to_string(),
            DatumValue::Float(value) => value.to_string(),
            DatumValue::Int(value) => value.to_string(),
        };

        write!(f, "{}", string)
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum DatumUnit {
    #[default]
    Unitless,
    PoweredOn,
    DegreesC,
}

impl Display for DatumUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            DatumUnit::Unitless => "",
            DatumUnit::PoweredOn => "⏼",
            DatumUnit::DegreesC => "°C",
        };

        write!(f, "{}", string)
    }
}

impl Datum {
    pub fn new<T: Into<DatumValue>>(
        value: T,
        unit: Option<DatumUnit>,
        timestamp: DateTime<Utc>,
    ) -> Datum {
        Datum {
            value: value.into(),
            unit,
            timestamp,
        }
    }

    pub fn new_now<T: Into<DatumValue>>(value: T, unit: Option<DatumUnit>) -> Datum {
        Datum::new(value, unit, Utc::now())
    }
}

#[cfg(test)]
mod datum_tests {
    use super::*;

    fn create<T: Into<DatumValue>>(value: T) -> Datum {
        Datum::new(value, None, Utc::now())
    }

    #[test]
    fn test_create_datum_bool() {
        let datum = create(true);
        assert_eq!(datum.get_as_bool(), Some(true));
    }

    #[test]
    fn test_create_datum_bool_failure() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_bool(), None);
    }

    #[test]
    fn test_create_datum_float() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_float(), Some(42.0));
    }

    #[test]
    fn test_create_datum_float_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_float(), None);
    }

    #[test]
    fn test_create_datum_int() {
        let datum = create(19);
        assert_eq!(datum.get_as_int(), Some(19));
    }

    #[test]
    fn test_create_datum_int_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_int(), None);
    }

    #[test]
    fn test_datum_display() {
        let now = Utc::now();
        let now_as_str = now.to_rfc3339();

        let datum = Datum::new(true, None, now);
        let expected = format!("true@@{}", now_as_str);
        assert_eq!(datum.to_string(), expected);

        let datum = Datum::new(42.3, Some(DatumUnit::PoweredOn), now);
        let expected = format!("42.3@⏼@{}", now_as_str);
        assert_eq!(datum.to_string(), expected);

        let datum = Datum::new(19, Some(DatumUnit::DegreesC), now);
        let expected = format!("19@°C@{}", now_as_str);
        assert_eq!(datum.to_string(), expected);
    }
}
