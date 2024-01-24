use std::{
    collections::HashMap,
    time::{
        Duration,
        SystemTime
    },
};
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CommandError {
    #[error("Invalid command")]
    InvalidCommand,
    #[error("Invalid query")]
    InvalidQuery,
    #[error("Invalid value")]
    InvalidValue,
    #[error("Invalid power state")]
    InvalidPowerState,
}

pub struct Param<'a> {
    default: &'a str,
    value: Option<String>,
    validation: Option<Regex>,
    supported_in_power_off: bool,
}

impl<'a> Param<'a> {
    fn new(default: &'a str, validation: &str, supported_in_power_off: bool) -> Param<'a> {
        let validation = if validation.len() > 0 {
            Some(Regex::new(validation).unwrap())
        } else {
            None
        };

        let value = if default.len() > 0 {
            Some(default.to_string())
        } else {
            None
        };

        Param {
            default,
            value,
            validation,
            supported_in_power_off
        }
    }

    pub fn get_value(&self) -> Result<Option<String>, CommandError> {
        if let Some(value) = &self.value {
            if value.len() > 0 {
                return Ok(Some(value.clone()));
            }
        }
        Err(CommandError::InvalidCommand)
    }

    #[inline]
    pub fn supported_in_power_off(&self) -> bool {
        self.supported_in_power_off
    }

    pub fn set_value(&mut self, value: &str) -> Result<(), CommandError> {
        if let Some(validation) = &self.validation {
            if validation.is_match(value) {
                if self.value.is_some() {
                    let result = if value == "INIT" {
                        self.default.to_string()
                    } else {
                        value.to_owned()
                    };
                    self.value = Some(result);
                }
                Ok(())
            } else {
                Err(CommandError::InvalidValue)
            }
        } else {
            Err(CommandError::InvalidCommand)
        }
    }
}

const WARMING_TIME: Duration = Duration::from_secs(5);
const COOLDOWN_TIME: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub enum PowerState {
    PowerOff,
    Warming(SystemTime),
    Cooling(SystemTime),
    LampOn,
}


impl PowerState {
    pub fn get_state(&mut self) -> PowerState {
        match self {
            // This ensures that timer effect becomes visible if expired
            PowerState::Warming(timer) => {
                if timer.elapsed().unwrap() > WARMING_TIME {
                    *self = PowerState::LampOn;
                }
            },
            PowerState::Cooling(timer) => {
                if timer.elapsed().unwrap() > COOLDOWN_TIME {
                    *self = PowerState::PowerOff;
                }
            },
            _ => (),
        }

        self.clone()
    }

    pub fn power_up(&mut self) {
        match self {
            PowerState::PowerOff => {
                *self = PowerState::Warming(SystemTime::now());
            },
            PowerState::Warming(_) => (),
            PowerState::Cooling(_) => (),
            PowerState::LampOn => (),
        }
    }

    pub fn power_down(&mut self) {
        match self {
            PowerState::PowerOff => (),
            PowerState::Warming(_) => (),
            PowerState::Cooling(_) => (),
            PowerState::LampOn => {
                *self = PowerState::Cooling(SystemTime::now());
            },
        }
    }

    pub fn as_str(&mut self) -> &'static str {
        match self.get_state() {
            PowerState::PowerOff => "00",
            PowerState::Warming(_) => "01",
            PowerState::Cooling(_) => "03",
            PowerState::LampOn => "02",
        }
    }
}




const TWO_CHARS: &str = "[A-Z0-9]{2}";
const TWO_DIGITS: &str = "\\d{2}";
const ON_OFF: &str = "(OFF|ON)";

const LAMP_HOURS_DEFAULT: &str = "100";
const AUTOHOME_DEFAULT: &str = "00";

pub struct CommandProcessor<'a> {
    commands: HashMap<&'static str, Param<'a>>,
    power_state: PowerState,
}

impl<'a> CommandProcessor<'a> {
    pub fn new() -> CommandProcessor<'a> {
        let mut processor = CommandProcessor {
            commands: HashMap::new(),
            power_state: PowerState::PowerOff,
        };

        let actual_commands = HashMap::from([
                ("SNO",Param::new("1234567890","", true)),
                //("PWR", Param::new("00", ON_OFF)),
                ("LAMP",Param::new(LAMP_HOURS_DEFAULT,"", false)),
                ("KEY", Param::new("", "[A-Z0-9]{2}|INIT", false)),
                ("AUTOHOME",Param::new(AUTOHOME_DEFAULT,TWO_CHARS, false)),
                /*
                ("SIGNAL",Param::new("01","")),
                ("ONTIME",Param::new("110","")),
                ("LAMP",Param::new("100","")),
                ("ERR",Param::new("00","")),
                ("SOURCE",Param::new("00",TWO_CHARS)),
                ("MUTE",Param::new("0000",ON_OFF)),
                ("VOL",Param::new("90","\\d+")),
                ("AUTOHOME",Param::new("00",TWO_CHARS)),
                ("ZOOM",Param::new("0","\\d{1,3}")),
                ("HREVERSE", Param::new("ON",ON_OFF)),
                ("VREVERSE", Param::new("ON", ON_OFF)),
                ("IMGSHIFT", Param::new("0 1", "-?[0-2] -?[0-2]")),
                ("REFRESHTIME", Param::new("00", TWO_DIGITS)) */
            ]);

        processor.commands = actual_commands;
        processor
    }

    fn process_power_set(&mut self, value: &str) -> Result<(), CommandError> {
        if value == "ON" {
            self.power_state.power_up();
        } else if value == "OFF" {
            self.power_state.power_down();
        } else {
            return Err(CommandError::InvalidCommand);
        }
        // This is always empty
        Ok(())
    }

    fn process_power_query(&mut self) -> Option<String> {
        Some(self.power_state.as_str().to_string())
    }

    fn process_query(&mut self, command: &str) -> Result<Option<String>, CommandError> {
        if command == "PWR" {
            Ok(self.process_power_query())
        } else {
            if let Some(param) = self.commands.get(command) {
                match (param.supported_in_power_off(), self.power_state.get_state()) {
                    (true, _) | (false, PowerState::LampOn) => param.get_value(),
                    _ => Err(CommandError::InvalidPowerState),
                }
            } else {
                Err(CommandError::InvalidCommand)
            }
        }
    }

    fn process_set(&mut self, command: &str, value: &str) -> Result<(), CommandError> {
        if command == "PWR" {
            self.process_power_set(value)
        } else {
            if let Some(param) = self.commands.get_mut(command) {
                match (param.supported_in_power_off(), self.power_state.get_state()) {
                    (true, _) | (false, PowerState::LampOn) => param.set_value(value),
                    _ => return Err(CommandError::InvalidPowerState),
                }
            } else {
                Err(CommandError::InvalidCommand)
            }
        }
    }

    pub fn process_message(&mut self, message: &str) -> Result<Option<String>, CommandError> {
        if message.ends_with("?") {
            let result = self.process_query(&message[0..message.len()-1]);
            match &result {
                Ok(Some(_)) => result,
                Ok(None) => Err(CommandError::InvalidQuery),
                // Either Ok(Some(_)) or Err(_) would have been needed to be copied. I took Err as it is rare event
                Err(err) => Err(err.clone()),
            }
        } else {
            let result = Regex::new("([A-Z][A-Z0-9]+) (.+)").unwrap().captures(message).map(|cap| {
                let command = cap.get(1).ok_or(CommandError::InvalidCommand)?;
                let value = cap.get(2).ok_or(CommandError::InvalidValue)?;

                self.process_set(command.as_str(), value.as_str())?;
                Ok(None)
            }).unwrap();
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum() {
        let mut state = PowerState::PowerOff;
        assert_eq!(state.as_str(), "00");
        state.power_up();
        assert_eq!(state.as_str(), "01");
        state.power_up();
        assert_eq!(state.as_str(), "01");
        std::thread::sleep(WARMING_TIME);
        assert_eq!(state.as_str(), "02");
        state.power_down();
        assert_eq!(state.as_str(), "03");
        state.power_down();
        assert_eq!(state.as_str(), "03");
        std::thread::sleep(COOLDOWN_TIME);
        assert_eq!(state.as_str(), "00");
        state.power_down();
        assert_eq!(state.as_str(), "00");
    }

    #[test]
    fn test_power_command() {
        let mut processor = CommandProcessor::new();
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("01".to_string()));
        std::thread::sleep(WARMING_TIME);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("02".to_string()));
        assert_eq!(processor.process_message("PWR OFF").unwrap(), None);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("03".to_string()));
        std::thread::sleep(COOLDOWN_TIME);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("00".to_string()));
    }

    #[test]
    fn test_power_state_logic() {
        let mut processor = CommandProcessor::new();
        assert_eq!(processor.process_message("SNO?").unwrap().is_some(), true);
        assert_eq!(processor.process_message("LAMP?"), Err(CommandError::InvalidPowerState));
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        assert_eq!(processor.process_message("LAMP?"), Err(CommandError::InvalidPowerState));
        std::thread::sleep(WARMING_TIME);
        assert_eq!(processor.process_message("LAMP?").unwrap(), Some(LAMP_HOURS_DEFAULT.to_string()));
    }

    #[test]
    fn test_set_get() {
        let mut processor = CommandProcessor::new();
        assert_eq!(processor.process_message("SNO?").unwrap().is_some(), true);
        assert_eq!(processor.process_message("SNO 1234567890"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        std::thread::sleep(WARMING_TIME);
        assert_eq!(processor.process_message("SNO?").unwrap(), Some("1234567890".to_string()));
        assert_eq!(processor.process_message("SNO 123456789"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("KEY 01").unwrap(), None);
        assert_eq!(processor.process_message("KEY?"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("AUTOHOME 01").unwrap(), None);
        assert_eq!(processor.process_message("AUTOHOME?").unwrap(), Some("01".to_string()));
    }
}