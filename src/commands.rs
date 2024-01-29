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

    pub fn get_value(&self) -> Result<String, CommandError> {
        if let Some(value) = &self.value {
            if value.len() > 0 {
                return Ok(value.clone());
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


#[derive(Debug, Clone)]
pub enum PowerState {
    PowerOff,
    Warming(SystemTime),
    Cooling(SystemTime),
    LampOn,
}


impl PowerState {
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

    pub fn as_str(&self) -> &'static str {
        match self {
            PowerState::PowerOff => "00",
            PowerState::Warming(_) => "02",
            PowerState::Cooling(_) => "03",
            PowerState::LampOn => "01",
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
    warming: Duration,
    cooling: Duration
}

impl<'a> CommandProcessor<'a> {
    pub fn new(warming: u64, cooling: u64) -> CommandProcessor<'a> {
        let mut processor = CommandProcessor {
            commands: HashMap::new(),
            power_state: PowerState::PowerOff,
            warming: Duration::from_secs(warming),
            cooling: Duration::from_secs(cooling)
        };

        let actual_commands = HashMap::from([
                ("SNO",Param::new("1234567890","", true)),
                //("PWR", Param::new("00", ON_OFF)),
                ("LAMP",Param::new(LAMP_HOURS_DEFAULT,"", false)),
                ("KEY", Param::new("", "[A-Z0-9]{2}|INIT", false)),
                ("FREEZE", Param::new("OFF", ON_OFF, false)),
                ("FASTBOOT", Param::new("01", TWO_DIGITS, false)),
                ("AUTOHOME",Param::new(AUTOHOME_DEFAULT,TWO_CHARS, false)),
                ("SIGNAL",Param::new("01","", false)),
                ("ONTIME",Param::new("110","", false)),
                ("LAMP",Param::new("100","", false)),
                ("ERR",Param::new("00","", true)),
                ("SOURCE",Param::new("00",TWO_CHARS, false)),
                ("MUTE",Param::new("0000",ON_OFF, false)),
                ("VOL",Param::new("90","\\d+", false)),
                ("AUTOHOME",Param::new("00",TWO_CHARS, false)),
                ("ZOOM",Param::new("0","\\d{1,3}", false)),
                ("HREVERSE", Param::new("ON",ON_OFF, false)),
                ("VREVERSE", Param::new("ON", ON_OFF, false)),
                ("IMGSHIFT", Param::new("0 1", "-?[0-2] -?[0-2]", false)),
                ("REFRESHTIME", Param::new("00", TWO_DIGITS, false))
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

    fn process_power_query(&mut self) -> &'static str {
        &self.get_power_state().as_str()
    }

    fn get_power_state(&mut self) -> PowerState {
        match self.power_state {
            // This ensures that timer effect becomes visible if expired
            PowerState::Warming(timer) => {
                if timer.elapsed().unwrap() > self.warming {
                    println!("Warm up complete");
                    self.power_state = PowerState::LampOn;
                } else {
                    println!("Warming: {:?}", timer.elapsed().unwrap());
                }

            },
            PowerState::Cooling(timer) => {
                if timer.elapsed().unwrap() > self.cooling {
                    self.power_state = PowerState::PowerOff;
                    println!("Cool down complete");
                } else {
                    println!("Cooling: {:?}", timer.elapsed().unwrap());
                }
            },
            _ => (),
        }
        self.power_state.clone()
    }

    fn process_query(&mut self, command: &str) -> Result<String, CommandError> {
        let value = if command == "PWR" {
            Ok(self.process_power_query().to_string())
        } else {
            let power_state = self.get_power_state();
            if let Some(param) = self.commands.get(command) {
                match (param.supported_in_power_off(), power_state) {
                    (true, _) | (false, PowerState::LampOn) => param.get_value(),
                    _ => Err(CommandError::InvalidPowerState),
                }
            } else {
                Err(CommandError::InvalidCommand)
            }
        }?;
        Ok(format!("{command}={value}"))
    }

    fn process_set(&mut self, command: &str, value: &str) -> Result<(), CommandError> {
        if command == "PWR" {
            self.process_power_set(value)
        } else {
            let power_state = self.get_power_state();

            if let Some(param) = self.commands.get_mut(command) {
                match (param.supported_in_power_off(), power_state) {
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
            let result = self.process_query(&message[0..message.len()-1])?;
            Ok(Some(result))
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

    const WARMING_TIME: u64 = 2;
    const COOLDOWN_TIME: u64 = 1;

    #[test]
    fn test_power_command() {
        let mut processor = CommandProcessor::new(WARMING_TIME, COOLDOWN_TIME);
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("PWR=02".to_string()));
        std::thread::sleep(Duration::from_secs(WARMING_TIME));
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("PWR=01".to_string()));
        assert_eq!(processor.process_message("PWR OFF").unwrap(), None);
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("PWR=03".to_string()));
        std::thread::sleep(Duration::from_secs(COOLDOWN_TIME));
        assert_eq!(processor.process_message("PWR?").unwrap(), Some("PWR=00".to_string()));
    }

    #[test]
    fn test_power_state_logic() {
        let mut processor = CommandProcessor::new(WARMING_TIME, COOLDOWN_TIME);
        assert_eq!(processor.process_message("SNO?").unwrap().is_some(), true);
        assert_eq!(processor.process_message("LAMP?"), Err(CommandError::InvalidPowerState));
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        assert_eq!(processor.process_message("LAMP?"), Err(CommandError::InvalidPowerState));
        std::thread::sleep(Duration::from_secs(WARMING_TIME));
        assert_eq!(processor.process_message("LAMP?").unwrap(), Some(format!("LAMP={LAMP_HOURS_DEFAULT}")));
    }

    #[test]
    fn test_set_get() {
        let mut processor = CommandProcessor::new(WARMING_TIME, COOLDOWN_TIME);
        assert_eq!(processor.process_message("SNO?").unwrap().is_some(), true);
        assert_eq!(processor.process_message("SNO 1234567890"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("PWR ON").unwrap(), None);
        std::thread::sleep(Duration::from_secs(WARMING_TIME));
        assert_eq!(processor.process_message("SNO?").unwrap(), Some("SNO=1234567890".to_string()));
        assert_eq!(processor.process_message("SNO 123456789"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("KEY 01").unwrap(), None);
        assert_eq!(processor.process_message("KEY?"), Err(CommandError::InvalidCommand));
        assert_eq!(processor.process_message("AUTOHOME 01").unwrap(), None);
        assert_eq!(processor.process_message("AUTOHOME?").unwrap(), Some("AUTOHOME=01".to_string()));
    }
}