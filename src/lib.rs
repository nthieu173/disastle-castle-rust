mod error;
mod room;

pub use error::CastleError;
pub use room::{connection::Connection, Room};

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    result,
};

type Result<T> = result::Result<T, CastleError>;

pub type Pos = (i8, i8);
pub type Rot = u16;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Ord, PartialOrd)]
pub struct PlacedRoom {
    pub info: Room,
    pub rotation: Rot,
}

impl PlacedRoom {
    pub fn from(room: Room, rotation: Rot) -> Self {
        Self {
            info: room,
            rotation,
        }
    }
    pub fn rotate(&self, rotation: Rot) -> Self {
        Self {
            info: self.info.clone(),
            rotation,
        }
    }
    pub fn get_connections(&self) -> [Connection; 4] {
        self.info.get_rotated_connections(self.rotation)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Castle {
    pub rooms: BTreeMap<Pos, PlacedRoom>,
    pub damage: u8,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Action {
    Place(Room, Pos, Rot),
    Move(Pos, Pos, Rot),
    Swap(Pos, Pos),
    Discard(Vec<Pos>),
    Damage(u8, u8, u8),
}

impl Castle {
    pub fn new(starting_room: Room) -> Castle {
        let mut rooms = BTreeMap::new();
        rooms.insert((0, 0), PlacedRoom::from(starting_room, 0));
        Castle { rooms, damage: 0 }
    }
    pub fn is_lost(&self) -> bool {
        self.damage as usize >= self.rooms.values().len()
            || self.rooms.values().all(|v| !v.info.throne)
    }
    pub fn get_links(&self) -> (u8, u8, u8, u8) {
        let mut diamond = 0;
        let mut cross = 0;
        let mut moon = 0;
        let mut wild = 0;
        for (pos, room) in self.rooms.iter() {
            for (i, con_pos) in connecting(*pos).iter().enumerate() {
                if let Some(con_room) = self.rooms.get(&con_pos) {
                    if let Ok(link) =
                        room.get_connections()[i].link(&con_room.get_connections()[(i + 2) % 4])
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
            if room.info.treasure > 0 && self.room_is_powered(*pos).unwrap() {
                treasure += room.info.treasure;
            }
        }
        treasure
    }
}

impl Castle {
    fn action_place(&self, room: Room, pos: Pos, rot: Rot) -> Result<Castle> {
        if self.damage > 0 {
            return Err(CastleError::MustDiscard);
        }
        if self.rooms.contains_key(&pos) {
            return Err(CastleError::TakenPosition);
        }
        if !self.can_place_room(&PlacedRoom::from(room.clone(), rot), pos) {
            return Err(CastleError::InvalidConnection);
        }
        let mut castle = self.clone();
        castle.rooms.insert(pos, PlacedRoom::from(room, rot));
        Ok(castle)
    }
    fn action_move(&self, from: Pos, to: Pos, rot: Rot) -> Result<Castle> {
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
            if !castle.can_place_room(&room.rotate(rot), to) {
                return Err(CastleError::InvalidConnection);
            }
            castle.rooms.insert(to, room);
            Ok(castle)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    fn action_swap(&self, pos_1: Pos, pos_2: Pos) -> Result<Castle> {
        if self.damage > 0 {
            return Err(CastleError::MustDiscard);
        }
        if pos_1 == pos_2 {
            Err(CastleError::InvalidPosition)
        } else if self.rooms.contains_key(&pos_1) && self.rooms.contains_key(&pos_2) {
            let mut castle = self.clone();
            let room1 = castle.rooms.remove(&pos_1).unwrap();
            let room2 = castle.rooms.remove(&pos_2).unwrap();

            // Then, first placing room2 in pos_1 then trying to place room1 in pos_2.
            castle.rooms.insert(pos_1, room2);
            if !castle.can_place_room(&room1, pos_2) {
                return Err(CastleError::InvalidConnection);
            }
            let room2 = castle.rooms.remove(&pos_1).unwrap();

            // First placing room1 in pos_2 then trying to place room2 in pos_1.
            castle.rooms.insert(pos_2, room1);
            if !castle.can_place_room(&room2, pos_1) {
                return Err(CastleError::InvalidConnection);
            }
            castle.rooms.insert(pos_1, room2); // We passed both checks, so we can swap them.
            Ok(castle)
        } else {
            Err(CastleError::EmptyPosition)
        }
    }
    fn action_discard_one(&self, pos: Pos) -> Result<Castle> {
        if !self.rooms.contains_key(&pos) {
            return Err(CastleError::EmptyPosition);
        }
        if self.rooms.get(&pos).unwrap().info.throne && self.rooms.len() > 1 {
            return Err(CastleError::NotOuterRoom);
        }
        let outer_pos: Vec<&Pos> = self
            .rooms
            .keys()
            .filter(|p| !self.rooms[p].info.throne && self.room_is_outer(**p).unwrap())
            .collect();
        if outer_pos.len() > 0 {
            if self.room_is_outer(pos).unwrap() {
                let mut castle = self.clone();
                castle.rooms.remove(&pos).unwrap();
                castle.damage -= 1;
                return Ok(castle);
            } else {
                return Err(CastleError::NotOuterRoom);
            }
        }
        let nearly_outer_pos: Vec<&Pos> = self
            .rooms
            .keys()
            .filter(|p| !self.rooms[p].info.throne && self.room_num_connected(**p).unwrap() <= 2)
            .collect();
        if nearly_outer_pos.len() > 0 {
            if self.room_num_connected(pos).unwrap() <= 2 {
                let mut castle = self.clone();
                castle.rooms.remove(&pos).unwrap();
                castle.damage -= 1;
                return Ok(castle);
            } else {
                return Err(CastleError::NotNearlyOuterRoom);
            }
        }
        return Err(CastleError::MustDiscard);
    }
    fn action_discard(&self, poses: Vec<Pos>) -> Result<Castle> {
        if self.damage == 0 {
            return Err(CastleError::NoDamage);
        }
        let mut castle = self.clone();
        for pos in poses {
            castle = castle.action_discard_one(pos)?;
        }
        if self.damage > 0 {
            Err(CastleError::MustDiscard)
        } else {
            Ok(castle)
        }
    }
    pub fn action_damage(&self, diamond_damage: u8, cross_damage: u8, moon_damage: u8) -> Castle {
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
            castle.rooms = BTreeMap::new();
        }
        castle
    }
    pub fn apply(&self, action: Action) -> Result<Castle> {
        match action {
            Action::Place(room, pos, rot) => self.action_place(room, pos, rot),
            Action::Move(from, to, rot) => self.action_move(from, to, rot),
            Action::Swap(pos_1, pos_2) => self.action_swap(pos_1, pos_2),
            Action::Discard(poses) => self.action_discard(poses),
            Action::Damage(diamond, cross, moon) => Ok(self.action_damage(diamond, cross, moon)),
        }
    }
    pub fn possible_actions(&self, shop: &Vec<Room>) -> Vec<Action> {
        if self.damage > 0 {
            return self
                .all_possible_discards()
                .into_iter()
                .map(|poses| Action::Discard(poses))
                .collect();
        }
        self.all_possible_placements(shop)
            .into_iter()
            .map(|(index, pos)| Action::Place(shop[index].clone(), pos, 0))
            .chain(
                self.all_possible_moves()
                    .into_iter()
                    .map(|(from, to)| Action::Move(from, to, 0)),
            )
            .chain(
                self.all_possible_swaps()
                    .into_iter()
                    .map(|(pos_1, pos_2)| Action::Swap(pos_1, pos_2)),
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
    pub fn all_possible_placements(&self, shop: &Vec<Room>) -> Vec<(usize, Pos)> {
        let mut possible = Vec::new();
        for (i, room) in shop.iter().enumerate() {
            for pos in self.possible_placements(&PlacedRoom::from(room.clone(), 0)) {
                possible.push((i, pos));
            }
        }
        possible
    }
    pub fn all_possible_moves(&self) -> Vec<(Pos, Pos)> {
        let mut possible = Vec::new();
        for from in self.rooms.keys() {
            possible.append(
                &mut self
                    .possible_moves(*from, 0)
                    .into_iter()
                    .map(|to| (*from, to))
                    .collect(),
            );
        }
        possible
    }
    pub fn all_possible_swaps(&self) -> Vec<(Pos, Pos)> {
        // Since the number of rooms is limited, we can just brute force and check all possible swaps
        let mut possible: Vec<(Pos, Pos)> = Vec::new();
        for pos_1 in self.rooms.keys() {
            possible.append(
                &mut self
                    .possible_swaps(*pos_1)
                    .into_iter()
                    .map(|pos_2| (*pos_1, pos_2))
                    .collect(),
            );
        }
        possible
    }
    pub fn all_possible_discards(&self) -> Vec<Vec<Pos>> {
        let mut possible = Vec::new();
        let mut queue: Vec<(Castle, Vec<Pos>)> = Vec::new();
        queue.append(
            &mut self
                .possible_discard()
                .into_iter()
                .map(|pos| (self.action_discard_one(pos).unwrap(), vec![pos]))
                .collect(),
        );
        while let Some((castle, discards)) = queue.pop() {
            if castle.damage == 0 {
                possible.push(discards);
            } else {
                queue.append(
                    &mut castle
                        .possible_discard()
                        .into_iter()
                        .map(|pos| (castle.action_discard_one(pos).unwrap(), vec![pos]))
                        .collect(),
                );
            }
        }
        possible
    }
    pub fn possible_discard(&self) -> Vec<Pos> {
        if self.is_lost() {
            return Vec::new();
        }
        let mut possible = Vec::new();
        if self.rooms.len() == 1 {
            possible.push(*self.rooms.keys().next().unwrap());
            return possible;
        }
        for (pos, room) in self.rooms.iter() {
            if self.room_is_outer(*pos).unwrap() && !room.info.throne {
                possible.push(*pos);
            }
        }
        if possible.len() > 0 {
            possible
        } else {
            for (pos, room) in self.rooms.iter() {
                if self.room_num_connected(*pos).unwrap() <= 2 && !room.info.throne {
                    possible.push(*pos);
                }
            }
            possible
        }
    }
    pub fn possible_placements(&self, room: &PlacedRoom) -> Vec<Pos> {
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
    pub fn possible_moves(&self, from: Pos, rotation: u16) -> Vec<Pos> {
        let mut castle = self.clone();
        let mut possible = Vec::new();
        if let Ok(room_is_outer) = self.room_is_outer(from) {
            if room_is_outer {
                let room = castle.rooms.remove(&from).unwrap();
                for to in castle.possible_placements(&room.rotate(rotation)) {
                    if from != to {
                        possible.push(to);
                    }
                }
                castle.rooms.insert(from, room);
            }
        }
        possible
    }
    pub fn possible_swaps(&self, from: Pos) -> Vec<Pos> {
        // Since the number of rooms is limited, we can just brute force and check all possible swaps
        let mut possible = Vec::new();
        let pos_1 = &from;
        if let Some(room1) = self.rooms.get(&from) {
            for (pos_2, room2) in self.rooms.iter() {
                if pos_1 != pos_2
                    && self.can_place_room(room1, *pos_2)
                    && self.can_place_room(room2, *pos_1)
                {
                    possible.push(*pos_2);
                }
            }
        }
        possible
    }
}

impl Castle {
    /*
     * Does not check for already existing room at position
     */
    fn can_place_room(&self, room: &PlacedRoom, pos: Pos) -> bool {
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
    fn room_is_powered(&self, pos: Pos) -> Result<bool> {
        if let Some(room) = self.rooms.get(&pos) {
            let connections = room.get_connections();
            for (i, con_pos) in connecting(pos).iter().enumerate() {
                if connections[i].power() {
                    if let Some(con_room) = self.rooms.get(&con_pos) {
                        if let Ok(link) =
                            connections[i].link(&con_room.get_connections()[(i + 2) % 4])
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
        let sample_action = actions[1].clone();
        let result = castle.apply(sample_action);
        assert!(result.is_ok());
        let new_castle = result.unwrap();
        assert_eq!(new_castle.rooms.len(), 2);
    }
}
