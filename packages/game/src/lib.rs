pub mod state;
use std::collections::BTreeSet;

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
        let rnd = rng.gen_range(0..2048);
        match rnd {            0..=24 => {
                let direction = NEXT_PATHES.iter().choose(rng).unwrap();
                let mv = state.dry_run_move_unit(current_unit_id, direction);
                if let Some(exec) = mv {
                    exec();
                }
            }
            25 => {
                let (_,unit) = state.units.get(current_unit_id).unwrap();
                let next_pathes: BTreeSet<_> = unit.pathes.iter().flat_map(|path| {
                    NEXT_PATHES.iter().map(move |next_path| path + next_path)
                }).collect();
                let mut next_pathes: Vec<_> = next_pathes.into_iter().collect();
                next_pathes.shuffle(rng);

                for path in next_pathes.iter() {
                    if let Some(exec) = state.dry_run_add_path(current_unit_id, path) {
                        exec();
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(Input::Click { address }) = inputs.last() {
        if let Some(_) = state.cells.get(address).cloned() {
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
        for _ in 0..10000 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
    }
}
