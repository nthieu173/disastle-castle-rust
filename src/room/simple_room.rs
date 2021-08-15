use super::Connection;
use super::Room;

#[derive(Clone, PartialEq, Eq)]
pub struct SimpleRoom {
    throne: bool,
    name: String,
    treasure: u8,
    rotation: u16,
    connections: [Connection; 4],
}

impl Room for SimpleRoom {
    fn is_throne<'a>(&'a self) -> &'a bool {
        &self.throne
    }
    fn get_name<'a>(&'a self) -> &'a str {
        &self.name
    }
    fn get_treasure<'a>(&'a self) -> &'a u8 {
        &self.treasure
    }
    fn get_original_connections<'a>(&'a self) -> &'a [Connection; 4] {
        &self.connections
    }
    fn get_rotation<'a>(&'a self) -> &'a u16 {
        &self.rotation
    }
    fn rotate<'a>(&'a self, rotation: u16) -> Box<dyn Room> {
        let mut room = self.clone();
        room.rotation = rotation;
        Box::new(room)
    }
}

impl SimpleRoom {
    pub fn from_room(r: &dyn Room) -> SimpleRoom {
        SimpleRoom {
            throne: *r.is_throne(),
            treasure: *r.get_treasure(),
            name: r.get_name().to_string(),
            rotation: *r.get_rotation(),
            connections: r.get_connections(),
        }
    }
}
