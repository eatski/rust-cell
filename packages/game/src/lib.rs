pub mod state;
use std::collections::BTreeSet;

use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use state::*;

pub enum Input {
    Click { address: Address },
}



fn calc_similarity<T: Ord>(a: &BTreeSet<T>, b: &BTreeSet<T>) -> (usize,usize) {
    let a_len = a.len();
    let b_len = b.len();
    let similarity = a.intersection(b).count();
    (similarity * 2,a_len+b_len)
}

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 * inputsがある場合は、入力に応じて点を追加する
 */
pub fn update(state: &mut GameState, inputs: &Vec<Input>, rng: &mut impl Rng) {
    let mut units_to_iter: Vec<_> = state.units.clone().into_iter().collect();
    units_to_iter.shuffle(rng);
    for (current_unit_id, _) in units_to_iter.iter() {
        if state.units.get(current_unit_id).is_none() {
            continue;
        }
        let rnd = rng.gen_range(0..1024);
        match rnd {
            0..=24 => {
                let direction = NEXT_PATHES.iter().choose(rng).unwrap();
                let fail = {
                    match state.dry_run_move_unit(current_unit_id, direction) {
                        Ok(exec) => {
                            exec();
                            None
                        }
                        Err(collisions) => Some(collisions),
                    }
                };
                if let Some(collisions) = fail {
                    let (cores, others): (Vec<_>, Vec<_>) = collisions
                        .into_iter()
                        .partition(|(_, path)| path == &UNIT_CORE_PATH);
                    if others.is_empty() {
                        let is_any_unit_is_similar = cores
                            .iter()
                            .any(|(unit_id, _)| {
                                let (_, target) = state.units.get(unit_id).unwrap();
                                let (_, current) = state.units.get(current_unit_id).unwrap();
                                fn get_pathes_and_blueprint(unit: &Unit) -> BTreeSet<&RelativePath> {
                                    unit.pathes
                                        .iter()
                                        .chain(unit.blueprint.iter())
                                        .collect()
                                }

                                let (simularity,denominator) = calc_similarity(&get_pathes_and_blueprint(target), &get_pathes_and_blueprint(current));
                                simularity >  rng.gen_range(0..denominator) + 1
                            });
                        if !is_any_unit_is_similar {
                            // coresのみ衝突している場合は、衝突したunitを消す
                            for (id, _) in cores {
                                state.remove_unit(&id);
                            }
                            state.dry_run_move_unit(current_unit_id, direction).unwrap()();
                        }
                    }
                }
            }
            25..=26 => {
                let (_, unit) = state.units.get(current_unit_id).unwrap();
                let next_pathes: BTreeSet<_> = unit
                    .pathes
                    .iter()
                    .flat_map(|path| NEXT_PATHES.iter().map(move |next_path| path + next_path))
                    .collect();
                let  (mut next_pathes_in_blueprint,mut next_pathes): (Vec<_>,Vec<_>) = 
                    next_pathes.into_iter().partition(|path| unit.blueprint.contains(path));
                next_pathes_in_blueprint.shuffle(rng);
                next_pathes.shuffle(rng);
                for path in next_pathes_in_blueprint.into_iter().chain(next_pathes.into_iter()) {
                    let executed = state
                        .dry_run_add_path(current_unit_id, &path)
                        .map(|exec| {
                            exec();
                        })
                        .map(|()| {
                            let (_,unit) = state.units.get_mut(current_unit_id).unwrap();
                            unit.blueprint.remove(&path);
                        });
                    if executed.is_some() {
                        break;
                    }
                }
            }
            27 => {
                let (address, unit) = state.units.get(current_unit_id).unwrap();
                let mut pathes = unit.pathes.clone();
                pathes.remove(&UNIT_CORE_PATH);
                let mut next_pathes: Vec<_> = NEXT_PATHES.iter().map(|path| address + path).collect();
                next_pathes.shuffle(rng);
                for address in next_pathes {
                    let pathes = pathes.clone();
                    if state.dry_run_spawn_unit(&address)
                    .map(move |exec| {
                        exec(Unit::new(pathes.clone()));
                    })
                    .is_some() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(Input::Click { address }) = inputs.last() {
        if let Some(exec) = state.dry_run_spawn_unit(address) {
            exec(Unit::default());
        };
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

        for _ in 0..30 {
            let address = Address {
                x: rng.gen_range(0..100),
                y: rng.gen_range(0..100),
            };
            update(&mut state, &vec![Input::Click { address }], &mut rng);
        }
        for _ in 0..50 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
        for _ in 0..50 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
        for _ in 0..1000 {
            update(&mut state, &vec![], &mut rng);
        }
        insta::assert_debug_snapshot!(state.finalize());
    }


    #[test]
    fn calc_similarity_test() {
        let a: BTreeSet<_> = vec![1].into_iter().collect();
        let b: BTreeSet<_> = vec![1].into_iter().collect();
        assert_eq!(calc_similarity(&a, &b), (2,2));
        let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
        let b: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(calc_similarity(&a, &b), (6,6));
        let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
        let b: BTreeSet<_> = vec![1, 2, 4].into_iter().collect();
        assert_eq!(calc_similarity(&a, &b), (4,6));
        let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
        let b: BTreeSet<_> = vec![1, 2, 4,5].into_iter().collect();
        assert_eq!(calc_similarity(&a, &b), (4,7));
    }
}
