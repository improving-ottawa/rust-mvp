use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum Command {
    CoolTo(f32), // the Controller tells the Actuator to cool the Environment to 'x' degrees C
    HeatTo(f32), // the Controller tells the Actuator to heat the Environment to 'x' degrees C
}

impl actuator::Command for Command {}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::CoolTo(temp) => write!(f, "CoolTo:{}", temp),
            Command::HeatTo(temp) => write!(f, "HeatTo:{}", temp),
        }
    }
}

impl Command {
    pub fn parse(string: &str) -> Result<Command, String> {
        let mut pieces = string.split(':');

        match (pieces.next(), pieces.next()) {
            (Some("CoolTo"), Some(temp)) => match temp.parse() {
                Ok(temp) => Ok(Command::CoolTo(temp)),
                Err(_) => Err(format!("cannot parse {} as f32", temp)),
            },
            (Some("HeatTo"), Some(temp)) => match temp.parse() {
                Ok(temp) => Ok(Command::HeatTo(temp)),
                Err(_) => Err(format!("cannot parse {} as f32", temp)),
            },
            _ => Err(format!("cannot parse {} as Command", string)),
        }
    }
}

#[cfg(test)]
mod actuator_temperature_command_tests {
    use super::*;

    fn serde(command: &Command) -> Result<Command, String> {
        let serialized = command.to_string();
        Command::parse(serialized.as_str())
    }

    #[test]
    fn test_serde_cool_to() {
        let command = Command::CoolTo(42.0);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }

    #[test]
    fn test_serde_heat_to() {
        let command = Command::HeatTo(19.3);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }
}
