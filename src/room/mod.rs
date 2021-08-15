pub mod connection;
pub mod simple_room;

use connection::Connection;

use std::{clone::Clone, convert::TryInto};

pub type Pos = (i8, i8);

pub trait Room: RoomClone {
    fn is_throne(&self) -> &bool;
    fn get_name(&self) -> &str;
    fn get_treasure(&self) -> &u8;
    fn get_original_connections(&self) -> &[Connection; 4];
    fn get_rotation(&self) -> &u16;
    fn rotate(&self, rotation: u16) -> Box<dyn Room>;
    fn get_connections(&self) -> [Connection; 4] {
        let connections = self.get_original_connections();
        let rotation = ((self.get_rotation() % 360) / 90) * 90; // Floor to 90 degrees increments
        let rotate_num: usize = (rotation / 90).into();
        let connections: Vec<Connection> = connections[4 - rotate_num..]
            .iter()
            .chain(connections[..4 - rotate_num].iter())
            .copied()
            .collect();
        connections.try_into().unwrap()
    }
}

impl PartialEq for dyn Room {
    fn eq(&self, other: &dyn Room) -> bool {
        self.is_throne() == other.is_throne()
            && self.get_name() == other.get_name()
            && self.get_treasure() == other.get_treasure()
            && self
                .get_original_connections()
                .iter()
                .eq(other.get_original_connections().iter())
    }
}

impl Eq for dyn Room {}

pub trait RoomClone {
    fn clone_box(&self) -> Box<dyn Room>;
}

impl<T> RoomClone for T
where
    T: 'static + Room + Clone,
{
    fn clone_box(&self) -> Box<dyn Room> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Room> {
    fn clone(&self) -> Box<dyn Room> {
        self.clone_box()
    }
}
