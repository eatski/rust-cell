use std::{collections::{BTreeMap, BTreeSet}, mem};

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
    units: BTreeMap<UnitId, HydratedUnit>,
    original: &'a mut GameState,
}

impl<'a> HydratedGameState<'a> {
    fn new(state: &'a mut GameState) -> Self {
        let mut units: BTreeMap<UnitId, HydratedUnit> = BTreeMap::new();
        for (address, unit_id) in state.cells.iter() {
            let unit_original = state.units.get(unit_id).unwrap();
            let unit = units.entry(*unit_id).or_insert(HydratedUnit {
                unit: unit_original.clone(),
                addresses: Default::default(),
            });
            unit.addresses.insert(*address);
        }
        Self {
            units,
            original: state,
        }
    }
    fn move_unit(&mut self, unit_id: &UnitId, direction: &Direction) {
        let unit = self.units.get_mut(unit_id).unwrap();
        if unit.unit.order != Some(PlayerOrder::Stop) {
            let next_addresses: BTreeSet<Address> = unit
                .addresses
                .iter()
                .map(|address| address.next(direction))
                .collect();
            let is_collision = next_addresses.iter().any(|address| {
                let next_unit_id = self.original.cells.get(address);
                next_unit_id.map(|id| id != unit_id).unwrap_or(false)
            });
            if !is_collision {
                for address in unit.addresses.iter() {
                    self.original.cells.remove(address);
                }
                for address in next_addresses.iter() {
                    self.original.cells.insert(*address, *unit_id);
                }
                unit.addresses = next_addresses;
            }
        }
    }
    fn merge_near_units(&mut self, target_unit_id: &UnitId) {
        let target_unit = self.units.get_mut(target_unit_id).unwrap();
        let cells = &self.original.cells;
        let near_unit_ids: BTreeSet<UnitId> = target_unit
            .addresses
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
        let addresses_to_shft: Vec<_> = {
            let source_unit = self.units.get_mut(source_unit_id).unwrap();
            let addresses = mem::take(&mut source_unit
                .addresses
                ).into_iter()
                .collect();
            addresses
        };
        let destination_unit = self.units.get_mut(destination_unit_id).unwrap();
        for address in addresses_to_shft.iter() {
            self.original.cells.insert(*address, *destination_unit_id);
        }
        destination_unit.addresses.extend(addresses_to_shft);
        self.original.units.remove(source_unit_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_mock_state() -> GameState {
        GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        }
    }

    #[test]
    fn new() {
        let mut state = GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let hydrated = HydratedGameState::new(&mut state);
        insta::assert_debug_snapshot!(hydrated);
    }

    #[test]
    fn move_unit() {
        let mut state = GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Right);
        insta::assert_debug_snapshot!(hydrated.original);
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(0, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Left);
        insta::assert_debug_snapshot!(hydrated.original);
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
        insta::assert_debug_snapshot!(hydrated.original);
    }

    #[test]
    fn merge_near_units() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit::default()), (1, Unit::default())].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.merge_near_units(&0);
        insta::assert_debug_snapshot!(hydrated.original);
    }
}

#[derive(Debug, Clone)]
struct HydratedUnit {
    unit: Unit,
    addresses: BTreeSet<Address>,
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
    match inputs.last() {
        Some(Input::Click { address }) => {
            if let Some(unit_id) = state.cells.get(address) {
                let unit = state.units.get_mut(unit_id).unwrap();
                if Some(PlayerOrder::Stop) == unit.order {
                    unit.order = None;
                } else {
                    unit.order = Some(PlayerOrder::Stop);
                }
            } else {
                let id = state.units.iter().map(|(id, _)| id).max().unwrap_or(&0) + 1;
                state.cells.insert(*address, id);
                state.units.insert(id, Unit::default());
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod update_test {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn update_test() {
        let seed: [i32; 32] = [
            1, 8, 13, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
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
        insta::assert_debug_snapshot!(state);
    }
}
