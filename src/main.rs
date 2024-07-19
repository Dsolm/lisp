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
           (+ (fibonacci (- N 1)) (fibonacci (- N 2)))))
    (defun fibonacci-with-let (N)
        (if (or (= N 0) (= N 1))
            1
            (let ((f1 (fibonacci (- N 1)))
                  (f2 (fibonacci (- N 2))))
                 (+ f1 f2))))

   (print fibonacci)
   (print (let () 2))

   (print (let ((a 10)
                (b 100))
              (+ a b)))

   (defun foreach (list fn)
        (if list
           (progn
             (fn (car list))
             (foreach (cdr list) fn))
           (print 1234)))

   (foreach (list 1 2 3 4 5 6) (lambda (x) (print x)))

   (print (cdr (list 1 2 3 4)))
   (print (car (list 1 2 3 4)))

   (defun x (l) (print (identity l)) (x (+ l 1)))

   (assert (= (fibonacci 5) (fibonacci-with-let 5)))

   (fibonacci 5)
   )";

    let tokens = tokenize(program.into());

    let mut iter = tokens.iter().peekable();
    let tree = parse_tokens(&mut iter).unwrap();

    let res = eval(&tree, &Arc::new(Env::new())).unwrap();

    println!("Result: {res}");
}
