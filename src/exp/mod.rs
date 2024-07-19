pub mod lambda;
pub mod list;

pub use lambda::Lambda;
pub use list::Cons;
pub use list::List;

use list::dolist;

use std::{error::Error, fmt, sync::Arc};

pub type LispErr = Box<dyn Error>;
pub type NativeFunction = fn(&[Exp]) -> Result<Exp, LispErr>;
pub type Macro = fn(&[Exp], &Arc<Env>) -> Result<Vec<Exp>, LispErr>;
// Expressions should be trivially copiable.
// In order to do that we need to implement Arc lists.
#[derive(Clone, Debug)]
pub enum Exp {
    List(Option<Arc<Cons>>),
    Num(i64),
    Symbol(String),
    Str(String),
    Vector(Vec<Exp>),
    Lambda(Lambda), // Too big.
    Func(NativeFunction),
    Macro(Macro),
    Bool(bool),
}

pub fn to_bool(exp: &Exp) -> bool {
    match exp {
        Bool(bool) => *bool,
        List(None) => false,
        _ => true,
    }
}
pub use Exp::*;

use crate::env::Env;

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            List(list) => {
                write!(f, "(").unwrap();
                match dolist(list, |exp| {
                    write!(f, "{exp} ").unwrap();
                    Ok(())
                }) {
                    Err(_) => {
                        write!(f, "PRINTING CONS IS UNIMPLEMENTED").unwrap();
                    }
                    _ => {}
                };
                write!(f, ")")
            }
            Num(n) => write!(f, "{n}"),
            Symbol(s) => write!(f, "{s}"),
            Str(s) => write!(f, "{s}"),
            Vector(v) => {
                write!(f, "[").unwrap();
                for element in v.iter() {
                    write!(f, "{element}, ").unwrap();
                }
                write!(f, "]")
            }
            Lambda(lambda) => write!(f, "{lambda:?}"),
            other => write!(f, "{other:?}"),
        }
    }
}
