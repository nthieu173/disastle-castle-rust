mod error;
mod room;

pub use error::CastleError;
pub use room::{connection::Connection, Pos, Room};

use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    result,
};

type Result<T> = result::Result<T, CastleError>;

#[derive(Clone, Debug)]
pub struct Castle {
    rooms: HashMap<Pos, Room>,
    damage: u8,
}

impl Hash for Castle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Sort the positions to get a stable Hash
        let mut positions: Vec<&Pos> = self.rooms.keys().collect();
        positions.sort_unstable();
        for pos in positions {
            self.rooms[pos].hash(state);
        }
        self.damage.hash(state);
    }
}

impl PartialEq for Castle {
    fn eq(&self, other: &Castle) -> bool {
        if self.damage != other.damage {
            return false;
        }
        for (pos, room) in self.rooms.iter() {
            if let Some(other_room) = other.rooms.get(pos) {
                if room != other_room || room.rotation != other_room.rotation {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl Eq for Castle {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Place(usize, Pos),
    Move(Pos, Pos),
    Swap(Pos, Pos),
    Discard(Pos),
}

impl Castle {
    pub fn new(starting_room: Room) -> Castle {
        let mut rooms = HashMap::new();
        rooms.insert((0, 0), starting_room);
        Castle { rooms, damage: 0 }
    }
    pub fn is_lost(&self) -> bool {
        self.damage as usize >= self.rooms.values().len() || self.rooms.values().all(|v| !v.throne)
    }
    pub fn deal_damage(&self, diamond_damage: u8, cross_damage: u8, moon_damage: u8) -> Castle {
        let (diamond_link, cross_link, moon_link, wild_link) = self.get_links();
        let mut castle = self.clone();
        if diamond_damage > diamond_link {
            castle.damage += diamond_damage - diamond_link;
        }
        if cross_damage > cross_link {
            castle.damage += cross_damage - cross_link;
        }
        if moon_damage > moon_link {
            castle.damage += moon_damage - moon_link;
        }
        if castle.damage > wild_link {
            castle.damage -= wild_link;
        }
        if castle.damage as usize >= castle.rooms.len() {
            castle.damage -= castle.rooms.len() as u8;
            castle.rooms = HashMap::new();
        }
        castle
    }
    pub fn get_rooms(&self) -> &HashMap<Pos, Room> {
        &self.rooms
    }
    pub fn get_damage(&self) -> u8 {
        self.damage
    }
    pub fn get_links(&self) -> (u8, u8, u8, u8) {
        let mut diamond = 0;
        let mut cross = 0;
        let mut moon = 0;
        let mut wild = 0;
        for (pos, room) in self.rooms.iter() {
            for (i, con_pos) in connecting(*pos).iter().enumerate() {
                if let Some(con_room) = self.rooms.get(&con_pos) {
                    if let Ok(link) = room.get_rotated_connections()[i]
                        .link(&con_room.get_rotated_connections()[(i + 2) % 4])
                    {
                        match link {
                            Connection::Wild => wild += 1,
                            Connection::Diamond(_) => diamond += 1,
                            Connection::Cross(_) => cross += 1,
                            Connection::Moon(_) => moon += 1,
                            Connection::None => (),
                        }
                    } else {
                        panic!("Castle has incorrectly placed room");
                    }
                }
            }
        }
        // Because we count all links twice, we need to divide by 2
        (diamond / 2, cross / 2, moon / 2, wild / 2)
    }
    pub fn get_treasure(&self) -> u8 {
        let mut treasure = 0;
        for (pos, room) in self.rooms.iter() {
            if room.treasure > 0 && self.room_is_powered(*pos).unwrap() {
                treasure += room.treasure;
            }
        }
        treasure
    }
}

impl Castle {
    pub fn place_room(&self, room: Room, pos: Pos) -> Result<Castle> {
        if self.damage > 0 {
            return Err(CastleError::MustDiscard);
        }
        if self.rooms.contains_key(&pos) {
            return Err(CastleError::TakenPosition);
        }
        if !self.can_place_room(&room, pos) {
            return Err(CastleError::InvalidConnection);
        }
        let mut castle = self.clone();
        castle.rooms.insert(pos, room);
        Ok(castle)
    }
    pub fn move_room(&self, from: Pos, to: Pos) -> Result<Castle> {
        if self.damage > 0 {
            return Err(CastleError::MustDiscard);
        }
        if from == to {
            Err(CastleError::InvalidPosition)
        } else if self.rooms.contains_key(&from) {
            if !self.room_is_outer(from).unwrap() {
                return Err(CastleError::NotOuterRoom);
            }
            if self.rooms.contains_key(&to) {
                return Err(CastleError::TakenPosition);
            }
            let mut castle = self.clone();
            let room = castle.rooms.remove(&from).unwrap();
            if !castle.can_place_room(&room, to) {
                return Err(CastleError::InvalidConnection);
            }
            castle.rooms.insert(to, room);
            Ok(castle)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    pub fn swap_room(&self, pos1: Pos, pos2: Pos) -> Result<Castle> {
        if self.damage > 0 {
            return Err(CastleError::MustDiscard);
        }
        if pos1 == pos2 {
            Err(CastleError::InvalidPosition)
        } else if self.rooms.contains_key(&pos1) && self.rooms.contains_key(&pos2) {
            let mut castle = self.clone();
            let room1 = castle.rooms.remove(&pos1).unwrap();
            let room2 = castle.rooms.remove(&pos2).unwrap();

            // Checking valid swap for room1 as if room2 was already swapped there.
            castle.rooms.insert(pos1, room2);
            if !castle.can_place_room(&room1, pos2) {
                return Err(CastleError::InvalidConnection);
            }
            let room2 = castle.rooms.remove(&pos1).unwrap();

            castle.rooms.insert(pos2, room1);
            // Checking valid swap for room2 as if room1 was already swapped there.
            if !castle.can_place_room(&room2, pos1) {
                return Err(CastleError::InvalidConnection);
            }
            castle.rooms.insert(pos1, room2); // We passed both checks, so we can swap them.
            Ok(castle)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    pub fn discard_room(&self, pos: Pos) -> Result<(Castle, Room)> {
        if self.damage == 0 {
            return Err(CastleError::NoDamage);
        }
        if !self.rooms.contains_key(&pos) {
            return Err(CastleError::EmptyPosition);
        }
        if self.rooms.get(&pos).unwrap().throne && self.rooms.len() > 1 {
            return Err(CastleError::NotOuterRoom);
        }
        let outer_pos: Vec<&Pos> = self
            .rooms
            .keys()
            .filter(|p| self.room_is_outer(**p).unwrap())
            .collect();
        if outer_pos.len() > 0 {
            if let Some(_) = outer_pos.iter().find(|p| ***p == pos) {
                let mut castle = self.clone();
                let room = castle.rooms.remove(&pos).unwrap();
                castle.damage -= 1;
                return Ok((castle, room));
            } else {
                return Err(CastleError::NotOuterRoom);
            }
        } else if let Some(_) = self
            .rooms
            .keys()
            .filter(|p| self.room_num_connected(**p).unwrap() < 2)
            .find(|p| **p == pos)
        {
            let mut castle = self.clone();
            let room = castle.rooms.remove(&pos).unwrap();
            castle.damage -= 1;
            return Ok((castle, room));
        }
        Err(CastleError::NotOuterRoom)
    }
    pub fn possible_actions(&self, shop: &Vec<Room>) -> Vec<Action> {
        if self.damage > 0 {
            return self
                .possible_discards()
                .into_iter()
                .map(|pos| Action::Discard(pos))
                .collect();
        }
        self.possible_placements(shop)
            .into_iter()
            .map(|(index, pos)| Action::Place(index, pos))
            .chain(
                self.possible_moves()
                    .into_iter()
                    .map(|(from, to)| Action::Move(from, to)),
            )
            .chain(
                self.possible_swaps()
                    .into_iter()
                    .map(|(pos1, pos2)| Action::Swap(pos1, pos2)),
            )
            .collect()
    }
    pub fn clear_rooms(&self) -> Castle {
        let mut castle = self.clone();
        castle.damage -= castle.rooms.len() as u8;
        castle.rooms.clear();
        castle
    }
}

impl Castle {
    fn possible_placements(&self, shop: &Vec<Room>) -> Vec<(usize, Pos)> {
        let mut possible = Vec::new();
        for (i, room) in shop.iter().enumerate() {
            for pos in self.placable_positions(room) {
                possible.push((i, pos));
            }
        }
        possible
    }
    fn possible_moves(&self) -> Vec<(Pos, Pos)> {
        let mut possible = Vec::new();
        let mut castle = self.clone();
        for from in self.rooms.keys() {
            if self.room_is_outer(*from).unwrap() {
                let room = castle.rooms.remove(from).unwrap();
                for to in castle.placable_positions(&room) {
                    if *from != to {
                        possible.push((*from, to));
                    }
                }
                castle.rooms.insert(*from, room);
            }
        }
        possible
    }
    fn possible_swaps(&self) -> Vec<(Pos, Pos)> {
        // Since the number of rooms is limited, we can just brute force and check all possible swaps
        let mut possible = Vec::new();
        for (pos1, room1) in self.rooms.iter() {
            for (pos2, room2) in self.rooms.iter() {
                if pos1 != pos2
                    && self.can_place_room(room1, *pos2)
                    && self.can_place_room(room2, *pos1)
                {
                    possible.push((*pos1, *pos2));
                }
            }
        }
        possible
    }
    fn possible_discards(&self) -> Vec<Pos> {
        if self.is_lost() {
            return Vec::new();
        }
        let mut possible = Vec::new();
        if self.rooms.len() == 1 {
            possible.push(*self.rooms.keys().next().unwrap());
            return possible;
        }
        for (pos, room) in self.rooms.iter() {
            if self.room_is_outer(*pos).unwrap() && !room.throne {
                possible.push(*pos);
            }
        }
        if possible.len() > 0 {
            possible
        } else {
            for (pos, room) in self.rooms.iter() {
                if self.room_num_connected(*pos).unwrap() < 2 && !room.throne {
                    possible.push(*pos);
                }
            }
            return possible;
        }
    }
}

impl Castle {
    fn placable_positions(&self, room: &Room) -> Vec<Pos> {
        let mut placable = HashSet::new();
        for pos in self.rooms.keys() {
            for con_pos in connecting(*pos) {
                if !self.rooms.contains_key(&con_pos) && self.can_place_room(room, con_pos) {
                    placable.insert(con_pos);
                }
            }
        }
        placable.into_iter().collect()
    }
    /*
     * Does not check for already existing room at position
     */
    fn can_place_room(&self, room: &Room, pos: Pos) -> bool {
        let mut count = 0;
        let mut connect = true;
        for (i, con_pos) in connecting(pos).iter().enumerate() {
            if let Some(con_room) = self.rooms.get(&con_pos) {
                if let Some(is_connected) = room.get_rotated_connections()[i]
                    .connect(&con_room.get_rotated_connections()[(i + 2) % 4])
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
                    if let Some(is_connected) = room.get_rotated_connections()[i]
                        .connect(&con_room.get_rotated_connections()[(i + 2) % 4])
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
    fn room_is_powered(&self, pos: Pos) -> Result<bool> {
        if let Some(room) = self.rooms.get(&pos) {
            let connections = room.get_rotated_connections();
            for (i, con_pos) in connecting(pos).iter().enumerate() {
                if connections[i].power() {
                    if let Some(con_room) = self.rooms.get(&con_pos) {
                        if let Ok(link) =
                            connections[i].link(&con_room.get_rotated_connections()[(i + 2) % 4])
                        {
                            if link.power() {
                                continue;
                            }
                        }
                    }
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
}

fn connecting(pos: Pos) -> [Pos; 4] {
    let (x, y) = pos;
    [(x, y - 1), (x + 1, y), (x, y + 1), (x - 1, y)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron;

    #[test]
    fn test_new() {
        let throne: Room = ron::from_str(
            "Room(
                throne: true,
                name: \"Throne Room (White)\",
                treasure: 0,
                rotation: 0,
                connections: (Wild, Wild, Wild, Wild)
            )",
        )
        .unwrap();
        Castle::new(throne);
    }

    #[test]
    fn test_possible_actions() {
        let throne: Room = ron::from_str(
            "Room(
                throne: true,
                name: \"Throne Room (White)\",
                treasure: 0,
                rotation: 0,
                connections: (Wild, Wild, Wild, Wild)
            )",
        )
        .unwrap();
        let castle = Castle::new(throne);
        let shop: Vec<Room> = ron::from_str(
            "[
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, None, None, Cross(false))
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, Diamond(false), None, None)
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, None, Moon(false), None)
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (Cross(false), None, None, None)
            ),
        ]",
        )
        .unwrap();
        let shop: Vec<Room> = shop.into_iter().collect();
        let actions = castle.possible_actions(&shop);
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn test_place_action() {
        let throne: Room = ron::from_str(
            "Room(
                throne: true,
                name: \"Throne Room (White)\",
                treasure: 0,
                rotation: 0,
                connections: (Wild, Wild, Wild, Wild)
            )",
        )
        .unwrap();
        let castle = Castle::new(throne);
        let shop: Vec<Room> = ron::from_str(
            "[
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, None, None, Cross(false))
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, Diamond(false), None, None)
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (None, None, Moon(false), None)
            ),
            Room(
                throne: false,
                treasure: 1,
                name: \"Small Vault\",
                rotation: 0,
                connections: (Cross(false), None, None, None)
            ),
        ]",
        )
        .unwrap();
        let shop: Vec<Room> = shop.into_iter().collect();
        let actions = castle.possible_actions(&shop);
        let sample_action = actions[1];
        let result = match sample_action {
            Action::Place(index, pos) => castle.place_room(shop[index].clone(), pos),
            _ => unreachable!(),
        };
        assert!(result.is_ok());
        let new_castle = result.unwrap();
        assert_eq!(new_castle.get_rooms().len(), 2);
    }
}
