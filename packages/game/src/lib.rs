use std::collections::{BTreeMap, BTreeSet};

use rand::{seq::SliceRandom, Rng};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy,PartialOrd, Ord)]
pub struct Address {
    pub x: isize,
    pub y: isize,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn directions() -> Vec<Self> {
        vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }
}

impl Address {
    fn next(&self, direction: &Direction) -> Self {
        match direction {
            Direction::Up => Address {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Address {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Address {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Address {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

pub enum Input {
    Click { address: Address },
}

#[derive(Debug, Clone, Default)]
pub struct Unit {
    pub order: Option<PlayerOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerOrder {
    Stop,
}

type UnitId = usize;

#[derive(Debug, Default)]
pub struct GameState {
    pub cells: BTreeMap<Address, UnitId>,
    pub units: BTreeMap<UnitId, Unit>,
}

#[derive(Debug)]
struct HydratedGameState<'a> {
    map: RelationalOneToMany<'a,UnitId,Address>,
    units: &'a mut BTreeMap<UnitId, Unit>,
}

impl<'a> HydratedGameState<'a> {
    fn new(state: &'a mut GameState) -> Self {
        Self {
            map: (&mut state.cells).into(),
            units: &mut state.units,
        }
    }
    fn move_unit(&mut self, unit_id: &UnitId, direction: &Direction) {
        let unit = self.units.get_mut(unit_id).unwrap();
        if unit.order != Some(PlayerOrder::Stop) {
            let current_addresses = self.map.get_one_to_many().get(unit_id).unwrap().clone();
            let next_addresses = 
                current_addresses
                .iter()
                .map(|address| address.next(direction));
                
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
    fn merge_near_units(&mut self, target_unit_id: &UnitId) {
        let cells = self.map.get_many_to_one();
        let addresses = self.map.get_one_to_many().get(target_unit_id).unwrap();
        let near_unit_ids: BTreeSet<UnitId> = addresses
            .iter()
            .flat_map(|address| {
                let directions = Direction::directions();
                directions
                    .into_iter()
                    .map(move |direction| cells.get(&address.next(&direction)))
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
        let source_addresses = self.map.get_one_to_many().get(source_unit_id).unwrap().clone();
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
    fn spawn_unit(&mut self,address: &Address) {
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
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Right);
        insta::assert_debug_snapshot!(state);
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(0, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Left);
        insta::assert_debug_snapshot!(state);
    }

    #[test]
    fn move_unit_stop() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(0,  Unit {
                order: Some(PlayerOrder::Stop),
                ..Unit::default()
            })].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Left);
        insta::assert_debug_snapshot!(state);
    }

    #[test]
    fn merge_near_units() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.merge_near_units(&0);
        insta::assert_debug_snapshot!(state);
    }
}

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 * inputsがある場合は、入力に応じて点を追加する
 */
pub fn update(state: &mut GameState, inputs: &Vec<Input>, rng: &mut impl Rng) {
    let units = state.units.clone();
    let mut units_to_iter: Vec<_> = units.iter().collect();
    units_to_iter.shuffle(rng);
    let mut hydrated = HydratedGameState::new(state);
    for (current_unit_id, _) in units_to_iter.iter() {
        if rng.gen_range(0..32) == 0 {
            let directions = Direction::directions();
            let direction = directions.choose(rng).unwrap();
            hydrated.move_unit(current_unit_id, direction);
            hydrated.merge_near_units(current_unit_id);
        }
    }
    if let Some(Input::Click { address }) = inputs.last() {
        if let Some(unit_id) = hydrated.map.get_many_to_one().get(address).cloned() {
            let unit = state.units.get_mut(&unit_id).unwrap();
            unit.order = if Some(PlayerOrder::Stop) == unit.order {
                None
            } else {
                Some(PlayerOrder::Stop)
            }
        } else {
            hydrated.spawn_unit(address);
        }
    }
}

#[cfg(test)]
mod update_test {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn update_test() {
        let seed: [u8; 32] = [
            1, 8, 13, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut state = GameState::default();
        update(
            &mut state,
            &vec![Input::Click {
                address: Address { x: 0, y: 0 },
            }],
            &mut rng,
        );
        update(
            &mut state,
            &vec![Input::Click {
                address: Address { x: 1, y: 2 },
            }],
            &mut rng,
        );
        update(
            &mut state,
            &vec![Input::Click {
                address: Address { x: 0, y: 0 },
            }],
            &mut rng,
        );
        // update を 30回繰り返す
        for _ in 0..30 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state);
    }
}


#[derive(Debug)]
struct RelationalOneToMany<'a,OneKey,ManyKey>{
    one_to_many: BTreeMap<OneKey, BTreeSet<ManyKey>>,
    original: &'a mut BTreeMap<ManyKey, OneKey>,
}

impl <'a,OneKey: Ord + Clone,ManyKey: Clone + Ord>From<&'a mut BTreeMap<ManyKey,OneKey>> for RelationalOneToMany<'a,OneKey,ManyKey> {
    fn from(original: &'a mut BTreeMap<ManyKey,OneKey>) -> Self {
        let one_to_many = original.iter().fold(BTreeMap::new(), |mut acc, (many_key, one_key)| {
            acc.entry(one_key.clone()).or_insert_with(BTreeSet::new).insert(many_key.clone());
            acc
        });
        RelationalOneToMany {
            one_to_many,
            original,
        }
    }
}

impl <'a,OneKey: Ord + Clone,ManyKey: Ord + Clone>RelationalOneToMany<'a,OneKey,ManyKey> {
    fn get_one_to_many(&self) -> &BTreeMap<OneKey, BTreeSet<ManyKey>> {
        &self.one_to_many
    }
    fn get_many_to_one(&self) -> &BTreeMap<ManyKey, OneKey> {
        &self.original
    }
    fn insert_many(&mut self, one_key: &OneKey, many_key: &ManyKey) {
        self.original.insert(many_key.clone(), one_key.clone());
        self.one_to_many.entry(one_key.clone()).or_insert_with(BTreeSet::new).insert(many_key.clone());
    }
    fn remove_many(&mut self, many_key: &ManyKey) {
        if let Some(one_key) = self.original.remove(many_key) {
            if let Some(many_keys) = self.one_to_many.get_mut(&one_key) {
                many_keys.remove(many_key);
            }
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
        let relational = RelationalOneToMany::from(&mut original);
        insta::assert_debug_snapshot!(relational);
    }
}