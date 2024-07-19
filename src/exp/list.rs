use super::Exp;

use std::sync::Arc;
use std::fmt;

pub type List = Option<Arc<Cons>>;

pub struct Cons {
    car: Exp,
    cdr: List,
}

impl fmt::Debug for Cons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} {:?}) ", self.car, self.cdr)
    }
}

impl Cons {
    pub fn new(car: Exp, cdr: &List) -> Cons {
        Cons {car, cdr: cdr.clone()}
    }
}


pub fn push(val: Exp, list: &List) -> List {
    Some(Arc::new(Cons::new(val, list)))
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
        let mut list = Some(Arc::new(Cons::new(last.clone(), &None)));
        for item in rest {
            list = push(item.clone(), &list);
        }
        list
    } else {
        None
    }
}
