use std::collections::{HashMap, HashSet};

use crate::{item::Item, parser::Action};

use super::transition_map;

pub fn compute_lr0_parsing_table(
    start_symbol: &str,
    terminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>,
) -> Result<HashMap<usize, HashMap<Option<String>, Action>>, String> {
    let (lr0_transition_map, closure_state_map, accept_state) =
        transition_map::compute_lr0_transition_map(start_symbol, map_lhs2items);

    let mut ret: HashMap<usize, HashMap<Option<String>, Action>> = HashMap::new();

    // shift
    for (u, map) in lr0_transition_map {
        let transitions_from_u = ret.entry(u).or_insert(HashMap::new());
        for (c, v) in map {
            transitions_from_u.insert(Some(c.to_string()), Action::Shift(v));
        }
    }

    // reduce
    for (closure, &state) in &closure_state_map {
        if let Some(reducible_item) = closure.into_iter().find(|&item| item.is_reducible()) {
            if closure.len() == 1 {
                let map = ret.get_mut(&state).unwrap();
                for symbol in terminal_symbols {
                    map.insert(
                        Some(symbol.to_string()),
                        Action::Reduce(reducible_item.clone()),
                    );
                }
            } else {
                return Err("shift/reduce or reduce/reduce conflict!".to_string());
            }
        }
    }

    //accept
    ret.insert(accept_state, HashMap::from([(None, Action::Accept)]));

    Ok(ret)
}
