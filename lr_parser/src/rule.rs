use proc_macro::{TokenTree, TokenTree::*};
use std::collections::{HashMap, HashSet};

use crate::common::get_ident_string;
use crate::item::Item;

pub fn parse_rule(
    start_symbol: &str,
    arg: Option<TokenTree>,
) -> (HashSet<String>, HashMap<String, HashSet<Item>>) {
    let mut cnt = 0;
    let mut tmp = f0(arg, &mut cnt);

    let mut ret1: HashSet<_> = tmp.iter().map(|(s, _)| s.clone()).collect();
    ret1.insert("S_".to_string());

    tmp.insert(
        "S_".to_string(),
        vec![(
            (0, "S_0".to_string()),
            vec![start_symbol.to_string(), "F_".to_string()],
        )],
    );
    let ret2 = f4(tmp);

    (ret1, ret2)
}

fn f0(
    arg: Option<TokenTree>,
    cnt: &mut usize,
) -> HashMap<String, Vec<((usize, String), Vec<String>)>> {
    if let Some(Group(grp)) = arg {
        grp.stream().into_iter().map(|tt| f1(tt, cnt)).collect()
    } else {
        panic!()
    }
}

fn f1(arg: TokenTree, cnt: &mut usize) -> (String, Vec<((usize, String), Vec<String>)>) {
    if let Group(grp) = arg {
        let mut it = grp.stream().into_iter();
        let left = get_ident_string(it.next());
        let v = it.map(|tt| f2(tt, cnt)).collect();
        (left, v)
    } else {
        panic!()
    }
}

fn f2(arg: TokenTree, cnt: &mut usize) -> ((usize, String), Vec<String>) {
    if let Group(grp) = arg {
        let mut it = grp.stream().into_iter();
        let rule = get_ident_string(it.next());
        let v = f3(it.next());
        *cnt += 1;
        ((*cnt, rule), v)
    } else {
        panic!()
    }
}

fn f3(arg: Option<TokenTree>) -> Vec<String> {
    if let Some(Group(grp)) = arg {
        grp.stream()
            .into_iter()
            .map(|tt| get_ident_string(Some(tt)))
            .collect()
    } else {
        panic!()
    }
}

fn f4(arg: HashMap<String, Vec<((usize, String), Vec<String>)>>) -> HashMap<String, HashSet<Item>> {
    let mut ret = HashMap::new();

    for (lhs, v) in arg {
        for ((rule_id, rule_name), rhs) in v {
            ret.entry(lhs.clone())
                .or_insert(HashSet::new())
                .insert(Item::from(
                    (rule_id, rule_name.clone()),
                    lhs.clone(),
                    rhs,
                    0,
                ));
        }
    }

    ret
}
