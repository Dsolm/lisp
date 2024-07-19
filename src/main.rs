use core::fmt;
use std::iter::zip;
use std::{collections::HashMap, error::Error, iter::Peekable};
use Exp::*;

type LispErr = Box<dyn Error>;

// Expressions should be trivially copiable.
// In order to do that we need to implement Arc lists.
#[derive(Clone, Debug)]
pub enum Exp {
    Num(i32),
    Symbol(String),
    // TODO: List is Rc, shared inmutable
    List(Vec<Exp>),
    Lambda(Lambda), // Too big.
    Func(fn(&[Exp], &Arc<Env>) -> Result<Exp, LispErr>),
    Macro(fn(&[Exp], &Arc<Env>) -> Result<Vec<Exp>, LispErr>),
    Bool(bool),
}

fn tokenize(expr: String) -> Vec<String> {
    let replaced = expr
        .replace("\n", "")
        .replace("(", " ( ")
        .replace(")", " ) ");
    replaced.split_whitespace().map(|x| x.to_string()).collect()
}

use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref TOPLEVEL: RwLock<HashMap<String, Exp>> = RwLock::new(init_toplevel());
}

// TODO: Use Default for Evn instead of New
#[derive(Debug)]
pub struct Env {
    local: HashMap<String, Exp>,
    upper: Option<Arc<Env>>, // arc because many different
                             // processes can stem from one function call and share its state.
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
        // TODO: Our expressions should be trivially copyable.
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

fn init_toplevel() -> HashMap<String, Exp> {
    let mut env = HashMap::new();
    env.insert(
        "+".into(),
        Func(|args, _| {
            let x = args.iter().try_fold(0, |acc, x| match x {
                Num(n) => Ok::<i32, LispErr>(acc + n),
                _ => Err("invalid + argument".into()),
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
                    Num(n) => Ok::<i32, LispErr>(acc - n),
                    _ => Err("invalid - argument".into()),
                })?;
                Ok(Num(x))
            } else {
                Err("invalid arguments to -".into())
            }
        }),
    );

    env.insert("nil".into(), Bool(false));

    env.insert(
        "describe-env".into(),
        Func(|_, env| {
            println!("Inspection: {:?}", env);
            Ok(List(vec![]))
        }),
    );

    env.insert(
        "*".into(),
        Func(|args, _| {
            let x = args.iter().try_fold(1, |acc, x| match x {
                Num(n) => Ok::<i32, LispErr>(acc * n),
                _ => Err("invalid + argument".into()),
            })?;

            Ok(Num(x))
        }),
    );

    env.insert(
        "print".into(),
        Func(|args, _| {
            for arg in args.iter() {
                println!("{arg:?}");
            }
            Ok(List(vec![]))
        }),
    );

    env.insert("progn".into(), Macro(|args, _| Ok(args.to_vec())));

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

    // env.insert(
    //     "set".to_string(),
    //     Macro(
    //         |args, env| {
    //             if args.len() != 2 {
    //                 return Err("Wrong arguments to set".into());
    //             }
    //             if let Symbol(place) = &args[0] {
    //                 env.insert(place.clone(), args[1].clone());
    //             }
    //             return Ok(vec![]);
    //         }
    //     )
    // );

    // env.insert(
    //     "let".into(),
    //     Macro(|args, env| {

    //         env.insert(
    //             "lambda".into(),
    //             Macro(|args, _| {
    //                 if let List(lambda_list) = &args[0] {
    //                     let mut llist: Vec<String> = vec![];
    //                     for arg in lambda_list {
    //                         llist.push(match arg {
    //                             Symbol(str) => str.clone(),
    //                             _ => return Err("Invalid lambda list".into()),
    //                         });
    //                     }

    //                     let body = Vec::from(&args[1..]);
    //                     Ok(vec![Lambda(Lambda::new(llist, body))])
    //                 } else {
    //                     return Err("Invalid lambda list".into());
    //                 }
    //             }),
    //         );
    //     })
    // )

    env.insert(
        "defun".into(),
        Macro(|args, _| {
            if let List(lambda_list) = &args[1] {
                let mut llist: Vec<String> = vec![];
                for arg in lambda_list {
                    llist.push(match arg {
                        Symbol(str) => str.clone(),
                        _ => return Err("Invalid lambda list".into()),
                    });
                }
                let body = Vec::from(&args[2..]);
                set_global(&args[0], &Lambda(Lambda::new(llist, body)))?;
                Ok(vec![]) // TODO: Use Nil instead
            } else {
                return Err("Invalid lambda list".into());
            }
        }),
    );

    env.insert(
        "if".into(),
        Macro(|args, env| {
            let evaled = eval(&args[0], env)?;
            let is_false = match evaled {
                Bool(false) => true,
                _ => false,
            };

            if is_false {
                Ok(vec![args[2].clone()])
            } else {
                Ok(vec![args[1].clone()])
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
                let x: bool = match arg {
                    Bool(x) => *x,
                    _ => return Err("or argument is not bool".into()),
                };
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

            let first: i32 = match args[0] {
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
            if let List(lambda_list) = &args[0] {
                let mut llist: Vec<String> = vec![];
                for arg in lambda_list {
                    llist.push(match arg {
                        Symbol(str) => str.clone(),
                        _ => return Err("Invalid lambda list".into()),
                    });
                }

                let body = Vec::from(&args[1..]);
                Ok(vec![Lambda(Lambda::new(llist, body))])
            } else {
                return Err("Invalid lambda list".into());
            }
        }),
    );
    env
}

#[derive(Clone)]
pub struct Lambda {
    args: Vec<String>,
    body: Vec<Exp>,
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lambda with arguments: {:?}", self.args)
    }
}

impl Lambda {
    pub fn new(args: Vec<String>, body: Vec<Exp>) -> Lambda {
        Lambda { args, body }
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

fn set_global(sym: &Exp, val: &Exp) -> Result<(), LispErr> {
    // TODO: We should not panic.
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

fn atom<'a>(token: &'a str) -> Exp {
    if let Ok(num) = token.parse::<i32>() {
        Num(num)
    } else {
        Symbol(token.to_string())
    }
}

fn parse_tokens<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a String>>,
) -> Result<Exp, LispErr> {
    let token = match tokens.next() {
        Some(token) => token,
        None => return Err("Unexpected EOF while parsing".into()),
    };
    match token.as_str() {
        "'" => {
            // TODO: Create a make_list!() macro instead.
            Ok(List(vec![
                Symbol(String::from("quote")),
                parse_tokens(tokens)?,
            ]))
        }
        "(" => {
            let mut list = vec![];
            while *tokens.peek().unwrap() != ")" {
                list.push(parse_tokens(tokens)?);
            }
            // discard )
            let _ = tokens.next();
            Ok(List(list))
        }
        ")" => Err("Unexpected )".into()),
        token => Ok(atom(token)),
    }
}

fn eval_many(exps: &Vec<Exp>, env: &Arc<Env>) -> Result<Exp, LispErr> {
    if let Some((last, exps)) = exps.split_last() {
        for exp in exps {
            eval(exp, env)?;
        }
        eval(last, env)
    } else {
        return Ok(List(vec![]));
    }
}

fn eval(exp: &Exp, env: &Arc<Env>) -> Result<Exp, LispErr> {
    match exp {
        Num(num) => Ok(Num(*num)),
        Symbol(sym) => env.get(sym),
        List(list) => {
            if list.len() == 0 {
                return Ok(List(vec![]));
            }

            let Symbol(first) = &list[0] else {
                return Err("Expression cannot be evaluated as a function or macro".into());
            };

            let rest = &list[1..];
            match env.get(first)? {
                Func(fun) => {
                    let mut arg_list = vec![];
                    for exp in rest {
                        arg_list.push(eval(exp, env)?);
                    }
                    fun(&arg_list, env)
                }
                Lambda(ref lambda) => {
                    let mut arg_list = vec![];
                    for exp in rest {
                        arg_list.push(eval(exp, env)?);
                    }
                    lambda.call(arg_list, env)
                }
                Macro(macr) => {
                    let macroexpand = macr(rest, env)?;
                    eval_many(&macroexpand, env)
                }
                _ => Err("Attempted to call non-callable object".into()),
            }
        }
        Lambda(lam) => Ok(Lambda(lam.clone())),
        _ => Err("wtf".into()),
    }
}

pub fn main() {
    let program = "(progn

   (print (* (* 1 1 1 (+ 1 1) 1 1) (* (* 1 1) (* 1 1))))

   (def identity (lambda (t) t))

   (defun fibonacci (N)
       (if (or (= N 0) (= N 1))
           1
           (+ (fibonacci (identity (- N 1))) (identity (fibonacci (- N 2))))))

   (defun x (l) (print (identity l)) (x (+ l 1)))

   (fibonacci 5)

   )";
    // (describe-env)

   // (print (* (* 1 1 1 (+ 1 1) 1 1) (* (* 1 1) (* 1 1))))

   // (def identity (lambda (t) t))


    let tokens = tokenize(program.into());

    let mut iter = tokens.iter().peekable();
    let tree = parse_tokens(&mut iter).unwrap();

    let res = eval(&tree, &Arc::new(Env::new()));

    println!("Result: {res:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib() {
        let program = "(progn

   (defun fibonacci (N)
       (if (or (= N 0) (= N 1))
           1
           (+ (fibonacci (- N 1)) (fibonacci (- N 2)))))

   (fibonacci 5)

   )";

        let tokens = tokenize(program.into());
        let mut iter = tokens.iter().peekable();
        let tree = parse_tokens(&mut iter).unwrap();
        // This is for command line arguments.
        let mut env = Arc::new(Env::new());
        let res = eval(&tree, &mut env).unwrap();

        if let Num(res) = res {
            assert_eq!(res, 8);
        } else {
            panic!("Unexpected result.");
        }
    }

    #[test]
    fn test_deflambda() {
        let program = "(progn
   (def identity (lambda (t) t))
   )";

        let tokens = tokenize(program.into());
        let mut iter = tokens.iter().peekable();
        let tree = parse_tokens(&mut iter).unwrap();
        // This is for command line arguments.
        let mut env = Arc::new(Env::new());
        let res = eval(&tree, &mut env).unwrap();

        if let List(nil) = res {
            assert!(nil.is_empty());
        } else {
            panic!("Unexpected result.");
        }

        

    }
}
