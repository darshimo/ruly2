use std::collections::{HashMap, HashSet};

use crate::{
    item::Item,
    parser::{first_sets, follow_sets, lr0, Action},
};

pub fn compute_slr_parsing_table(
    start_symbol: &str,
    nonterminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>,
) -> Result<HashMap<usize, HashMap<Option<String>, Action>>, String> {
    let (lr0_transition_map, closure_state_map, accept_state) =
        lr0::transition_map::compute_lr0_transition_map(start_symbol, map_lhs2items);

    let first_sets = first_sets::compute_first_sets(nonterminal_symbols, map_lhs2items);
    let follow_sets =
        follow_sets::compute_follow_sets(nonterminal_symbols, map_lhs2items, &first_sets);

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
        let map = ret.get_mut(&state).unwrap();
        for item in closure {
            if item.is_reducible() {
                for symbol in follow_sets.get(item.get_lhs()).unwrap() {
                    if let Some(_) =
                        map.insert(Some(symbol.to_string()), Action::Reduce(item.clone()))
                    {
                        return Err("shift/reduce or reduce/reduce conflict!".to_string());
                    }
                }
            }
        }
    }

    //accept
    ret.insert(accept_state, HashMap::from([(None, Action::Accept)]));

    Ok(ret)
}
