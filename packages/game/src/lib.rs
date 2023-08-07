use std::collections::HashMap;

use rand::{seq::SliceRandom, Rng};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Address {
    pub x: isize,
    pub y: isize,
}

pub enum Input {
    Click { address: Address },
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: usize,
}

#[derive(Debug, Default)]
pub struct GameState {
    pub cells: HashMap<Address, Unit>,
}

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 * inputsがある場合は、入力に応じて点を追加する
 */
pub fn update(state: &mut GameState, inputs: &Vec<Input>) {
    let mut rng = rand::thread_rng();
    let mut units = state
        .cells
        .clone()
        .into_iter()
        .collect::<Vec<(Address, Unit)>>();
    units.shuffle(&mut rng);
    for (address, unit) in units.into_iter() {
        if rng.gen_range(0..8) == 0 {
            let direction = rng.gen_range(0..4);
            let next_address = match direction {
                0 => Address {
                    x: address.x,
                    y: address.y - 1,
                },
                1 => Address {
                    x: address.x,
                    y: address.y + 1,
                },
                2 => Address {
                    x: address.x - 1,
                    y: address.y,
                },
                3 => Address {
                    x: address.x + 1,
                    y: address.y,
                },
                _ => panic!("direction is invalid"),
            };
            if !state.cells.contains_key(&next_address) {
                state.cells.remove(&address);
                state.cells.insert(next_address, unit);
            }
        }
    }
    for input in inputs {
        match input {
            Input::Click { address } => {
                state.cells.insert(
                    address.clone(),
                    Unit {
                        id: state.cells.len(),
                    },
                );
            }
        }
    }
}
