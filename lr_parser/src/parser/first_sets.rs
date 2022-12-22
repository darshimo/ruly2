use std::collections::{HashMap, HashSet};

use crate::item::Item;

pub fn compute_first_sets(
    nonterminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>,
) -> HashMap<Vec<String>, HashSet<Option<String>>> {
    let mut first_sets = {
        let mut tmp = HashMap::from([(vec![], HashSet::from([None]))]);

        for (_, item_set) in map_lhs2items {
            for item in item_set {
                let lhs = item.get_lhs();
                let rhs = item.get_rhs();

                tmp.insert(vec![lhs.to_string()], HashSet::new());

                for i in 0..rhs.len() {
                    tmp.insert(rhs[i..].to_vec(), HashSet::new());
                }
            }
        }

        tmp
    };

    loop {
        let mut not_changed = true;

        for (_, item_set) in map_lhs2items {
            for item in item_set {
                let lhs = item.get_lhs();
                let rhs = &item.get_rhs()[..];

                for i in (0..rhs.len()).rev() {
                    let w = &rhs[i..];
                    let a = w.first().expect("6");

                    let new_set = if nonterminal_symbols.contains(a) {
                        let mut first_set_of_a =
                            first_sets.get(&vec![a.to_string()]).unwrap().clone();

                        if first_set_of_a.remove(&None) {
                            first_set_of_a
                                .union(first_sets.get(&w[1..]).expect("8"))
                                .into_iter()
                                .map(|x| x.clone())
                                .collect()
                        } else {
                            first_set_of_a
                        }
                    } else {
                        HashSet::from([Some(a.to_string())])
                    };
                    let old_set = first_sets.get(w).expect("2");

                    if !old_set.is_superset(&new_set) {
                        *first_sets.get_mut(w).expect("3") =
                            old_set.union(&new_set).map(|s| s.clone()).collect();
                        not_changed = false;
                    }
                }

                let new_set = first_sets.get(rhs).expect("4");
                let old_set = first_sets.get(&vec![lhs.to_string()]).expect("5");

                if !old_set.is_superset(&new_set) {
                    *first_sets.get_mut(&vec![lhs.to_string()]).expect("1") =
                        old_set.union(&new_set).map(|s| s.clone()).collect();
                    not_changed = false;
                }
            }
        }

        if not_changed {
            break;
        }
    }

    first_sets
}
