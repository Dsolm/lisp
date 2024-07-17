fn tokenize(expr: String) -> Vec<String> {
    let replaced = expr
        .replace("\n", "")
        .replace("(", " ( ")
        .replace(")", " ) ");
    replaced.split_whitespace().map(|x| x.to_string()).collect()
}

use core::fmt;
use std::iter::zip;

type LispErr = Box<dyn Error>;

#[derive(Clone)]
struct Lambda {
    lambda_list: Vec<String>,
    exps: Vec<Exp>,
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lambda with arguments: {:?}", self.lambda_list)
    }
}

impl Lambda {
    pub fn new(lambda_list: Vec<String>, exps: Vec<Exp>) -> Lambda {
        Lambda { lambda_list, exps }
    }

    pub fn call(self: &mut Self, args: &Vec<Exp>, env: &mut Env) -> Result<Exp, LispErr> {
        if self.lambda_list.len() != args.len() {
            return Err("Wrong number of function arguments".into());
        }
        env.extend(zip(self.lambda_list.clone(), args.clone()));
        eval_many(&self.exps, env)
    }
}

#[derive(Clone, Debug)]
enum Exp {
    // T,
    Num(i32),
    Symbol(String),
    List(Vec<Exp>),
    Lambda(Lambda),
    // Quote(Box<Exp>),
    Func(fn(&[Exp], &mut Env) -> Result<Exp, LispErr>),
    Macro(fn(&[Exp], &mut Env) -> Result<Vec<Exp>, LispErr>),
    Bool(bool),
}

type Env = HashMap<String, Exp>;

fn set_symbol(env: &mut Env, sym: &Exp, val: &Exp) -> Result<(), LispErr> {
    if let Symbol(place) = sym {
        let evaled = eval(val, env)?;
        env.insert(place.to_owned(), evaled);
        Ok(())
    } else {
        Err("Cannot set non-symbol".into())
    }
}

fn new_env() -> Env {
    let mut env = HashMap::new();
    env.insert(
        "+".to_string(),
        Func(|args: &[Exp], _: &mut Env| -> Result<Exp, LispErr> {
            let x = args.iter().try_fold(0, |acc, x| match x {
                Num(n) => Ok::<i32, LispErr>(acc + n),
                _ => Err("invalid + argument".into()),
            })?;

            Ok(Num(x))
        }),
    );

    env.insert(
        "-".to_string(),
        Func(|args: &[Exp], _: &mut Env| -> Result<Exp, LispErr> {
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

    env.insert("nil".to_string(), Bool(false));

    env.insert(
        "describe-env".to_string(),
        Func(|_, env| {
            println!("Inspection: {:?}", env);
            Ok(List(vec![]))
        }),
    );

    env.insert(
        "*".to_string(),
        Func(|args: &[Exp], _: &mut Env| -> Result<Exp, LispErr> {
            let x = args.iter().try_fold(1, |acc, x| match x {
                Num(n) => Ok::<i32, LispErr>(acc * n),
                _ => Err("invalid + argument".into()),
            })?;

            Ok(Num(x))
        }),
    );

    env.insert(
        "print".to_string(),
        Func(|args, _| {
            for arg in args.iter() {
                println!("{arg:?}");
            }
            Ok(List(vec![]))
        }),
    );

    env.insert("progn".to_string(), Macro(|args, _| Ok(args.to_vec())));

    env.insert(
        "def".to_string(),
        Macro(|args, env| {
            if args.len() != 2 {
                return Err("Wrong number of arguments to def".into());
            }
            set_symbol(env, &args[0], &args[1])?;
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

    env.insert(
        "defun".to_string(),
        Macro(|args, env| {
            if let List(lambda_list) = &args[1] {
                let mut llist: Vec<String> = vec![];
                for arg in lambda_list {
                    llist.push(match arg {
                        Symbol(str) => str.clone(),
                        _ => return Err("Invalid lambda list".into()),
                    });
                }

                let body = Vec::from(&args[2..]);
                _ = set_symbol(env, &args[0], &Lambda(Lambda::new(llist, body)));

                Ok(vec![])
            } else {
                return Err("Invalid lambda list".into());
            }
        }),
    );

    env.insert(
        "if".to_string(),
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
        "or".to_string(),
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
        "=".to_string(),
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
        "lambda".to_string(),
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

    // env.insert(
    //     "eval".to_string(),
    //     Func(
    //         |args, env| {
    //             eval(&args[0], env)?;
    //         }
    //     )
    // );

    env
}

use std::{collections::HashMap, error::Error, iter::Peekable};

use Exp::*;

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

fn eval_many(exps: &[Exp], env: &mut Env) -> Result<Exp, LispErr> {
    if let Some((last, exps)) = exps.split_last() {
        for exp in exps {
            eval(exp, env)?;
        }
        eval(last, env)
    } else {
        return Ok(List(vec![]));
    }
}

fn eval(exp: &Exp, env: &mut Env) -> Result<Exp, LispErr> {
    match exp {
        Num(num) => Ok(Num(*num)),
        Symbol(sym) => match env.get(sym) {
            Some(v) => Ok(v.clone()),
            None => {
                println!("Unbound symbol: {sym}");
                Err("Symbol is unbound".into())
            }
        },
        List(list) => {
            if list.len() == 0 {
                return Ok(List(vec![]));
            }
            let rest = &list[1..];
            match eval(&list[0], env)? {
                Func(fun) => {
                    let mut arg_list = vec![];
                    for exp in rest {
                        arg_list.push(eval(exp, env)?);
                    }
                    fun(&arg_list, env)
                }
                Lambda(mut lambda) => {
                    let mut arg_list = vec![];
                    for exp in rest {
                        arg_list.push(eval(exp, env)?);
                    }
                    // Esto está muy mal. Además de no ser eficiente hace que las lambdas no puedan modificar el entorno, pero
                    // Pero las lambdas siguen estando implementadas como si sí que pudiesen.
                    // Hacemos esto porque no podemos tener closures que capturan entorno mutable en rust de forma segura sin implementar
                    // garbage collection. Si hiciesemos que el entorno que reciben las funciones fuese inmutable tendríamos que cambiar
                    // todo el código del evaluador.
                    // Se podría mejorar un poco usando Cow
                    let prev_env = env.clone();
                    let res = lambda.call(&arg_list, env);
                    *env = prev_env;
                    res
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
           (+ (fibonacci (- N 1)) (fibonacci (- N 2)))))

   (defun x (l) (print (identity l)) (x (+ l 1)))
   (fibonacci 5)
   )";
    // (describe-env)

    let tokens = tokenize(program.into());

    let mut iter = tokens.iter().peekable();
    let tree = parse_tokens(&mut iter).unwrap();

    let mut env = new_env();
    let res = eval(&tree, &mut env);

    println!("Result: {res:?}");
}
