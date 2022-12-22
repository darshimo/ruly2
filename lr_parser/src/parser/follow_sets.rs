use std::collections::{HashMap, HashSet};

use crate::item::Item;

pub fn compute_follow_sets(
    nonterminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>,
    first_sets: &HashMap<Vec<String>, HashSet<Option<String>>>,
) -> HashMap<String, HashSet<String>> {
    let mut follow_sets: HashMap<String, HashSet<String>> = map_lhs2items
        .iter()
        .map(|(s, _)| (s.clone(), HashSet::new()))
        .collect();

    loop {
        let mut not_changed = true;

        for (_, item_set) in map_lhs2items {
            for item in item_set {
                let lhs = item.get_lhs();
                let rhs = item.get_rhs();

                for i in (0..rhs.len()).rev() {
                    let a = rhs.get(i).unwrap();
                    let w = &rhs[i + 1..];

                    if nonterminal_symbols.contains(a) {
                        let (first_set_of_w_without_epsilon, w_can_be_epsilon) = first_sets
                            .get(w)
                            .unwrap()
                            .into_iter()
                            .fold((HashSet::new(), false), |(mut set, b), x| {
                                if let Some(symbol) = x {
                                    set.insert(symbol.to_string());
                                    (set, b)
                                } else {
                                    (set, true)
                                }
                            });

                        let new_set = if w_can_be_epsilon {
                            first_set_of_w_without_epsilon
                                .union(follow_sets.get(lhs).unwrap())
                                .into_iter()
                                .map(|x| x.clone())
                                .collect()
                        } else {
                            first_set_of_w_without_epsilon
                        };
                        let old_set = follow_sets.get(a).unwrap();

                        if !old_set.is_superset(&new_set) {
                            *follow_sets.get_mut(a).unwrap() =
                                old_set.union(&new_set).map(|s| s.clone()).collect();
                            not_changed = false;
                        }
                    }
                }
            }
        }

        if not_changed {
            break;
        }
    }

    follow_sets
}
