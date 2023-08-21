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

impl Add<&RelativePath> for &RelativePath {
    type Output = RelativePath;
    fn add(self, rhs: &RelativePath) -> Self::Output {
        RelativePath {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub const NEXT_PATHES: [RelativePath; 4] = [
    RelativePath { x: 0, y: -1 },
    RelativePath { x: 0, y: 1 },
    RelativePath { x: -1, y: 0 },
    RelativePath { x: 1, y: 0 },
];

pub const UNIT_CORE_PATH: RelativePath = RelativePath { x: 0, y: 0 };

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
    pub pathes: BTreeSet<RelativePath>,
    pub blueprint: BTreeSet<RelativePath>,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            pathes: [UNIT_CORE_PATH].into(),
            blueprint: Default::default(),
        }
    }
}

impl Unit {
    pub fn new(blueprint: BTreeSet<RelativePath>) -> Self {
        Self {
            pathes: [UNIT_CORE_PATH].into(),
            blueprint,
        }
    }
}

pub type UnitId = usize;

#[derive(Debug)]
pub struct FinalizedGameState {
    pub cells: Vec<(Address, Unit)>,
}

#[derive(Debug, Default)]
pub struct GameState {
    pub units: BTreeMap<UnitId, (Address, Unit)>,
    pub cells: BTreeMap<Address, (UnitId, RelativePath)>,
}

impl GameState {
    pub fn dry_run_move_unit<'a>(
        &'a mut self,
        unit_id: &'a UnitId,
        direction: &'a RelativePath,
    ) -> Result<impl FnOnce() + 'a, BTreeSet<(UnitId, RelativePath)>> {
        let (address, unit) = self.units.get(unit_id).unwrap().clone();
        let current_pathes = unit.pathes.clone();
        let current_address: BTreeSet<_> = current_pathes
            .into_iter()
            .map(|path| (&address + &path, path))
            .collect();
        let next_addresses: BTreeSet<_> = current_address
            .iter()
            .map(move |(address, path)| (address + &direction, path.clone()))
            .collect();

        let collision_addresses: BTreeSet<_> = next_addresses
            .iter()
            .filter_map(|(address, _)| {
                let next_unit_id = self.cells.get(&address);
                next_unit_id.and_then(|(next_unit_id, path)| {
                    if next_unit_id != unit_id {
                        Some((next_unit_id.clone(), path.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect();
        if collision_addresses.is_empty() {
            Ok(move || {
                let (address, _) = self.units.get_mut(&unit_id).unwrap();
                for (address, _) in current_address {
                    self.cells.remove(&address);
                }
                for (address, path) in next_addresses {
                    self.cells.insert(address, (*unit_id, path.clone()));
                }
                *address = &address.clone() + &direction;
            })
        } else {
            Err(collision_addresses)
        }
    }

    /**
     * unitを新しくスポーンさせる
     */
    pub fn dry_run_spawn_unit<'a>(
        &'a mut self,
        address: &'a Address,
    ) -> Option<impl FnOnce(Unit) + 'a> {
        if self.cells.contains_key(address) {
            return None;
        }
        let address = address.clone();
        Some(move |unit| {
            let unit_id = self.units.keys().max().unwrap_or(&0) + 1;
            self.cells.insert(address, (unit_id, UNIT_CORE_PATH));
            self.units.insert(unit_id, (address, unit));
        })
    }

    /**
     * unitのpathを追加する
     */
    pub fn dry_run_add_path<'a>(
        &'a mut self,
        unit_id: &UnitId,
        path: &RelativePath,
    ) -> Option<impl FnOnce() + 'a> {
        let new_address = {
            let (address, _) = self.units.get(unit_id).unwrap();
            &address.clone() + path
        };
        let new_address_cloned = new_address.clone();
        if self.cells.contains_key(&new_address) {
            return None;
        }
        let path = path.clone();
        let unit_id = unit_id.clone();
        Some(move || {
            let (_, unit) = self.units.get_mut(&unit_id).unwrap();
            unit.pathes.insert(path.clone());
            self.cells.insert(new_address_cloned, (unit_id, path));
        })
    }

    /**
     * unitを削除する
     */
    pub fn remove_unit(&mut self, unit_id: &UnitId) {
        let (address, unit) = self.units.remove(unit_id).unwrap();
        for path in unit.pathes {
            self.cells.remove(&(&address + &path));
        }
    }

    pub fn finalize(&self) -> FinalizedGameState {
        let cells = self
            .units
            .iter()
            .map(|(_, (address, unit))| (address.clone(), unit.clone()))
            .collect();
        FinalizedGameState { cells }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_unit() {
        let mut state = GameState::default();
        state.dry_run_spawn_unit(&Address { x: 0, y: 0 }).unwrap()(Unit::default());
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.dry_run_spawn_unit(&Address { x: 2, y: 0 }).unwrap()(Unit::default());
        state
            .dry_run_move_unit(&unit_id, &RelativePath { x: 1, y: 0 })
            .unwrap()();
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState::default();
        state.dry_run_spawn_unit(&Address { x: 0, y: 0 }).unwrap()(Unit::default());
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.dry_run_spawn_unit(&Address { x: 2, y: 0 }).unwrap()(Unit::default());
        state
            .dry_run_move_unit(&unit_id, &RelativePath { x: -1, y: 0 })
            .unwrap()();
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_3() {
        let mut state = GameState::default();
        state.dry_run_spawn_unit(&Address { x: 0, y: 0 }).unwrap()(Unit::default());
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.dry_run_spawn_unit(&Address { x: 1, y: 0 }).unwrap()(Unit::default());
        let error = state
            .dry_run_move_unit(&unit_id, &RelativePath { x: 1, y: 0 })
            .map(|_| ())
            .unwrap_err();
        insta::assert_debug_snapshot!(error);
    }
}
