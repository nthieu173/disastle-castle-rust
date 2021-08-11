pub mod simple_room;
pub mod connection;
pub mod error;

use error::RoomError;
use connection::Connection;

use std::{result, convert::TryInto};

type Result<T> = result::Result<T, RoomError>;
pub type Pos = (i8, i8);

pub trait Room {
    fn is_throne(&self) -> &bool;
    fn get_name(&self) -> &str;
    fn get_original_connections(&self) -> &[Connection; 4];
    fn get_rotation(&self) -> &u16;
    fn get_connections(&self) -> [Connection; 4] {
        rotate(self.get_original_connections(), *self.get_rotation()).unwrap_or_else(
            |_e: RoomError|
                panic!("Room has invalid rotation value {}", self.get_rotation())
            )
    }
    fn rotate(&self, rotation: u16) -> Result<[Connection; 4]> {
        if rotation != 0 && rotation != 90 && rotation != 180 && rotation != 270 {
            return Err(RoomError::InvalidRotation);
        }
        let connections = self.get_connections();
        rotate(&connections, rotation)
    }
}

fn rotate(connections: &[Connection; 4], rotation: u16) -> Result<[Connection; 4]> {
    if rotation != 0 && rotation != 90 && rotation != 180 && rotation != 270 {
    }
    let rotate_num: usize = (rotation / 90).into();
    let connections: Vec<Connection> = connections[4-rotate_num..].iter()
                                        .chain(connections[..4-rotate_num].iter())
                                        .copied().collect();
    Ok(connections.try_into().unwrap())
}