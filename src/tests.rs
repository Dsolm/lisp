#[cfg(test)]
mod tests {
    use crate::exp::*;
    use crate::eval::eval;
    use crate::env::Env;
    use crate::tokenizer::tokenize;
    use crate::parser::parse_tokens;

    use crate::*;

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

    #[test]
    fn test_let_no_bindings() {
        let program = "(progn
         (let ()
            2)
         )";

        let tokens = tokenize(program.into());
        let mut iter = tokens.iter().peekable();
        let tree = parse_tokens(&mut iter).unwrap();
        // This is for command line arguments.
        let mut env = Arc::new(Env::new());
        let res = eval(&tree, &mut env).unwrap();

        if let Num(two) = res {
            assert_eq!(two, 2);
        } else {
            panic!("Unexpected result.");
        }
    }

    #[test]
    fn test_let_bindings() {
        let program = "(let ((a 10) (b 100))
                                (+ a b))";

        let tokens = tokenize(program.into());
        let mut iter = tokens.iter().peekable();
        let tree = parse_tokens(&mut iter).unwrap();
        // This is for command line arguments.
        let mut env = Arc::new(Env::new());
        let res = eval(&tree, &mut env).unwrap();

        if let Num(num) = res {
            assert_eq!(num, 110);
        } else {
            panic!("Unexpected result.");
        }
    }
}
