use crate::eval::{eval, eval_many, eval_lambda_call};
use crate::exp::Lambda;
use crate::exp::*;

use crate::exp::list::list_from_slice;

use std::vec::Vec;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Env {
    pub local: HashMap<String, Exp>,
    upper: Option<Arc<Env>>, 
                             
}

impl Env {
    pub fn new() -> Self {
        Self {
            local: HashMap::new(),
            upper: None,
        }
    }

    pub fn from_upper(upper: &Arc<Env>) -> Self {
        Self {
            local: HashMap::new(),
            upper: Some(Arc::clone(upper)),
        }
    }

    pub fn insert(&mut self, symbol: &str, val: Exp) {
        self.local.insert(symbol.to_string(), val);
    }

    pub fn get(&self, symbol: &str) -> Result<Exp, LispErr> {
        match self.local.get(symbol) {
            Some(exp) => Ok(exp.clone()),
            None => {
                if let Some(upper) = &self.upper {
                    upper.get(symbol)
                } else {
                    let toplevel = TOPLEVEL.read().unwrap();
                    match toplevel.get(symbol) {
                        Some(exp) => Ok(exp.clone()),
                        None => Err(format!("Symbol {symbol} is unbound").into()),
                    }
                }
            }
        }
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TOPLEVEL: RwLock<HashMap<String, Exp>> = RwLock::new(init_toplevel());
}

pub fn set_global(sym: &Exp, val: &Exp) -> Result<(), LispErr> {
    if let Symbol(place) = sym {
        let evaled = eval(val, &Arc::new(Env::new()))?;
        {
            let mut env = TOPLEVEL.write().unwrap();
            env.insert(place.to_string(), evaled);
        }
        Ok(())
    } else {
        Err("Cannot set non-symbol".into())
    }
}

fn init_toplevel() -> HashMap<String, Exp> {
    let mut env = HashMap::new();
    env.insert(
        "+".into(),
        Func(|args, _| {
            let x = args.iter().try_fold(0, |acc, x| match x {
                Num(n) => Ok::<i64, LispErr>(acc + n),
                _ => Err(format!("invalid + argument: {}", x).into()),
            })?;

            Ok(Num(x))
        }),
    );

    env.insert(
        "-".into(),
        Func(|args, _| {
            if let Some((first, rest)) = args.split_first() {
                let first = match first {
                    Num(n) => n,
                    _ => return Err("- arg is not num".into()),
                };

                let x = rest.iter().try_fold(*first, |acc, x| match x {
                    Num(n) => Ok::<i64, LispErr>(acc - n),
                    _ => Err("invalid - argument".into()),
                })?;
                Ok(Num(x))
            } else {
                Err("invalid arguments to -".into())
            }
        }),
    );

    env.insert("nil".into(), List(None));
    env.insert("false".into(), Bool(false));
    env.insert("true".into(), Bool(true));

    env.insert(
        "*".into(),
        Func(|args, _| {
            let x = args.iter().try_fold(1, |acc, x| match x {
                Num(n) => Ok::<i64, LispErr>(acc * n),
                _ => Err("invalid + argument".into()),
            })?;

            Ok(Num(x))
        }),
    );

    env.insert(
        "print".into(),
        Func(|args, _| {
            for arg in args.iter() {
                println!("{arg}");
            }
            Ok(Vector(vec![]))
        }),
    );

    env.insert("progn".into(), Macro(|args, _| Ok(args.to_vec())));

    env.insert(
        "thread/spawn".into(),
        Func(|args, env| {
            use std::thread;
            let Lambda(lambda) = &args[0] else {
                return Err("thread/spawn param is not a function".into());
            };
            let lambda = lambda.clone();
            let env = env.clone();
            thread::spawn(move || {
                let _ = eval_lambda_call(&lambda, &vec![], &env);
            });
            Ok(List(None))
        })
    );

    env.insert(
        "dotimes".into(),
        Func(|args, env| {
            if args.len() != 2 {
                return Err("wrong number of arguments to dotimes".into());
            }
            let Num(times) = &args[0] else {
                return Err("dotimes first param is not a number".into());
            };
            let Lambda(function) = &args[1] else {
                return Err("dotimes second param is not a function".into());
            };
            for i in 0..*times {
                eval_lambda_call(function, &vec![Num(i)], env)?;
            }
            Ok(List(None))
        })
    );

    env.insert(
        "def".into(),
        Macro(|args, _| {
            if args.len() != 2 {
                return Err("Wrong number of arguments to def".into());
            }
            set_global(&args[0], &args[1])?;
            Ok(vec![])
        }),
    );

    fn run_in_let(bindings: &Exp, body: &[Exp], upper_env: &Arc<Env>) -> Result<Exp, LispErr> {
        let Vector(bindings) = bindings else {
            return Err(format!("Expected let binding list, found {:?}", bindings).into());
        };

        let mut let_env = Env::from_upper(upper_env);

        for binding in bindings {
            let Vector(binding) = binding else {
                return Err(
                    "Let bindings should have this format ((name value) (name value)...)".into(),
                );
            };

            if binding.len() != 2 {
                return Err(
                    "Let bindings should have this format ((name value) (name value)...)".into(),
                );
            }

            let Symbol(name) = &binding[0] else {
                return Err("Let cannot bind value to a non-symbol".into());
            };
            let value = eval(&binding[1], upper_env)?;
            let_env.insert(name, value);
        }

        let let_env = Arc::new(let_env);
        eval_many(body, &let_env)
    }

    env.insert(
        "let".into(),
        Macro(|args, env| {
            if args.len() < 2 {
                return Err("Not enough arguments passed to let".into());
            }
            let bindings = &args[0];
            let body = &args[1..];
            let res = run_in_let(bindings, body, env)?;
            Ok(vec![res])
        }),
    );

    env.insert(
        "assert".into(),
        Func(|args, _| {
            for x in args {
                assert!(to_bool(x));
            }
            Ok(List(None))
        }),
    );

    env.insert(
        "cons".into(),
        Func(|args, _| {
            if args.len() != 2 {
                return Err("Wrong arguments to cons".into());
            }
            Ok(List(Some(Arc::new(Cons::new(
                args[0].clone(),
                args[1].clone(),
            )))))
        }),
    );

    env.insert("list".into(), Func(|args, _| Ok(List(list_from_slice(args)))));

    env.insert(
        "car".into(),
        Func(|args, _| {
            if args.len() != 1 {
                return Err("Wrong arguments to car".into());
            }
            match &args[0] {
                // car of nil is nil
                List(None) => Ok(List(None)),
                List(Some(list)) => Ok(list.car.clone()),
                _ => Err("car argument is not a list".into()),
            }
        }),
    );

    env.insert(
        "cdr".into(),
        Func(|args, _| {
            if args.len() != 1 {
                return Err("Wrong arguments to cdr".into());
            }
            match &args[0] {
                // cdr of nil is nil
                List(None) => Ok(List(None)),
                List(Some(list)) => Ok(list.cdr.clone()),
                _ => Err("car argument is not a list".into()),
            }
        }),
    );

    env.insert(
        "defun".into(),
        Macro(|args, _| {
            if let Vector(lambda_list) = &args[1] {
                let mut llist: Vec<String> = vec![];
                for arg in lambda_list {
                    llist.push(match arg {
                        Symbol(str) => str.clone(),
                        _ => return Err("Invalid lambda list".into()),
                    });
                }
                let body = Vec::from(&args[2..]);
                set_global(&args[0], &Lambda(Lambda::new(llist, body)))?;
                Ok(vec![])
            } else {
                return Err("Invalid lambda list".into());
            }
        }),
    );

    env.insert(
        "if".into(),
        Macro(|args, env| {
            let evaled = eval(&args[0], env)?;
            let is_true = to_bool(&evaled);
            if is_true {
                Ok(vec![args[1].clone()])
            } else {
                Ok(vec![args[2].clone()])
            }
        }),
    );

    env.insert(
        "or".into(),
        Func(|args, _| {
            if args.is_empty() {
                return Err("or with empty args".into());
            }

            for arg in args {
                let x: bool = to_bool(arg);
                if x == true {
                    return Ok(Bool(true));
                }
            }
            Ok(Bool(false))
        }),
    );

    env.insert(
        "=".into(),
        Func(|args, _| {
            if args.is_empty() {
                return Err("no arguments".into());
            }

            let first: i64 = match args[0] {
                Num(n) => n,
                _ => return Err("= arg is not a number".into()),
            };

            let equal = args.iter().all(|x| match x {
                Num(n) => *n == first,
                _ => panic!("= arg is not a number"),
            });

            Ok(Bool(equal))
        }),
    );

    env.insert(
        "lambda".into(),
        Macro(|args, _| {
            let lam = Lambda::from_list(args)?;
            Ok(vec![Lambda(lam)])
        }),
    );
    env
}
