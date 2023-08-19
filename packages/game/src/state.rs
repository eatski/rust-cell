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
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            pathes: [
                UNIT_CORE_PATH,
            ].into(),
        }
    }
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

    pub fn dry_run_move_unit<'a>(&'a mut self, unit_id: &'a UnitId, direction: &'a RelativePath) -> Option<impl FnOnce() + 'a>{
        let (address,unit) = self.units.get(unit_id).unwrap().clone();
        let current_pathes = unit.pathes.clone();
        let current_address:BTreeSet<_>  = current_pathes.into_iter().map(|path| &address + &path).collect();
        let next_addresses:BTreeSet<_>  = current_address.iter().map(move |address| address + &direction).collect();

        let is_collision = next_addresses.iter().any(|address| {
            let next_unit_id = self.cells.get(&address);
            next_unit_id.map(|id| id != unit_id).unwrap_or(false)
        });
        if !is_collision {
            Some(move || {
                let (address,_) = self.units.get_mut(&unit_id).unwrap();
                for address in current_address {
                    self.cells.remove(&address);
                }
                for address in next_addresses {
                    self.cells.insert(address, *unit_id);
                }
                *address = &address.clone() + &direction;
            })
        }else{
            None
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
    pub fn dry_run_add_path<'a>(&'a mut self, unit_id: &UnitId, path: &RelativePath) -> Option<impl FnOnce() + 'a>{
  
        let new_address = {
            let (address,_) = self.units.get(unit_id).unwrap();
            &address.clone() + path
        };
        let new_address_cloned = new_address.clone();
        if self.cells.contains_key(&new_address) {
            return None
        }
        let path = path.clone();
        let unit_id = unit_id.clone();
        Some(move || {
            let (_,unit) = self.units.get_mut(&unit_id).unwrap();
            unit.pathes.insert(path.clone());
            self.cells.insert(new_address_cloned, unit_id);
        })
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
        state.dry_run_move_unit(&unit_id, &RelativePath { x: 1, y: 0 }).unwrap()();
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState::default();
        state.spawn_unit(&Address { x: 0, y: 0 });
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.spawn_unit(&Address { x: 2, y: 0 });
        state.dry_run_move_unit(&unit_id, &RelativePath { x: -1, y: 0 }).unwrap()();
        insta::assert_debug_snapshot!(state.finalize());
    }

    #[test]
    fn move_unit_3() {
        let mut state = GameState::default();
        state.spawn_unit(&Address { x: 0, y: 0 });
        let unit_id = state.units.first_key_value().unwrap().0.clone();
        state.spawn_unit(&Address { x: 1, y: 0 });
        assert!(state.dry_run_move_unit(&unit_id, &RelativePath { x: 1, y: 0 }).is_none());
    }

}
