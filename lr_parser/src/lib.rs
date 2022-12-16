use proc_macro::*;

mod common;
mod item;
mod rule;
mod table;
mod token;

#[proc_macro]
pub fn impl_lr_parser(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let algorithm = common::get_ident_string(iter.next());

    let start_symbol = common::get_ident_string(iter.next());

    let terminal_symbols = token::get_terminal_symbols(iter.next());

    let (nonterminal_symbols, map_lhs2items) = rule::parse_rule(&start_symbol, iter.next());

    let (parsing_table, closure_state_map, accept_state) =
        table::compute_table(&start_symbol, &map_lhs2items);

    "".to_string().parse().unwrap()
}
