#[derive(Debug, Clone, Copy)]
pub enum Connection {
    None,
    Any,
    Diamond,
    Cross,
    Moon,
}

impl Connection {
    pub fn connect(&self, other: &Connection) -> Option<bool> {
        if matches!(self, Connection::None) && matches!(other, Connection::None) {
            return None;
        }
        Some(!matches!(self, Connection::None) && !matches!(other, Connection::None))
    }
    pub fn link(&self, other: &Connection) -> Connection {
        match (self, other) {
            (Connection::Any, Connection::Any) => Connection::Any,
            (Connection::Any, Connection::Diamond) => Connection::Diamond,
            (Connection::Diamond, Connection::Any) => Connection::Diamond,
            (Connection::Diamond, Connection::Diamond) => Connection::Diamond,
            (Connection::Any, Connection::Cross) => Connection::Cross,
            (Connection::Cross, Connection::Any) => Connection::Cross,
            (Connection::Cross, Connection::Cross) => Connection::Cross,
            (Connection::Any, Connection::Moon) => Connection::Moon,
            (Connection::Moon, Connection::Any) => Connection::Moon,
            (Connection::Moon, Connection::Moon) => Connection::Moon,
            (_, _) => Connection::None,
        }
    }
}
