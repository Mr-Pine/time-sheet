use serde::Deserialize;
use toml::value::Datetime;

use crate::input::WorkingArea;
use crate::time::WorkingDuration;

#[derive(Debug, Clone, Deserialize)]
pub struct Contract {
    working_time: WorkingDuration,
    area: WorkingArea,
    wage: Option<f32>,
    start_date: Option<Datetime>,
    end_date: Option<Datetime>,
    bg_content: Option<String>,
}

impl Contract {
    /// How long the employee has to work each month.
    pub fn working_time(&self) -> &WorkingDuration {
        &self.working_time
    }

    /// In which field the employee is working at the university.
    pub fn area(&self) -> &WorkingArea {
        &self.area
    }

    /// How much the employee makes per hour (in euros).
    pub fn wage(&self) -> Option<f32> {
        self.wage
    }

    /// When the contract starts.
    pub fn start_date(&self) -> Option<&Datetime> {
        self.start_date.as_ref()
    }

    /// When the contract ends.
    pub fn end_date(&self) -> Option<&Datetime> {
        self.end_date.as_ref()
    }

    /// In the bottom left of the final PDF is a small signature.
    ///
    /// If this is set, then the signature will be replaced with this text.
    pub fn bg_content(&self) -> Option<&str> {
        self.bg_content.as_deref()
    }
}