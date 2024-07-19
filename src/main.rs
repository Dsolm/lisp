use std::sync::Arc;

pub mod tokenizer;
pub mod tests;
pub mod eval;
pub mod exp; 
pub mod env;
pub mod parser;

use eval::eval;
use env::Env;
use tokenizer::tokenize;
use parser::parse_tokens;


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

