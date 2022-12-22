use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{
    item::{Item, LR1Closure},
    parser::{first_sets, Action},
};

pub fn compute_lr1_parsing_table(
    start_symbol: &str,
    nonterminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>, // lhs -> 左辺がlhsであり，かつポインタが左端にあるitemの集合
) -> Result<HashMap<usize, HashMap<Option<String>, Action>>, String> {
    let first_sets = &first_sets::compute_first_sets(nonterminal_symbols, map_lhs2items);

    let initial_item = Item::from(
        (0, "S_0".to_string()),
        "S_".to_string(),
        vec![start_symbol.to_string(), "F_".to_string()],
        0,
    );
    let accept_item = Item::from(
        (0, "S_0".to_string()),
        "S_".to_string(),
        vec![start_symbol.to_string(), "F_".to_string()],
        2,
    );

    let initial_closure = LR1Closure::from(
        [(initial_item, BTreeSet::from([None]))],
        map_lhs2items,
        first_sets,
    );
    let accept_closure = LR1Closure::from(
        [(accept_item, BTreeSet::from([None]))],
        map_lhs2items,
        first_sets,
    );

    let mut parsing_table = HashMap::new();
    let mut closure_state_map = HashMap::new();
    rec(
        initial_closure,
        &map_lhs2items,
        &mut parsing_table,
        &mut closure_state_map,
        first_sets,
    )?;

    let &accept_state = closure_state_map.get(&accept_closure).unwrap();

    parsing_table
        .get_mut(&accept_state)
        .unwrap()
        .insert(None, Action::Accept);

    Ok(parsing_table)
}

fn rec(
    closure: LR1Closure,
    map_lhs2items: &HashMap<String, HashSet<Item>>, // lhs -> 左辺がlhsのitemの集合
    parsing_table: &mut HashMap<usize, HashMap<Option<String>, Action>>,
    closure_state_map: &mut HashMap<LR1Closure, usize>,
    first_sets: &HashMap<Vec<String>, HashSet<Option<String>>>,
) -> Result<usize, String> {
    if let Some(&closure_num) = closure_state_map.get(&closure) {
        return Ok(closure_num);
    }

    let closure_num = closure_state_map.len();
    closure_state_map.insert(closure.clone(), closure_num);
    parsing_table.insert(closure_num, HashMap::new());

    // Map<読む文字, 次のclosureに含まれる(item,先読み文字の集合)>
    let mut nexts = HashMap::new();

    for (mut item, lookahead_set) in closure.clone() {
        if let Some(x) = item.shift() {
            // シフト項の場合
            let next_item = item;
            let next_lookahead_set = lookahead_set;
            nexts
                .entry(x)
                .or_insert(BTreeMap::new())
                .insert(next_item, next_lookahead_set);
        } else {
            // 還元項の場合
            let reducible_item = item;
            for lookahead_symbol in lookahead_set {
                if let Some(_) = parsing_table
                    .get_mut(&closure_num)
                    .unwrap()
                    .insert(lookahead_symbol, Action::Reduce(reducible_item.clone()))
                {
                    return Err("reduce/reduce conflict!".to_string());
                }
            }
        }
    }

    for (c, v) in nexts {
        let next_closure = LR1Closure::from(v, map_lhs2items, first_sets);
        let next_closure_num = rec(
            next_closure,
            map_lhs2items,
            parsing_table,
            closure_state_map,
            first_sets,
        )?;
        if let Some(_) = parsing_table
            .get_mut(&closure_num)
            .unwrap()
            .insert(Some(c), Action::Shift(next_closure_num))
        {
            return Err("shift/reduce conflict!".to_string());
        }
    }

    Ok(closure_num)
}
