use std::{
    collections::HashMap,
    time,
};
use regex::Regex;
use thiserror::Error;

pub struct Param<'a> {
    default: &'a str,
    value: String,
    validation: Regex
}

impl<'a> Param<'a> {
    fn new(default: &'a str, validation: &str) -> Param<'a> {
        Param {
            default,
            value: default.to_string(),
            validation: Regex::new(validation).unwrap()
        }
    }

    pub fn get_value(&self) -> Option<String> {
        if self.value.len() > 0 {
            Some(self.value.clone())
        } else {
            None
        }
    }

    pub fn set_value(&mut self, value: &str) -> bool {
        if self.validation.is_match(value) {
            if value == "INIT" {
                self.value = self.default.to_string();
            } else {
                self.value = value.to_string();
            }
            true
        } else {
            false
        }
    }

    pub fn get_validation(&self) -> &Regex {
        &self.validation
    }

}

const WARMING_TIME: time::Duration = time::Duration::from_secs(5);
const COOLDOWN_TIME: time::Duration = time::Duration::from_secs(3);

#[derive(Debug, Clone)]
pub enum PowerState {
    PowerOff,
    Warming(time::SystemTime),
    Cooling(time::SystemTime),
    LampOn,
    Terminated,
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
                *self = PowerState::Warming(time::SystemTime::now());
            },
            PowerState::Warming(_) => (),
            PowerState::Cooling(_) => (),
            PowerState::LampOn => (),
            PowerState::Terminated => (),
        }
    }

    pub fn power_down(&mut self) {
        match self {
            PowerState::PowerOff => {
                *self = PowerState::Warming(time::SystemTime::now());
            },
            PowerState::Warming(_) => (),
            PowerState::Cooling(_) => (),
            PowerState::LampOn => (),
            PowerState::Terminated => (),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PowerState::PowerOff => "00",
            PowerState::Warming(_) => "01",
            PowerState::Cooling(_) => "03",
            PowerState::LampOn => "02",
            PowerState::Terminated => "99",
        }
    }
}




const TWO_CHARS: &str = "[A-Z0-9]{2}";
const TWO_DIGITS: &str = "\\d{2}";
const ON_OFF: &str = "(OFF|ON)";

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command")]
    InvalidCommand,
    #[error("Invalid query")]
    InvalidQuery,
    #[error("Invalid value")]
    InvalidValue,
}


pub struct CommandProcessor<'a> {
    commands: HashMap<&'static str, Param<'a>>,
    power_state: PowerState,
}

impl<'a> CommandProcessor<'a> {
    pub fn new() -> CommandProcessor<'a> {
        CommandProcessor {
            commands: HashMap::from([
                ("SNO",Param::new("1234567890","")),
                ("PWR", Param::new("00", ON_OFF)),
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
                ("KEY", Param::new("0", "[A-Z0-9]{2}|INIT")),
                ("HREVERSE", Param::new("ON",ON_OFF)),
                ("VREVERSE", Param::new("ON", ON_OFF)),
                ("IMGSHIFT", Param::new("0 1", "-?[0-2] -?[0-2]")),
                ("REFRESHTIME", Param::new("00", TWO_DIGITS)) */
            ]),
            power_state: PowerState::PowerOff,
        }
    }

    fn process_query(&self, command: &str) -> Option<String> {
        if let Some(param) = self.commands.get(command) {
            param.get_value()
        } else {
            None
        }
    }

    fn process_set(&mut self, command: &str, value: &str) -> Option<String> {
        let param = self.commands.get_mut(command).unwrap();
        if param.set_value(value) {
            param.get_value()
        } else {
            None
        }
    }

    // Regex match ([A-Z][A-Z0-9]+)

    pub fn process_message(&mut self, message: &str) -> Result<Option<String>, CommandError> {
        if message.ends_with("?\r") {
            let result = self.process_query(&message[0..message.len()-2]);
            if result.is_none() {
                Err(CommandError::InvalidQuery)
            } else {
                Ok(result)
            }
        } else {
            let result = Regex::new("([A-Z][A-Z0-9]+) (.+)").unwrap().captures(message).map(|cap| {
                let command = cap.get(0).ok_or(CommandError::InvalidCommand)?;
                let value = cap.get(1).ok_or(CommandError::InvalidValue)?;

                Ok(self.process_set(command.as_str(), value.as_str()))
            }).unwrap();
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}