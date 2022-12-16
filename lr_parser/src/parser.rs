use std::collections::{HashMap, HashSet};

use crate::item::Item;

mod lr0;

#[derive(Debug)]
pub enum Action {
    Shift(usize),
    Reduce(Item),
    Accept,
}

pub fn create_parser(
    algorithm: &str,
    start_symbol: &str,
    terminal_symbols: &HashSet<String>,
    nonterminal_symbols: &HashSet<String>,
    map_lhs2items: &HashMap<String, HashSet<Item>>,
) -> String {
    let result = match &*algorithm {
        "LR0" => lr0::parsing_table::compute_lr0_parsing_table(
            start_symbol,
            terminal_symbols,
            map_lhs2items,
        ),

        _ => panic!(),
    };

    match result {
        Ok(parsing_table) => {
            let mut ret = String::new();

            ret.push_str(&enum_status(start_symbol));
            ret.push_str(&struct_automaton());
            ret.push_str(&impl_automaton(start_symbol, &parsing_table));
            ret.push_str(&impl_yacc(start_symbol));
            ret.push_str(&enum_tree(terminal_symbols, nonterminal_symbols));
            ret.push_str(&impl_tree(terminal_symbols));

            ret
        }

        Err(_) => {
            format!(
                "
impl Yacc {{
    pub fn parse(v: &Vec<Token>) -> Result<{}, ()> {{
        Err(())
    }}
}}",
                start_symbol
            )
        }
    }
}

fn enum_status(start_symbol: &str) -> String {
    format!(
        "
enum Status {{
    Finished({}),
    Running,
}}",
        start_symbol
    )
}

fn struct_automaton() -> String {
    "
struct Automaton {
    input: std::collections::VecDeque<Tree>,
    state_stack: Vec<usize>,
    symbol_stack: Vec<Tree>,
}"
    .to_string()
}

fn impl_automaton(
    start_symbol: &str,
    parsing_table: &HashMap<usize, HashMap<Option<String>, Action>>,
) -> String {
    let mut ret = String::new();

    ret.push_str(
        "
impl Automaton {",
    );

    ret.push_str(&fn_new());

    ret.push_str(&fn_step(parsing_table, start_symbol));

    ret.push_str(&fn_run(start_symbol));

    ret.push_str(
        "
}",
    );

    ret
}

fn fn_new() -> String {
    "
    fn new(input: Vec<Tree>) -> Self {
        let mut input: std::collections::VecDeque<_> = input.into_iter().collect();
        input.push_back(Tree::F_(()));
        Automaton {
            input,
            state_stack: vec![0],
            symbol_stack: vec![],
        }
    }"
    .to_string()
}

fn shift(from: usize, x: &str, to: usize) -> String {
    format!(
        "
            (Some({}), Some(Tree::{}(_))) => {{
                self.state_stack.push({});
                self.symbol_stack.push(self.input.pop_front().unwrap());
                return Status::Running;
            }}",
        from, x, to
    )
}

fn reduce(from: usize, x: &str, item: &Item) -> String {
    let field_num = item.get_rhs().len();

    let s1 = item
        .get_rhs()
        .iter()
        .enumerate()
        .rev()
        .fold("".to_string(), |mut s, (i, symbol)| {
            s.push_str(&format!("Some(Tree::{}(t{})), ", symbol, i));
            s
        });
    let s2 = (0..field_num).fold("".to_string(), |mut s, i| {
        s.push_str(&format!("Box::new(t{}), ", i));
        s
    });

    format!(
        "
            (Some({}), Some(Tree::{}(_))) => {{
                if let ({}) = ({}) {{
                    if let ({}) = ({}) {{
                        self.input.push_front(Tree::{}({}::{}({})));
                        return Status::Running;
                    }}
                }}
            }}",
        from,
        x,
        "Some(_), ".repeat(item.get_rhs().len()),
        "self.state_stack.pop(), ".repeat(item.get_rhs().len()),
        s1,
        "self.symbol_stack.pop(), ".repeat(item.get_rhs().len()),
        item.get_lhs(),
        item.get_lhs(),
        item.get_rule_name(),
        s2
    )
}

fn accept(from: usize, start_symbol: &str) -> String {
    format!(
        "
            (Some({}), None) => {{
                if let Some(Tree::F_(_)) = self.symbol_stack.pop() {{
                    if let Some(Tree::{}(x)) = self.symbol_stack.pop() {{
                        return Status::Finished(x);
                    }}
                }}
            }}",
        from, start_symbol
    )
}

fn fn_step(
    parsing_table: &HashMap<usize, HashMap<Option<String>, Action>>,
    start_symbol: &str,
) -> String {
    let mut ret = String::new();

    ret.push_str(
        "
    fn step(&mut self) -> Status {
        match (self.state_stack.last(), self.input.front()) {",
    );

    for (from, map) in parsing_table {
        for pair in map {
            let s = match pair {
                (Some(x), Action::Shift(to)) => shift(*from, x, *to),
                (Some(x), Action::Reduce(item)) => reduce(*from, x, item),
                (None, Action::Accept) => accept(*from, start_symbol),
                _ => panic!(),
            };

            ret.push_str(&s);
        }
    }

    ret.push_str(
        "

            _ => {}
        }

        panic!()
    }",
    );

    ret
}

fn fn_run(start_symbol: &str) -> String {
    format!(
        "
    fn run(&mut self) -> Result<{}, ()> {{
        loop {{
            if let Status::Finished(t) = self.step() {{
                return Ok(t);
            }}
        }}
    }}",
        start_symbol
    )
}

fn impl_yacc(start_symbol: &str) -> String {
    format!(
        "
impl Yacc {{
    pub fn parse(v: &Vec<Token>) -> Result<{}, ()> {{
        let v: Vec<_> = v.iter().map(|x| Tree::from(x)).collect();
        let mut automaton = Automaton::new(v);

        automaton.run()
    }}
}}",
        start_symbol
    )
}

fn enum_tree(terminal_symbols: &HashSet<String>, nonterminal_symbols: &HashSet<String>) -> String {
    let mut ret = String::new();

    ret.push_str(
        "
#[derive(Debug)]
enum Tree {",
    );

    for symbol in terminal_symbols {
        if symbol == "F_" {
            continue;
        }

        ret.push_str(&format!(
            "
    {}({}),",
            symbol, symbol
        ));
    }

    for symbol in nonterminal_symbols {
        if symbol == "S_" {
            continue;
        }

        ret.push_str(&format!(
            "
    {}({}),",
            symbol, symbol
        ));
    }

    ret.push_str(
        "
    F_(()),
}",
    );

    ret
}

fn impl_tree(terminal_symbols: &HashSet<String>) -> String {
    let mut ret = String::new();

    ret.push_str(
        "
impl Tree {
    fn from(t: &Token) -> Self {
        match t {",
    );

    for symbol in terminal_symbols {
        if symbol == "F_" {
            continue;
        }

        ret.push_str(&format!(
            "
            Token::{}(x) => Tree::{}(x.clone()),",
            symbol, symbol
        ));
    }

    ret.push_str(
        "
        }
    }
}",
    );

    ret
}
