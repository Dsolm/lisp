pub mod lambda;
pub mod list;

pub use list::Cons;
pub use list::List;
pub use lambda::Lambda;

use std::{error::Error, sync::Arc};

pub type LispErr = Box<dyn Error>;
pub type NativeFunction = fn(&[Exp]) -> Result<Exp, LispErr>;
pub type Macro = fn(&[Exp], &Arc<Env>) -> Result<Vec<Exp>, LispErr>;
// Expressions should be trivially copiable.
// In order to do that we need to implement Arc lists.
#[derive(Clone, Debug)]
pub enum Exp {
    Vector(Option<Arc<Cons>>),
    Num(i32),
    Symbol(String),
    // TODO: List is Rc, shared immutable
    List(Vec<Exp>),
    Lambda(Lambda), // Too big.
    Func(NativeFunction),
    Macro(Macro),
    Bool(bool),
}

pub use Exp::*;

use crate::env::Env;
