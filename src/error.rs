use std::{error::Error, fmt};

#[derive(Debug)]
pub enum CastleError {
    TakenPosition,
    EmptyPosition,
    InvalidConnection,
    InvalidPosition,
    NotOuterRoom,
}

impl fmt::Display for CastleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastleError::TakenPosition => write!(f, "Room position is already taken"),
            CastleError::EmptyPosition => write!(f, "Room position does not contain a room"),
            CastleError::InvalidConnection => write!(f, "Room cannot be placed, moved or swapped because the connections to it does not match up."),
            CastleError::InvalidPosition => write!(f, "Cannot select the same position as both the source and destination of a move or swap."),
            CastleError::NotOuterRoom => write!(f, "Room cannot be moved or discarded because it is not an outer room."),
        }
    }
}

impl Error for CastleError {}
