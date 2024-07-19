use std::sync::Arc;

pub mod env;
pub mod eval;
pub mod exp;
pub mod parser;
pub mod tests;
pub mod tokenizer;

use env::Env;
use eval::eval;
use parser::parse_tokens;
use tokenizer::tokenize;

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
