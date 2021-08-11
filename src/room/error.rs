use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RoomError {
    InvalidRotation,
}

impl fmt::Display for RoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          RoomError::InvalidRotation => write!(f, "Invalid rotation. Rotation must be 0, 90, 180 or 270."),
        }
    }
}

impl Error for RoomError {}