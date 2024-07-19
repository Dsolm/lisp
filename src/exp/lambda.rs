use core::fmt;
use std::iter::zip;

use crate::{eval::eval_many, exp::*};

#[derive(Clone)]
pub struct Lambda {
    args: Vec<String>,
    body: Vec<Exp>,
}

impl Lambda {
    pub fn new(args: Vec<String>, body: Vec<Exp>) -> Lambda {
        Lambda { args, body }
    }

    pub fn from_list(args: &[Exp]) -> Result<Lambda, LispErr> {
        if let List(lambda_list) = &args[0] {
            let mut llist: Vec<String> = vec![];
            for arg in lambda_list {
                llist.push(match arg {
                    Symbol(str) => str.clone(),
                    _ => return Err("Invalid lambda list".into()),
                });
            }

            let body = Vec::from(&args[1..]);
            Ok(Lambda::new(llist, body))
        } else {
            return Err("Invalid lambda list".into());
        }
    }

    pub fn call(self: &Self, args: Vec<Exp>, env: &Arc<Env>) -> Result<Exp, LispErr> {
        if self.args.len() != args.len() {
            return Err("Wrong number of function arguments".into());
        }

        let mut inner_env = Env::from_upper(env);

        inner_env.local.extend(zip(self.args.clone(), args));
        let inner_env = Arc::new(inner_env);
        eval_many(&self.body, &inner_env)
    }
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lambda with arguments: {:?}", self.args)
    }
}
