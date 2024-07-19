use crate::exp::Exp;
use crate::exp::LispErr;

use std::fmt;
use std::sync::Arc;

pub type List = Option<Arc<Cons>>;

pub struct Cons {
    pub car: Exp,
    pub cdr: Exp,
}

impl fmt::Debug for Cons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} . {:?}) ", self.car, self.cdr)
    }
}

impl fmt::Display for Cons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} . {}) ", self.car, self.cdr)
    }
}

impl Cons {
    pub fn new(car: Exp, cdr: Exp) -> Cons {
        Cons { car: car, cdr: cdr }
    }
}

pub fn push(val: Exp, list: List) -> List {
    Some(Arc::new(Cons {
        car: val,
        cdr: Exp::List(list),
    }))
}

pub fn next(list: &List) -> List {
    match list {
        // The CDR of nil is nil, like in common lisp.
        None => None,
        Some(list) => {
            return Some(list.clone());
        }
    }
}

pub fn list_from_slice(vec: &[Exp]) -> List {
    if let Some((last, rest)) = vec.split_last() {
        let mut list = Some(Arc::new(Cons::new(last.clone(), Exp::List(None))));
        for item in rest.iter().rev() {
            list = push(item.clone(), list);
        }
        list
    } else {
        None
    }
}

pub fn print_list(list: &List) -> Result<(), LispErr> {
    println!("(");
    dolist(list, |exp| {
        println!("{exp} ");
        Ok(())
    })?;
    println!(")");
    Ok(())
}

pub fn dolist(
    list: &List,
    mut func: impl FnMut(&Exp) -> Result<(), LispErr>,
) -> Result<(), LispErr> {
    match list {
        Some(list) => {
            func(&list.car)?;
            match &list.cdr {
                Exp::List(cdr) => dolist(&cdr, func)?,
                _ => return Err("dolist argument is not a list".into()),
            }
        }
        None => {}
    }
    Ok(())
}
