use crate::error::CastleError;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub enum Connection {
    None,
    Wild,
    Diamond(bool),
    Cross(bool),
    Moon(bool),
}

impl Connection {
    pub fn connect(&self, other: &Connection) -> Option<bool> {
        if matches!(self, Connection::None) && matches!(other, Connection::None) {
            return None;
        }
        Some(!matches!(self, Connection::None) && !matches!(other, Connection::None))
    }
    /*
    Tells the powered state of THIS connection if connected to other.
    */
    pub fn link(&self, other: &Connection) -> Result<Connection, CastleError> {
        match (self, other) {
            (Connection::Wild, Connection::Wild) => Ok(Connection::Wild),
            (Connection::Wild, Connection::Diamond(_)) => Ok(Connection::Diamond(true)),
            (Connection::Wild, Connection::Cross(_)) => Ok(Connection::Cross(true)),
            (Connection::Wild, Connection::Moon(_)) => Ok(Connection::Moon(true)),
            (Connection::Diamond(power), Connection::Wild) => Ok(Connection::Diamond(*power)),
            (Connection::Cross(power), Connection::Wild) => Ok(Connection::Cross(*power)),
            (Connection::Moon(power), Connection::Wild) => Ok(Connection::Moon(*power)),
            (Connection::Cross(power), Connection::Cross(_)) => Ok(Connection::Cross(*power)),
            (Connection::Diamond(power), Connection::Diamond(_)) => Ok(Connection::Diamond(*power)),
            (Connection::Moon(power), Connection::Moon(_)) => Ok(Connection::Moon(*power)),
            (Connection::None, Connection::None) => Ok(Connection::None),
            (Connection::None, _) => Err(CastleError::InvalidConnection),
            (_, Connection::None) => Err(CastleError::InvalidConnection),
            (_, _) => Ok(Connection::None),
        }
    }
    pub fn power(&self) -> bool {
        match self {
            Connection::Diamond(power) => *power,
            Connection::Cross(power) => *power,
            Connection::Moon(power) => *power,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        assert!(Connection::None.connect(&Connection::None).is_none());
        assert!(Connection::None.connect(&Connection::Wild).is_some());
        assert!(Connection::None
            .connect(&Connection::Diamond(true))
            .is_some());
        assert!(Connection::None.connect(&Connection::Cross(true)).is_some());
        assert!(Connection::None.connect(&Connection::Moon(true)).is_some());
    }
}
