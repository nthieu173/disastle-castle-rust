pub mod connection;

use connection::Connection;
use serde::{Deserialize, Serialize};

use std::{clone::Clone, convert::TryInto, fmt, hash::Hash};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Ord, PartialOrd)]
pub struct Room {
    pub name: String,
    pub throne: bool,
    pub treasure: u8,
    pub connections: [Connection; 4],
}

impl Room {
    pub fn get_rotated_connections(&self, rotation: u16) -> [Connection; 4] {
        let connections = self.connections;
        let rotation = ((rotation % 360) / 90) * 90; // Floor to 90 degrees increments
        let rotate_num: usize = (rotation / 90).into();
        let connections: Vec<Connection> = connections[4 - rotate_num..]
            .iter()
            .chain(connections[..4 - rotate_num].iter())
            .copied()
            .collect();
        connections.try_into().unwrap()
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Room")
            .field("name", &self.name)
            .field("connections", &self.connections)
            .finish()
    }
}
