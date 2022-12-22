pub use lr_parser::*;
pub use once_cell::sync::Lazy;
pub use regex::Regex;

#[macro_export]
macro_rules! syntax {
    (
        WHITESPACE $tt1:tt
        TOKEN $tt2:tt
        RULE $tt3:tt
        START { $i1:tt }
        ALGORITHM { $i2:ident }
    ) => {
        declare_whitespace_regex!($tt1);
        declare_token_extractors!($tt2);
        impl_token!($tt2);
        impl_terminal_symbol!($tt2);
        impl_nonterminal_symbol!($tt3);

        impl_lex!();
        impl_yacc!($tt2, $tt3, $i1, $i2);

        impl_parser!($i1);
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
        #[derive(Debug, Clone)]
        enum Token {
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
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $i(String);
            impl $i {
                pub fn as_str<'a>(&'a self) -> &'a str {
                    &self.0
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_parsablell_for_terminal_symbol {
    ( { $( $i:ident => $tt:tt );*; } ) => {
        $(
            impl ParsableLL for $i {
                fn parse_ll(v: &Vec<Token>, idx: &mut usize) -> Result<Self, String> {
                    if let Some(Token::$i(x)) = v.get(*idx) {
                        *idx += 1;
                        return Ok(x.clone());
                    } else {
                        Err("ParseError!".to_string())
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_nonterminal_symbol {
    ( { $( $i1:ident => $( $i2:ident ( $($tt:tt),* ) )|+ );*; } ) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum $i1 {
                $( $i2( $( Box<$tt> ),* ) ),*,
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_parsablell_for_nonterminal_symbol {
    ( { $( $i1:ident => $( $i2:ident ( $($tt:tt),* ) )|+ );*; } ) => {
        $(
            impl ParsableLL for $i1 {
                fn parse_ll(v: &Vec<Token>, idx: &mut usize) -> Result<Self, String> {
                    let start_idx = *idx;

                    $(
                        let mut c = || -> Result<Self, String> {
                            Ok($i1::$i2(
                                $( Box::new($tt::parse_ll(v, idx)?) ),*,
                            ))
                        };
                        if let Ok(x) = c() {
                            return Ok(x);
                        }
                        *idx = start_idx;
                    )*

                    Err("ParseError!".to_string())
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_lex {
    () => {
        struct Lex;

        impl Lex {
            fn tokenize(s: &str) -> Result<Vec<Token>, String> {
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
macro_rules! define_parsablell {
    () => {
        trait ParsableLL: Sized {
            fn parse_ll(v: &Vec<Token>, idx: &mut usize) -> Result<Self, String>;
        }
    };
}

#[macro_export]
macro_rules! define_yacc {
    () => {
        struct Yacc;
    };
}

#[macro_export]
macro_rules! impl_parser_ll {
    ( $i:ident ) => {
        impl Yacc {
            fn parse(v: &Vec<Token>) -> Result<$i, String> {
                let mut idx = 0;
                let result = $i::parse_ll(&v, &mut idx)?;
                if idx == v.len() {
                    Ok(result)
                } else {
                    Err("ParseError!".to_string())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_yacc {
    ( $tt1:tt , $tt2:tt , $i1:ident , LL ) => {
        impl_parsablell_for_terminal_symbol!($tt1);
        impl_parsablell_for_nonterminal_symbol!($tt2);
        define_parsablell!();
        define_yacc!();
        impl_parser_ll!($i1);
    };

    ( { $( $i1:ident => $tt1:tt );*; } , { $( $i2:ident => $( $i3:ident ( $($tt2:tt),* ) )|+ );*; } , $i4:ident , $i5:ident ) => {
        define_yacc!();
        impl_lr_parser!( $i5 $i4 { $( $i1 )* } { $( { $i2 $( { $i3 ( $( $tt2 )* ) } )* } )* } );
    };
}

#[macro_export]
macro_rules! impl_parser {
    ( $i:ident ) => {
        pub struct Parser;
        impl Parser {
            pub fn parse(s: &str) -> Result<$i, String> {
                let v = Lex::tokenize(s)?;
                Yacc::parse(&v)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_ll() {
        use crate::*;

        syntax!(
            WHITESPACE {
                r"[ \n\r\t]*"
            }

            TOKEN {
                Cons => {r"::"};
                Nil => {r"\[\]"};
                Num => {r"[1-9][0-9]*"};
                Op => {r"\*|\+"};
            }

            RULE {
                List => List0(Term, Cons, List)
                    | List1(Nil);

                Term => Term0(Num, Op, Term)
                    | Term1(Num);
            }

            START {
                List
            }

            ALGORITHM {
                LL
            }
        );

        let s = "1+2::3*4::[]";
        let result = Parser::parse(s);
        let expected = List::List0(
            Box::new(Term::Term0(
                Box::new(Num(1.to_string())),
                Box::new(Op("+".to_string())),
                Box::new(Term::Term1(Box::new(Num(2.to_string())))),
            )),
            Box::new(Cons("::".to_string())),
            Box::new(List::List0(
                Box::new(Term::Term0(
                    Box::new(Num(3.to_string())),
                    Box::new(Op("*".to_string())),
                    Box::new(Term::Term1(Box::new(Num(4.to_string())))),
                )),
                Box::new(Cons("::".to_string())),
                Box::new(List::List1(Box::new(Nil("[]".to_string())))),
            )),
        );

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_lr0() {
        use crate::*;

        syntax!(
            WHITESPACE {
                r"[ \n\r\t]*"
            }

            TOKEN {
                L => {r"<"};
                R => {r">"};
            }

            RULE {
                S => S0(A, A);

                A => A0(L, A, R)
                    | A1(L, R);
            }

            START {
                S
            }

            ALGORITHM {
                LR0
            }
        );

        let s = "<<>><>";
        let result = Parser::parse(s);

        let a_lr = A::A1(Box::new(L("<".to_string())), Box::new(R(">".to_string())));
        let a_llrr = A::A0(
            Box::new(L("<".to_string())),
            Box::new(a_lr.clone()),
            Box::new(R(">".to_string())),
        );
        let expected = S::S0(Box::new(a_llrr), Box::new(a_lr));

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_slr() {
        use crate::*;

        syntax!(
            WHITESPACE {
                r"[ \n\r\t]*"
            }

            TOKEN {
                P => {r"\+"};
                M => {r"\*"};
                N => {"[1-9][0-9]*"};
            }

            RULE {
                E => E0(E, P, T)
                    | E1(T);

                T => T0(T, M, N)
                    | T1(N);
            }

            START {
                E
            }

            ALGORITHM {
                SLR
            }
        );

        let s = "1*2*3+4*5+6";
        let result = Parser::parse(s);

        let t_1m2 = T::T0(
            Box::new(T::T1(Box::new(N(1.to_string())))),
            Box::new(M("*".to_string())),
            Box::new(N(2.to_string())),
        );
        let t_1m2m3 = T::T0(
            Box::new(t_1m2),
            Box::new(M("*".to_string())),
            Box::new(N(3.to_string())),
        );
        let t_4m5 = T::T0(
            Box::new(T::T1(Box::new(N(4.to_string())))),
            Box::new(M("*".to_string())),
            Box::new(N(5.to_string())),
        );
        let t_6 = T::T1(Box::new(N(6.to_string())));
        let expected = E::E0(
            Box::new(E::E0(
                Box::new(E::E1(Box::new(t_1m2m3))),
                Box::new(P("+".to_string())),
                Box::new(t_4m5),
            )),
            Box::new(P("+".to_string())),
            Box::new(t_6),
        );

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_lr1() {
        use crate::*;

        syntax!(
            WHITESPACE {
                r"[ \n\r\t]*"
            }

            TOKEN {
                Eq => {"="};
                P => {r"\+"};
                N => {"[1-9][0-9]*"};
                Id => {"[a-z]+"};
            }

            RULE {
                A => A0(E, Eq, E)
                    | A1(Id);

                E => E0(E, P, T)
                    | E1(T);

                T => T0(N)
                    | T1(Id);
            }

            START {
                A
            }

            ALGORITHM {
                LR1
            }
        );

        let s = "x+2+y=4+z";
        let result = Parser::parse(s);
        let e_xp2 = E::E0(
            Box::new(E::E1(Box::new(T::T1(Box::new(Id("x".to_string())))))),
            Box::new(P("+".to_string())),
            Box::new(T::T0(Box::new(N(2.to_string())))),
        );
        let e_xp2py = E::E0(
            Box::new(e_xp2),
            Box::new(P("+".to_string())),
            Box::new(T::T1(Box::new(Id("y".to_string())))),
        );
        let e_4pz = E::E0(
            Box::new(E::E1(Box::new(T::T0(Box::new(N(4.to_string())))))),
            Box::new(P("+".to_string())),
            Box::new(T::T1(Box::new(Id("z".to_string())))),
        );
        let expected = A::A0(
            Box::new(e_xp2py),
            Box::new(Eq("=".to_string())),
            Box::new(e_4pz),
        );

        assert_eq!(result, Ok(expected));
    }
}
