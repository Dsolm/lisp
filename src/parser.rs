use std::iter::Peekable;

use crate::exp::*;

pub fn parse_tokens<'a>(
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
            Ok(Vector(list))
        }
        ")" => Err("Unexpected )".into()),
        token => Ok(atom(token)),
    }
}

fn atom<'a>(token: &'a str) -> Exp {
    if let Ok(num) = token.parse::<i64>() {
        Num(num)
    } else {
        Symbol(token.to_string())
    }
}
