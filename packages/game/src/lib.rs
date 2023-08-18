pub mod state;
use state::*;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};


pub enum Input {
    Click { address: Address },
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
    for (current_unit_id, _) in units_to_iter.iter() {
        if rng.gen_range(0..32) == 0 {
            let direction = NEXT_PATHES.iter().choose(rng).unwrap();
            state.move_unit(current_unit_id, direction);
        }
    }
    if let Some(Input::Click { address }) = inputs.last() {
        if let Some(unit_id) = state.cells.get(address).cloned() {
            let (_,unit) = state.units.get_mut(&unit_id).unwrap();
            unit.order = if Some(PlayerOrder::Stop) == unit.order {
                None
            } else {
                Some(PlayerOrder::Stop)
            }
        } else {
            state.spawn_unit(address);
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
        let mut state = Default::default();
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
        update(
            &mut state,
            &vec![Input::Click {
                address: Address { x: 5, y: 0 },
            }],
            &mut rng,
        );
        update(
            &mut state,
            &vec![Input::Click {
                address: Address { x: 5, y: 4 },
            }],
            &mut rng,
        );
        for _ in 0..50 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
        for _ in 0..50 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
        for _ in 0..100000 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
    }
}
