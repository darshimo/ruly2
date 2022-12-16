use proc_macro::{TokenTree, TokenTree::*};

pub fn get_ident_string(arg: Option<TokenTree>) -> String {
    if let Some(Ident(id)) = arg {
        let s = id.to_string();
        if s.contains("_") {
            panic!("The identifiers are not allowd to contain \"_\"! ({})", s);
        }
        s
    } else {
        panic!()
    }
}
