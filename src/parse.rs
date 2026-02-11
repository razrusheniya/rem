use crate::*;

pub const SPACE: &str = " ";

impl Define {
    pub fn parse(source: &str) -> Result<Vec<Define>, String> {
        let mut result = Vec::new();
        for line in tokenize(source, "\n")? {
            if let Some(func) = line.strip_prefix("fn ") {
                let (head, body) = ok!(func.split_once(")"))?;
                let (name, args) = ok!(head.split_once("("))?;
                let args = tokenize(args, ",")?
                    .iter()
                    .map(|x| Name::new(x.trim()))
                    .collect::<Result<IndexSet<Name>, String>>()?;
                let body = Expr::parse(body)?;
                result.push(Define(Name::new(name)?, args, body));
            }
        }
        Ok(result)
    }
}

impl Expr {
    pub fn parse(source: &str) -> Result<Expr, String> {
        let token = source.trim();
        if let Some(token) = token.strip_prefix("let ") {
            if let Ok((name, value)) = once!(token, "=") {
                Ok(Expr::Let(
                    Box::new(Expr::parse(&name)?),
                    Box::new(Expr::parse(&value)?),
                ))
            } else {
                Ok(Expr::Let(
                    Box::new(Expr::parse(token)?),
                    Box::new(Expr::Undefined),
                ))
            }
        } else if let Some(token) = token.strip_prefix("if ") {
            if let Ok((cond, body)) = once!(token, "then") {
                if let Ok((then, els)) = once!(&body, "else") {
                    Ok(Expr::If(
                        Box::new(Expr::parse(&cond)?),
                        Box::new(Expr::parse(&then)?),
                        Some(Box::new(Expr::parse(&els)?)),
                    ))
                } else {
                    Ok(Expr::If(
                        Box::new(Expr::parse(&cond)?),
                        Box::new(Expr::parse(&body)?),
                        None,
                    ))
                }
            } else {
                Err(format!("invalid `if` statement, `then` section not found"))
            }
        } else if let Some(token) = token.strip_prefix("while ") {
            if let Ok((cond, body)) = once!(token, "do") {
                Ok(Expr::While(
                    Box::new(Expr::parse(&cond)?),
                    Box::new(Expr::parse(&body)?),
                ))
            } else {
                Err(format!("invalid `while` statement, `do` section not found"))
            }
        } else if let Some(token) = token
            .strip_prefix("{")
            .and_then(|token| token.strip_suffix("}"))
        {
            let mut block = vec![];
            for line in tokenize(token, "\n")? {
                let (line, _) = once!(&line, ";").unwrap_or((line, String::new()));
                if line.trim().is_empty() {
                    continue;
                }
                block.push(Expr::parse(&line)?);
            }
            Ok(Expr::Block(block))
        } else if let Some(token) = token.strip_prefix("break ") {
            Ok(Expr::Break(Box::new(Expr::parse(&token)?)))
        } else if let Some(token) = token.strip_prefix("return ") {
            Ok(Expr::Return(Box::new(Expr::parse(&token)?)))
        } else if token == "break" {
            Ok(Expr::Break(Box::new(Expr::Undefined)))
        } else if token == "return" {
            Ok(Expr::Return(Box::new(Expr::Undefined)))
        } else if let Ok(operator) = parse_op(&token) {
            Ok(operator)
        } else if let Some(ptr) = token.strip_prefix("*") {
            Ok(Expr::Derefer(Box::new(Expr::parse(ptr)?)))
        } else if let Some(ptr) = token.strip_prefix("&") {
            if let Ok(name) = Name::new(ptr) {
                Ok(Expr::Pointer(name))
            } else if let Ok(Expr::Derefer(ptr)) = Expr::parse(ptr) {
                Ok(*ptr.clone())
            } else {
                Err(format!("invalid reference"))
            }
        } else if token.starts_with("\"") && token.ends_with("\"") {
            Ok(Expr::String(token.to_owned()))
        } else if let Some(expr) = token.strip_prefix("(").and_then(|x| x.strip_suffix(")")) {
            Expr::parse(expr)
        } else if let (true, Some(expr)) = (token.contains("("), token.strip_suffix(")")) {
            let (name, args) = ok!(expr.split_once("("))?;
            Ok(Expr::Call(
                Box::new(Expr::parse(&name)?),
                tokenize(&args, ",")?
                    .iter()
                    .map(|x| Expr::parse(x))
                    .collect::<Result<Vec<_>, String>>()?,
            ))
        } else if let (true, Some(expr)) = (token.contains("["), token.strip_suffix("]")) {
            let (arr, idx) = ok!(expr.rsplit_once("["))?;
            Ok(Expr::Derefer(Box::new(Expr::Add(
                Box::new(Expr::parse(arr)?),
                Box::new(Expr::Mul(
                    Box::new(Expr::parse(idx)?),
                    Box::new(Expr::Integer(8)),
                )),
            ))))
        } else if let Ok(literal) = token.parse::<i64>() {
            Ok(Expr::Integer(literal))
        } else if let Ok(literal) = token.parse::<bool>() {
            Ok(Expr::Integer(if literal { 1 } else { 0 }))
        } else {
            Ok(Expr::Variable(Name::new(token)?))
        }
    }
}

fn parse_op(source: &str) -> Result<Expr, String> {
    let tokens: Vec<String> = tokenize(source, SPACE)?;
    let n = ok!(tokens.len().checked_sub(2))?;
    let operator = ok!(tokens.get(n))?;
    let lhs = &ok!(tokens.get(..n))?.join(SPACE);
    let rhs = &ok!(tokens.get(n + 1..))?.join(SPACE);
    Ok(match operator.as_str() {
        "+" => Expr::Add(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "-" => Expr::Sub(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "*" => Expr::Mul(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "/" => Expr::Div(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "%" => Expr::Mod(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "==" => Expr::Eql(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "!=" => Expr::NotEq(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        ">" => Expr::Gt(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "<" => Expr::Lt(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        ">=" => Expr::GtEq(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "<=" => Expr::LtEq(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "&" => Expr::And(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "|" => Expr::Or(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        "^" => Expr::Xor(Box::new(Expr::parse(lhs)?), Box::new(Expr::parse(rhs)?)),
        op => return Err(format!("unknown operator: {op}")),
    })
}
