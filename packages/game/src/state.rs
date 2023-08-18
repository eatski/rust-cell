use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Add,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Address {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct RelativePath {
    pub x: isize,
    pub y: isize,
}

pub const NEXT_PATHES: [RelativePath; 4] = [
    RelativePath { x: 0, y: -1 },
    RelativePath { x: 0, y: 1 },
    RelativePath { x: -1, y: 0 },
    RelativePath { x: 1, y: 0 },
];

impl Add<&RelativePath> for &Address {
    type Output = Address;
    fn add(self, rhs: &RelativePath) -> Self::Output {
        Address {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub order: Option<PlayerOrder>,
    pub pathes: BTreeSet<RelativePath>,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            order: None,
            pathes: [
                RelativePath { x: 0, y: 0 },
            ].into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerOrder {
    Stop,
}

pub type UnitId = usize;

#[derive(Debug)]
pub struct FinalizedGameState {
    pub cells: Vec<(Address, Unit)>,
}

#[derive(Debug,Default)]
pub struct GameState{
    pub units: BTreeMap<UnitId, (Address,Unit)>,
    pub cells: BTreeMap<Address, UnitId>,
}

impl GameState{
    pub fn move_unit(&mut self, unit_id: &UnitId, direction: &RelativePath) {
        let (address,unit) = self.units.get_mut(unit_id).unwrap();
        let address_cloned = address.clone();
        if unit.order != Some(PlayerOrder::Stop) {
            let current_pathes = &unit.pathes;
            let current_address = current_pathes.iter().map(|path| &address_cloned + path);
            let next_addresses = current_address.clone().map(|address| &address + direction);

            let is_collision = next_addresses.clone().any(|address| {
                let next_unit_id = self.cells.get(&address);
                next_unit_id.map(|id| id != unit_id).unwrap_or(false)
            });
            if !is_collision {
                for address in current_address {
                    self.cells.remove(&address);
                }
                for address in next_addresses {
                    self.cells.insert(address, *unit_id);
                }
                *address = &address_cloned + direction;
            }
        }
    }
    /**
     * unitを新しくスポーンさせる
     */
    pub fn spawn_unit(&mut self, address: &Address) {
        let unit_id = self.units.keys().max().unwrap_or(&0) + 1;
        self.cells.insert(*address, unit_id);
        self.units.insert(unit_id, (*address,Unit::default()));
    }
    
    /**
     * unitのpathを追加する
     */
    pub fn add_path(&mut self, unit_id: &UnitId, path: &RelativePath) {
        let (address,unit) = self.units.get_mut(unit_id).unwrap();
        if unit.order != Some(PlayerOrder::Stop) {
            unit.pathes.insert(path.clone());
            self.cells.insert(*address, *unit_id);
        }
    }

    pub fn finalize(&self) -> FinalizedGameState {
        let cells = self.units.iter().map(|(_, (address, unit))| (address.clone(), unit.clone())).collect();
        FinalizedGameState { cells }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_unit() {
        let mut state = GameState::default();
        state.spawn_unit(&Address { x: 0, y: 0 });
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.spawn_unit(&Address { x: 2, y: 0 });
        state.move_unit(&unit_id, &RelativePath { x: 1, y: 0 });
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState::default();
        state.spawn_unit(&Address { x: 0, y: 0 });
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.spawn_unit(&Address { x: 2, y: 0 });
        state.move_unit(&unit_id, &RelativePath { x: -1, y: 0 });
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_3() {
        let mut state = GameState::default();
        state.spawn_unit(&Address { x: 0, y: 0 });
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.spawn_unit(&Address { x: 1, y: 0 });
        state.move_unit(&unit_id, &RelativePath { x: 1, y: 0 });
        insta::assert_debug_snapshot!(state.finalize());
    }

}
