// a boolean type with the values "On" and "Off"

use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, ValueEnum, Eq, PartialEq, Hash)]
pub enum OnOffType {
    On,
    Off,
}

impl OnOffType {
    pub fn from_string(value: &str) -> Self {
        match value {
            "0" => OnOffType::Off,
            _ => OnOffType::On,
        }
    }

    pub fn flipped(&self) -> Self {
        match self {
            OnOffType::On => OnOffType::Off,
            OnOffType::Off => OnOffType::On,
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            OnOffType::On => 1,
            OnOffType::Off => 0,
        }
    }
}

// add Display
impl fmt::Display for OnOffType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OnOffType::On => write!(f, "on"),
            OnOffType::Off => write!(f, "off"),
        }
    }
}

// add parser
impl FromStr for OnOffType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(OnOffType::On),
            "off" => Ok(OnOffType::Off),
            _ => Err(()),
        }
    }
}
