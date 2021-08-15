#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    None,
    Any,
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
    pub fn link(&self, other: &Connection) -> Connection {
        match (self, other) {
            (Connection::Any, Connection::Any) => Connection::Any,
            (Connection::Any, Connection::Diamond(_)) => Connection::Diamond(false),
            (Connection::Any, Connection::Cross(_)) => Connection::Cross(false),
            (Connection::Any, Connection::Moon(_)) => Connection::Moon(false),
            (Connection::Diamond(power), Connection::Any) => Connection::Diamond(*power),
            (Connection::Cross(power), Connection::Any) => Connection::Cross(*power),
            (Connection::Moon(power), Connection::Any) => Connection::Moon(*power),
            (Connection::Cross(power), Connection::Cross(_)) => Connection::Cross(*power),
            (Connection::Diamond(power), Connection::Diamond(_)) => Connection::Diamond(*power),
            (Connection::Moon(power), Connection::Moon(_)) => Connection::Moon(*power),
            (_, _) => Connection::None,
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
