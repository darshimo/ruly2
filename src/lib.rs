pub use once_cell::sync::Lazy;
pub use regex::Regex;

#[macro_export]
macro_rules! syntax {
    (
        WHITESPACE $tt1:tt
        TOKEN $tt2:tt
        RULE $tt3:tt
        START $tt4:tt
    ) => {
        declare_whitespace_regex!($tt1);
        declare_token_extractors!($tt2);
        impl_token!($tt2);
        impl_terminal_symbol!($tt2);
        impl_nonterminal_symbol!($tt3);
        impl_lex!();
        impl_yacc!();
    };
}

#[macro_export]
macro_rules! declare_whitespace_regex {
    ( { $e:expr } ) => {
        static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| {
            let regex = Regex::new($e).unwrap();

            if !regex.is_match("") {
                panic!("The empty string does not match for the WHITESPACE regex!");
            }

            regex
        });
    };
}

#[macro_export]
macro_rules! push_closure {
    ( $ret:ident, $i:ident, { $e:expr } ) => {
        $ret.not_reserved.push({
            let regex = Regex::new($e).unwrap();
            Box::new(move |pos: usize, s: &str| {
                if let Some(mat) = regex.find_at(s, pos) {
                    if pos == mat.start() {
                        return Some(Token::$i($i(mat.as_str().to_string())));
                    }
                }
                None
            })
        });
    };
    ( $ret:ident, $i:ident, { $e:expr, Reserved } ) => {
        $ret.reserved.push({
            let regex = Regex::new($e).unwrap();
            Box::new(move |pos: usize, s: &str| {
                if let Some(mat) = regex.find_at(s, pos) {
                    if pos == mat.start() {
                        return Some(Token::$i($i(mat.as_str().to_string())));
                    }
                }
                None
            })
        });
    };
}

#[macro_export]
macro_rules! declare_token_extractors {
    ( { $( $i:ident => $tt:tt );*; } ) => {

        struct Closures {
            reserved: Vec<Box<dyn Fn(usize, &str) -> Option<Token> + Send + Sync>>,
            not_reserved: Vec<Box<dyn Fn(usize, &str) -> Option<Token> + Send + Sync>>,
        }

        impl Closures {
            fn new() -> Self {
                Self {
                    reserved: vec![],
                    not_reserved: vec![],
                }
            }
        }

        static TOKEN_EXTRACTORS: Lazy<Closures> = Lazy::new(|| {
            let mut ret = Closures::new();

            $(
                push_closure!(ret, $i, $tt);
            )*

            ret
        });
    };
}

#[macro_export]
macro_rules! impl_token {
    ( { $( $i:ident => $tt:tt );*; } ) => {
        #[derive(Debug)]
        pub enum Token {
            $(
                $i($i),
            )*
        }

        impl Token {
            fn get_str(&self) -> String {
                match self {
                $(
                    Token::$i($i(s)) => s.to_string(),
                )*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_terminal_symbol {
    ( { $( $i:ident => $tt:tt );*; } ) => {
        $(
            #[derive(Debug)]
            pub struct $i(String);
        )*
    };
}

#[macro_export]
macro_rules! impl_nonterminal_symbol {
    ( { $( $i1:ident => $( $i2:ident ( $($tt:tt),* ) )|+ );*; } ) => {
        $(
            #[derive(Debug)]
            pub enum $i1 {
                $( $i2( $( Box<$tt> ),* ) ),*,
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_lex {
    () => {
        pub struct Lex;

        impl Lex {
            pub fn tokenize(s: &str) -> Result<Vec<Token>, String> {
                let mut ret = vec![];

                let mut current_pos = 0;

                Self::skip(&s, &mut current_pos);

                while current_pos < s.len() {
                    if let Some(token) = Self::find_and_split(&s, &mut current_pos) {
                        ret.push(token);
                    } else {
                        return Err(format!(
                            "TokenizeError at Col {}: \"{}\"",
                            current_pos,
                            &s[current_pos..std::cmp::min(current_pos + 30, s.len())]
                        ));
                    }

                    Self::skip(&s, &mut current_pos);
                }

                Ok(ret)
            }

            fn find_and_split(s: &str, current_pos: &mut usize) -> Option<Token> {
                if let Some(token) = Self::find_not_reserved(s, current_pos) {
                    let word = token.get_str();
                    *current_pos += word.len();
                    if let Some(token_reserved) = Self::is_match_with_reserved(&word) {
                        Some(token_reserved)
                    } else {
                        Some(token)
                    }
                } else {
                    None
                }
            }

            fn find_not_reserved(s: &str, current_pos: &mut usize) -> Option<Token> {
                for closure in TOKEN_EXTRACTORS.not_reserved.iter() {
                    if let Some(token) = closure(*current_pos, s) {
                        return Some(token);
                    }
                }

                None
            }

            fn is_match_with_reserved(word: &str) -> Option<Token> {
                for closure in TOKEN_EXTRACTORS.reserved.iter() {
                    if let Some(token) = closure(0, &word) {
                        if word == token.get_str() {
                            return Some(token);
                        }
                    }
                }

                None
            }

            fn skip(s: &str, current_pos: &mut usize) {
                if let Some(mat) = WHITESPACE_REGEX.find_at(&s, *current_pos) {
                    if *current_pos == mat.start() {
                        *current_pos = mat.end();
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_yacc {
    () => {
        pub struct Yacc;
    };
}

#[macro_export]
macro_rules! impl_yacc {
    () => {
        define_yacc!();
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
