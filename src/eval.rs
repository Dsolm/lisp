use crate::exp::*;
use crate::env::Env;
use crate::exp::lambda::Lambda;

use std::sync::Arc;

pub fn eval_many(exps: &[Exp], env: &Arc<Env>) -> Result<Exp, LispErr> {
    if let Some((last, exps)) = exps.split_last() {
        for exp in exps {
            eval(exp, env)?;
        }
        eval(last, env)
    } else {
        return Ok(List(vec![]));
    }
}

fn eval_fun_call(func: NativeFunction, args: &[Exp], env: &Arc<Env>) -> Result<Exp, LispErr> {
    let mut arg_list = vec![];
    for exp in args {
        arg_list.push(eval(exp, env)?);
    }
    func(&arg_list)
}

fn eval_lambda_call(lambda: &Lambda, args: &[Exp], env: &Arc<Env>) -> Result<Exp, LispErr> {
    let mut arg_list = vec![];
    for arg in args {
        arg_list.push(eval(arg, env)?);
    }
    lambda.call(arg_list, env)
}

fn eval_macro(macr: Macro, args: &[Exp], env: &Arc<Env>) -> Result<Exp, LispErr> {
    let macroexpand = macr(args, env)?;
    eval_many(&macroexpand, env)
}

pub fn eval(exp: &Exp, env: &Arc<Env>) -> Result<Exp, LispErr> {
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
                Func(fun) => eval_fun_call(fun, rest, env),
                Lambda(ref lambda) => eval_lambda_call(lambda, rest, env),
                Macro(macr) => eval_macro(macr, rest, env),
                _ => Err("Attempted to call non-callable object".into()),
            }
        }
        Lambda(lam) => Ok(Lambda(lam.clone())),
        _ => Err("wtf".into()),
    }
}
