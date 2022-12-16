use std::collections::{BTreeSet, HashMap, HashSet};

use crate::item::{Item, LR0Closure};

pub fn compute_table(
    start_symbol: &str,
    map_lhs2items: &HashMap<String, HashSet<Item>>, // lhs -> 左辺がlhsであり，かつポインタが左端にあるitemの集合
) -> (
    HashMap<usize, HashMap<String, usize>>,
    HashMap<LR0Closure, usize>,
    usize,
) {
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

    let initial_closure = LR0Closure::from([initial_item], map_lhs2items);
    let accept_closure = LR0Closure::from([accept_item], map_lhs2items);

    let mut parsing_table = HashMap::new();
    let mut closure_state_map = HashMap::new();
    rec(
        initial_closure,
        &map_lhs2items,
        &mut parsing_table,
        &mut closure_state_map,
    );

    let &accept_state = closure_state_map.get(&accept_closure).unwrap();

    (parsing_table, closure_state_map, accept_state)
}

fn rec(
    closure: LR0Closure,
    map_lhs2items: &HashMap<String, HashSet<Item>>, // lhs -> 左辺がlhsのitemの集合
    parsing_table: &mut HashMap<usize, HashMap<String, usize>>,
    closure_state_map: &mut HashMap<LR0Closure, usize>,
) -> usize {
    if let Some(&closure_num) = closure_state_map.get(&closure) {
        return closure_num;
    }

    let closure_num = closure_state_map.len();
    closure_state_map.insert(closure.clone(), closure_num);
    parsing_table.insert(closure_num, HashMap::new());

    // 読む文字と次のitemのvdqのマップ
    let mut nexts = HashMap::new();

    for item in &closure {
        if let Some(x) = item.get_symbol_under_pointer() {
            let mut next_item = item.clone();
            next_item.inc_pointer();
            nexts.entry(x).or_insert(BTreeSet::new()).insert(next_item);
        }
    }

    for (c, v) in nexts {
        let next_closure = LR0Closure::from(v, map_lhs2items);
        let next_closure_num = rec(
            next_closure,
            map_lhs2items,
            parsing_table,
            closure_state_map,
        );
        parsing_table
            .entry(closure_num)
            .or_insert(HashMap::new())
            .insert(c, next_closure_num);
    }

    closure_num
}
