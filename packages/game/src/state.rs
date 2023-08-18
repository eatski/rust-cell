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

const NEXT_PATHES: [RelativePath; 4] = [
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

#[derive(Debug, Clone, Default)]
pub struct Unit {
    pub order: Option<PlayerOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerOrder {
    Stop,
}

pub type UnitId = usize;

#[derive(Debug, Default)]
pub struct GameState {
    pub cells: BTreeMap<Address, UnitId>,
    pub units: BTreeMap<UnitId, Unit>,
}

#[derive(Debug,Default)]
pub struct HydratedGameState{
    pub map: RelationalOneToMany<UnitId, Address>,
    pub units: BTreeMap<UnitId, Unit>,
}

impl HydratedGameState{
    pub fn normarize(&self) -> GameState {
        GameState {
            cells: self.map.get_many_to_one().clone(),
            units: self.units.clone(),
        }
    }

    pub fn optimize(state: GameState) -> Self {
        Self {
            map: state.cells.into(),
            units: state.units,
        }
    }

    pub fn move_unit(&mut self, unit_id: &UnitId, direction: &RelativePath) {
        let unit = self.units.get_mut(unit_id).unwrap();
        if unit.order != Some(PlayerOrder::Stop) {
            let current_addresses = self.map.get_one_to_many().get(unit_id).unwrap().clone();
            let next_addresses = current_addresses.iter().map(|address| address + direction);

            let is_collision = next_addresses.clone().any(|address| {
                let next_unit_id = self.map.get_many_to_one().get(&address);
                next_unit_id.map(|id| id != unit_id).unwrap_or(false)
            });
            if !is_collision {
                for address in current_addresses.iter() {
                    self.map.remove_many(&address);
                }
                for address in next_addresses {
                    self.map.insert_many(unit_id, &address);
                }
            }
        }
    }
    pub fn merge_near_units(&mut self, target_unit_id: &UnitId) {
        let cells = self.map.get_many_to_one();
        let addresses = self.map.get_one_to_many().get(target_unit_id).unwrap();
        let near_unit_ids: BTreeSet<UnitId> = addresses
            .iter()
            .flat_map(|address| {
                let directions = NEXT_PATHES;
                directions
                    .into_iter()
                    .map(move |direction| cells.get(&(address + &direction)))
            })
            .filter(|unit_id| unit_id.map(|id| id != target_unit_id).unwrap_or(false))
            .filter_map(|unit_id| unit_id.copied())
            .collect();
        // near_unit_idのunitを吸収する
        for near_id in near_unit_ids.iter() {
            self.shift_addresses(near_id, target_unit_id);
        }
    }
    /**
     * source_unit_idのaddressをdestination_unit_idに移動する
     */
    fn shift_addresses(&mut self, source_unit_id: &UnitId, destination_unit_id: &UnitId) {
        let source_addresses = self
            .map
            .get_one_to_many()
            .get(source_unit_id)
            .unwrap()
            .clone();
        for address in source_addresses.iter() {
            self.map.remove_many(&address);
        }
        for address in source_addresses {
            self.map.insert_many(destination_unit_id, &address);
        }
        self.units.remove(source_unit_id);
    }
    /**
     * unitを新しくスポーンさせる
     */
    pub fn spawn_unit(&mut self, address: &Address) {
        let unit_id = self.units.keys().max().unwrap_or(&0) + 1;
        self.map.insert_many(&unit_id, address);
        self.units.insert(unit_id, Unit::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_unit() {
        let mut state = GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::optimize(state);
        hydrated.move_unit(&0, &RelativePath { x: 1, y: 0 });
        insta::assert_debug_snapshot!(hydrated.normarize());
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(0, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::optimize(state);
        hydrated.move_unit(&0, &RelativePath { x: -1, y: 0 });
        insta::assert_debug_snapshot!(hydrated.normarize());
    }

    #[test]
    fn move_unit_stop() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(
                0,
                Unit {
                    order: Some(PlayerOrder::Stop),
                    ..Unit::default()
                },
            )]
            .into(),
        };
        let mut hydrated = HydratedGameState::optimize(state);
        hydrated.move_unit(&0, &RelativePath { x: -1, y: 0 });
        insta::assert_debug_snapshot!(hydrated.normarize());
    }

    #[test]
    fn merge_near_units() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::optimize(state);
        hydrated.merge_near_units(&0);
        insta::assert_debug_snapshot!(hydrated.normarize());
    }
}

#[derive(Debug)]
pub struct RelationalOneToMany<OneKey, ManyKey> {
    one_to_many: BTreeMap<OneKey, BTreeSet<ManyKey>>,
    original: BTreeMap<ManyKey, OneKey>,
}

impl <OneKey, ManyKey>Default for RelationalOneToMany<OneKey, ManyKey> {
    fn default() -> Self {
        Self {
            one_to_many: BTreeMap::new(),
            original: BTreeMap::new(),
        }
    }
}

impl<OneKey: Ord + Clone, ManyKey: Ord + Clone> RelationalOneToMany<OneKey, ManyKey> {
    pub fn get_one_to_many(&self) -> &BTreeMap<OneKey, BTreeSet<ManyKey>> {
        &self.one_to_many
    }
    pub fn get_many_to_one(&self) -> &BTreeMap<ManyKey, OneKey> {
        &self.original
    }
    pub fn insert_many(&mut self, one_key: &OneKey, many_key: &ManyKey) {
        self.original.insert(many_key.clone(), one_key.clone());
        self.one_to_many
            .entry(one_key.clone())
            .or_insert_with(BTreeSet::new)
            .insert(many_key.clone());
    }
    pub fn remove_many(&mut self, many_key: &ManyKey) {
        if let Some(one_key) = self.original.remove(many_key) {
            if let Some(many_keys) = self.one_to_many.get_mut(&one_key) {
                many_keys.remove(many_key);
            }
        }
    }
}

impl<OneKey: Ord + Clone, ManyKey: Clone + Ord> From<BTreeMap<ManyKey, OneKey>>
    for RelationalOneToMany<OneKey, ManyKey>
{
    fn from(original: BTreeMap<ManyKey, OneKey>) -> Self {
        let one_to_many = original
            .iter()
            .fold(BTreeMap::new(), |mut acc, (many_key, one_key)| {
                acc.entry(one_key.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(many_key.clone());
                acc
            });
        RelationalOneToMany {
            one_to_many,
            original,
        }
    }
}

#[cfg(test)]
mod relational_one_to_many_test {
    use super::*;

    #[test]
    fn new() {
        let mut original = BTreeMap::new();
        original.insert(0, 0);
        original.insert(1, 0);
        original.insert(2, 1);
        let relational = RelationalOneToMany::from(original);
        insta::assert_debug_snapshot!(relational);
    }
}
