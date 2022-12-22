use proc_macro::{TokenTree::*, *};
use std::collections::HashSet;

pub fn get_terminal_symbols(arg: Option<TokenTree>) -> HashSet<String> {
    if let Some(Group(grp)) = arg {
        let mut ret: HashSet<_> = grp
            .stream()
            .into_iter()
            .map(|tt| {
                if let Ident(id) = tt {
                    id.to_string()
                } else {
                    panic!()
                }
            })
            .collect();

        ret.insert("F_".to_string());

        ret
    } else {
        panic!()
    }
}
