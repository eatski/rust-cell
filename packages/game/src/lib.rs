use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, Rng};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Unit;

type UnitId = usize;

#[derive(Debug, Default)]
pub struct GameState {
    pub cells: HashMap<Address, UnitId>,
    pub units: HashMap<UnitId, Unit>,
}

#[derive(Debug)]
struct HydratedGameState<'a> {
    units: HashMap<UnitId, HydratedUnit>,
    state: &'a mut GameState,
}

impl<'a> HydratedGameState<'a> {
    fn new(state: &'a mut GameState) -> Self {
        let mut units: HashMap<UnitId, HydratedUnit> = HashMap::new();
        for (address, unit_id) in state.cells.iter() {
            let unit = units.entry(*unit_id).or_insert(HydratedUnit {
                unit: Unit,
                addresses: Vec::new(),
            });
            unit.addresses.push(*address);
        }
        Self { units, state }
    }
    fn move_unit(&mut self, unit_id: &UnitId, direction: &Direction) {
        let unit = self.units.get_mut(unit_id).unwrap();
        let next_addresses: Vec<Address> = unit
            .addresses
            .iter()
            .map(|address| address.next(direction))
            .collect();
        let is_collision = next_addresses.iter().any(|address| {
            let next_unit_id = self.state.cells.get(address);
            next_unit_id.map(|id| id != unit_id).unwrap_or(false)
        });
        if !is_collision {
            for address in unit.addresses.iter() {
                self.state.cells.remove(address);
            }
            for address in next_addresses.iter() {
                self.state.cells.insert(*address, *unit_id);
            }
            unit.addresses = next_addresses;
        }
    }
    fn merge_near_units(&mut self, target_unit_id: &UnitId) {
        let cloned_units = self.units.clone();
        let target_unit = self.units.get_mut(target_unit_id).unwrap();
        let cells = &self.state.cells;
        let near_unit_ids: HashSet<UnitId> = target_unit
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
            let near_unit = cloned_units.get(near_id).unwrap();
            for address in near_unit.addresses.iter() {
                self.state.cells.insert(*address, *target_unit_id);
                target_unit.addresses.push(*address);
            }
            self.state.units.remove(near_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_mock_state() -> GameState {
        GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit), (1, Unit)].into(),
        }
    }

    #[test]
    fn new() {
        let mut state = GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit), (1, Unit)].into(),
        };
        let hydrated = HydratedGameState::new(&mut state);
        insta::assert_debug_snapshot!(hydrated);
    }

    #[test]
    fn move_unit() {
        let mut state = GameState {
            cells: [(Address { x: 0, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit), (1, Unit)].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Right);
        insta::assert_debug_snapshot!(hydrated.state);
    }

    #[test]
    fn move_unit_2() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 0)].into(),
            units: [(0, Unit)].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.move_unit(&0, &Direction::Left);
        insta::assert_debug_snapshot!(hydrated.state);
    }

    #[test]
    fn merge_near_units() {
        let mut state = GameState {
            cells: [(Address { x: 1, y: 0 }, 0), (Address { x: 2, y: 0 }, 1)].into(),
            units: [(0, Unit), (1, Unit)].into(),
        };
        let mut hydrated = HydratedGameState::new(&mut state);
        hydrated.merge_near_units(&0);
        insta::assert_debug_snapshot!(hydrated.state);
    }
}

#[derive(Debug,Clone)]
struct HydratedUnit {
    unit: Unit,
    addresses: Vec<Address>,
}

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 * inputsがある場合は、入力に応じて点を追加する
 */
pub fn update(state: &mut GameState, inputs: &Vec<Input>) {
    let mut rng = rand::thread_rng();
    let units = state.units.clone();
    let mut units_to_iter: Vec<_> = units.iter().collect();
    units_to_iter.shuffle(&mut rng);
    let mut hydrated = HydratedGameState::new(state);
    for (current_unit_id, _) in units_to_iter.iter() {
        if rng.gen_range(0..8) == 0 {
            let directions = Direction::directions();
            let direction = directions.choose(&mut rng).unwrap();
            hydrated.move_unit(current_unit_id, direction);
            hydrated.merge_near_units(current_unit_id);
        }
    }
    match inputs.last() {
        Some(Input::Click { address }) => {
            let id = state.units.iter().map(|(id, _)| id).max().unwrap_or(&0) + 1;
            state.cells.insert(*address, id);
            state.units.insert(id, Unit);
        }
        None => {},
    }
}
