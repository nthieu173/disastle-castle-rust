mod error;
mod room;
pub use error::CastleError;
pub use room::{connection::Connection, simple_room::SimpleRoom, Pos, Room};

use std::{
    collections::{HashMap, HashSet},
    result,
};

type Result<T> = result::Result<T, CastleError>;

pub struct Castle {
    rooms: HashMap<Pos, Box<dyn Room>>,
}

impl Castle {
    pub fn new(starting_room: Box<dyn Room>) -> Castle {
        let mut rooms = HashMap::new();
        rooms.insert((0, 0), starting_room);
        Castle { rooms }
    }
    pub fn is_lost(&self) -> bool {
        self.rooms.values().all(|v| !v.is_throne())
    }
    pub fn view(&self) -> HashMap<Pos, SimpleRoom> {
        let mut result = HashMap::new();
        for (pos, room) in self.rooms.iter() {
            result.insert(*pos, SimpleRoom::from_room(room.as_ref()));
        }
        result
    }
    pub fn get_links(&self) -> (u8, u8, u8, u8) {
        let mut diamond = 0;
        let mut cross = 0;
        let mut moon = 0;
        let mut any = 0;
        for (pos, room) in self.rooms.iter() {
            for (i, con_pos) in connecting(*pos).iter().enumerate() {
                if let Some(con_room) = self.rooms.get(&con_pos) {
                    match room.get_connections()[i].link(&con_room.get_connections()[(i + 2) % 4]) {
                        Connection::Any => any += 1,
                        Connection::Diamond => diamond += 1,
                        Connection::Cross => cross += 1,
                        Connection::Moon => moon += 1,
                        Connection::None => panic!("Castle has incorrectly placed room."),
                    }
                }
            }
        }
        // Because we count all links twice, we need to divide by 2
        (diamond / 2, cross / 2, moon / 2, any / 2)
    }
    pub fn place_room(&mut self, room: Box<dyn Room>, pos: Pos) -> Result<()> {
        if self.rooms.contains_key(&pos) {
            return Err(CastleError::TakenPosition);
        }
        if !self.can_place_room(room.as_ref(), pos) {
            return Err(CastleError::InvalidConnection);
        }
        self.rooms.insert(pos, room);
        Ok(())
    }
    pub fn move_room(&mut self, from: Pos, to: Pos) -> Result<()> {
        if from == to {
            Err(CastleError::InvalidPosition)
        } else if let Some(room) = self.rooms.get(&from) {
            if !self.room_is_outer(from).unwrap() {
                return Err(CastleError::NotOuterRoom);
            }
            if self.rooms.contains_key(&to) {
                return Err(CastleError::TakenPosition);
            }
            if !self.can_place_room(room.as_ref(), to) {
                return Err(CastleError::InvalidConnection);
            }
            let room = self.rooms.remove(&from).unwrap();
            self.rooms.insert(to, room);
            Ok(())
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    pub fn swap_room(&mut self, pos1: Pos, pos2: Pos) -> Result<()> {
        if pos1 == pos2 {
            Err(CastleError::InvalidPosition)
        } else if let Some(room1) = self.rooms.remove(&pos1) {
            if let Some(room2) = self.rooms.remove(&pos2) {
                // Checking valid swap for room1
                if !self.can_place_room(room1.as_ref(), pos2) {
                    self.rooms.insert(pos1, room1);
                    self.rooms.insert(pos2, room2);
                    return Err(CastleError::InvalidConnection);
                }
                // Checking valid swap for room2
                if !self.can_place_room(room2.as_ref(), pos1) {
                    self.rooms.insert(pos1, room1);
                    self.rooms.insert(pos2, room2);
                    return Err(CastleError::InvalidConnection);
                }
                self.rooms.insert(pos2, room1);
                self.rooms.insert(pos1, room2);
                Ok(())
            } else {
                self.rooms.insert(pos1, room1);
                Err(CastleError::EmptyPosition)
            }
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    pub fn discard_room(&mut self, pos: Pos) -> Result<Box<dyn Room>> {
        if !self.rooms.contains_key(&pos) {
            return Err(CastleError::EmptyPosition);
        }
        if *self.rooms.get(&pos).unwrap().is_throne() && self.rooms.len() > 1 {
            return Err(CastleError::NotOuterRoom);
        }
        let outer_pos: Vec<&Pos> = self
            .rooms
            .keys()
            .filter(|p| self.room_is_outer(**p).unwrap())
            .collect();
        if outer_pos.len() > 0 {
            if let Some(_) = outer_pos.iter().find(|p| ***p == pos) {
                let room = self.rooms.remove(&pos).unwrap();
                return Ok(room);
            } else {
                return Err(CastleError::NotOuterRoom);
            }
        } else if let Some(_) = self
            .rooms
            .keys()
            .filter(|p| self.room_num_connected(**p).unwrap() < 2)
            .find(|p| **p == pos)
        {
            let room = self.rooms.remove(&pos).unwrap();
            return Ok(room);
        }
        Err(CastleError::NotOuterRoom)
    }
    pub fn possible_placements(&self, shop: &Vec<Box<dyn Room>>) -> Vec<(usize, Pos)> {
        let mut possible = Vec::new();
        for (i, room) in shop.iter().enumerate() {
            for pos in self.placable_positions(room.as_ref()) {
                possible.push((i, pos));
            }
        }
        possible
    }
    pub fn possible_moves(&self) -> Vec<(Pos, Pos)> {
        let mut possible = Vec::new();
        for (from, room) in self.rooms.iter() {
            if self.room_is_outer(*from).unwrap() {
                for to in self.placable_positions(room.as_ref()) {
                    possible.push((*from, to));
                }
            }
        }
        possible
    }
    pub fn possible_swaps(&self) -> Vec<(Pos, Pos)> {
        // Since the number of rooms is limited, we can just brute force and check all possible swaps
        let mut possible = Vec::new();
        for (pos1, room1) in self.rooms.iter() {
            for (pos2, room2) in self.rooms.iter() {
                if pos1 != pos2
                    && self.can_place_room(room1.as_ref(), *pos2)
                    && self.can_place_room(room2.as_ref(), *pos1)
                {
                    possible.push((*pos1, *pos2));
                }
            }
        }
        possible
    }
}

impl Castle {
    fn placable_positions(&self, room: &dyn Room) -> Vec<Pos> {
        let mut placable = HashSet::new();
        for pos in self.rooms.keys() {
            for con_pos in connecting(*pos) {
                if !self.rooms.contains_key(&pos) && self.can_place_room(room, con_pos) {
                    placable.insert(con_pos);
                }
            }
        }
        placable.into_iter().collect()
    }
    /*
     * Does not check for already existing room at position
     */
    fn can_place_room(&self, room: &dyn Room, pos: Pos) -> bool {
        let mut count = 0;
        let mut connect = true;
        for (i, con_pos) in connecting(pos).iter().enumerate() {
            if let Some(con_room) = self.rooms.get(&con_pos) {
                if let Some(is_connected) =
                    room.get_connections()[i].connect(&con_room.get_connections()[(i + 2) % 4])
                {
                    if is_connected {
                        count += 1;
                    } else {
                        connect = false;
                        break;
                    }
                }
            }
        }
        return connect && count > 0;
    }
    fn room_is_outer(&self, pos: Pos) -> Result<bool> {
        Ok(self.room_num_connected(pos)? == 1)
    }
    fn room_num_connected(&self, pos: Pos) -> Result<u8> {
        if let Some(room) = self.rooms.get(&pos) {
            let mut count = 0;
            for (i, con_pos) in connecting(pos).iter().enumerate() {
                if let Some(con_room) = self.rooms.get(&con_pos) {
                    if let Some(is_connected) =
                        room.get_connections()[i].connect(&con_room.get_connections()[(i + 2) % 4])
                    {
                        if is_connected {
                            count += 1;
                        }
                    }
                }
            }
            Ok(count)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
}

fn connecting(pos: Pos) -> [Pos; 4] {
    let (x, y) = pos;
    [(x, y - 1), (x, y + 1), (x + 1, y), (x - 1, y)]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}