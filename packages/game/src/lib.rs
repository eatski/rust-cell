use std::collections::HashMap;

use rand::Rng;

#[derive(Debug,PartialEq, Eq, Hash, Clone, Copy)]
pub struct Address {
    pub x: isize,
    pub y: isize,
}

pub enum Input {
    Click { address: Address },
}

#[derive(Debug)]
pub struct Unit {
    pub id: usize,
}

#[derive(Debug,Default)]
pub struct GameState {
    pub cells: HashMap<Address,Unit>,
}

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 * inputsがある場合は、入力に応じて点を追加する
 */
pub fn update(state: &mut GameState,inputs: &Vec<Input>){
    let mut rng = rand::thread_rng();
    let cells: HashMap<_,_> = state.cells.drain().collect();
    for (address,unit) in cells.into_iter() {
        let direction = rng.gen_range(0..4);
        let next_address = match direction {
            0 => Address { x: address.x, y: address.y - 1 },
            1 => Address { x: address.x, y: address.y + 1 },
            2 => Address { x: address.x - 1, y: address.y },
            3 => Address { x: address.x + 1, y: address.y },
            _ => panic!("direction is invalid"),
        };
        state.cells.insert(next_address,unit);
    }
    for input in inputs {
        match input {
            Input::Click { address } => {
                state.cells.insert(*address,Unit{id:0});
            }
        }
    }
}