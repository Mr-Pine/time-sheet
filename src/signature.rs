use std::path::{Path, PathBuf};

use crate::time::Date;

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    /// Path to a signature that will then be automatically added.
    path: PathBuf,
    /// The width of the signature in cm, by default `3.8cm`.
    width: f32,
    date: Date,
}

impl Signature {
    #[must_use]
    pub fn new(date: Date, path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            width: 3.8,
            date: date.into(),
        }
    }

    #[must_use]
    pub fn date(&self) -> &Date {
        &self.date
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        self.width
    }
}
