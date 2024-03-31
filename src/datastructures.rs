use std::{collections::HashMap, time::Duration};

use crate::wcoerror::WCOError;

#[derive(Debug)]
pub struct Competitor {
    pub name: String,
    pub wca_id: Option<String>,
    pub events: Vec<Event>,
    pub personal_records: HashMap<Event, Duration>,
}

impl Competitor {
    pub fn new(name: String, wca_id: Option<String>, events: Vec<Event>) -> Self {
        Competitor {
            name,
            wca_id,
            events,
            personal_records: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Ev333,
    Ev222,
    Ev444,
    Ev555,
    Ev666,
    Ev777,
    Ev333bf,
    Ev333fm,
    Ev333oh,
    EvClock,
    EvMinx,
    EvPyram,
    EvSkewb,
    EvSq1,
    Ev444bf,
    Ev555bf,
    Ev333mbf,
}

impl Event {
    pub fn pretty_name(&self) -> &'static str {
        match self {
            Event::Ev333 => "3x3x3 Cube",
            Event::Ev222 => "2x2x2 Cube",
            Event::Ev444 => "4x4x4 Cube",
            Event::Ev555 => "5x5x5 Cube",
            Event::Ev666 => "6x6x6 Cube",
            Event::Ev777 => "7x7x7 Cube",
            Event::Ev333bf => "3x3x3 Blindfolded",
            Event::Ev333fm => "3x3x3 Fewest Moves",
            Event::Ev333oh => "3x3x3 One-Handed",
            Event::EvClock => "Clock",
            Event::EvMinx => "Megaminx",
            Event::EvPyram => "Pyraminx",
            Event::EvSkewb => "Skewb",
            Event::EvSq1 => "Square-1",
            Event::Ev444bf => "4x4x4 Blindfolded",
            Event::Ev555bf => "5x5x5 Blindfolded",
            Event::Ev333mbf => "3x3x3 Multi-Blind",
        }
    }

    pub fn code_name(&self) -> &'static str {
        match self {
            Event::Ev333 => "333",
            Event::Ev222 => "222",
            Event::Ev444 => "444",
            Event::Ev555 => "555",
            Event::Ev666 => "666",
            Event::Ev777 => "777",
            Event::Ev333bf => "333bf",
            Event::Ev333fm => "333fm",
            Event::Ev333oh => "333oh",
            Event::EvClock => "clock",
            Event::EvMinx => "minx",
            Event::EvPyram => "pyram",
            Event::EvSkewb => "skewb",
            Event::EvSq1 => "sq1",
            Event::Ev444bf => "444bf",
            Event::Ev555bf => "555bf",
            Event::Ev333mbf => "333mbf",
        }
    }
}

impl TryFrom<&str> for Event {
    type Error = WCOError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "333" => Ok(Event::Ev333),
            "222" => Ok(Event::Ev222),
            "444" => Ok(Event::Ev444),
            "555" => Ok(Event::Ev555),
            "666" => Ok(Event::Ev666),
            "777" => Ok(Event::Ev777),
            "333bf" => Ok(Event::Ev333bf),
            "333fm" => Ok(Event::Ev333fm),
            "333oh" => Ok(Event::Ev333oh),
            "clock" => Ok(Event::EvClock),
            "minx" => Ok(Event::EvMinx),
            "pyram" => Ok(Event::EvPyram),
            "skewb" => Ok(Event::EvSkewb),
            "sq1" => Ok(Event::EvSq1),
            "444bf" => Ok(Event::Ev444bf),
            "555bf" => Ok(Event::Ev555bf),
            "333mbf" => Ok(Event::Ev333mbf),
            _ => Err(WCOError::ParsingError(format!(
                r#"Failed to parse "{}" into an event"#,
                value
            ))),
        }
    }
}
