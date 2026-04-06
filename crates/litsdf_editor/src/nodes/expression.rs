//! Simple math expression parser and evaluator for the Expression node.
//!
//! Supports: +, -, *, /, %, parentheses, float literals, named variables (a-z),
//! and functions: sin, cos, abs, sqrt, min, max.
//!
//! Variables become input pins on the Expression node.

/// Parsed expression AST.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(f32),
    Var(usize), // index into variables list
    BinOp(Op, Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Func(Func, Box<Expr>),
    Func2(Func2, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum Op { Add, Sub, Mul, Div, Mod }

#[derive(Debug, Clone, Copy)]
pub enum Func { Sin, Cos, Abs, Sqrt }

#[derive(Debug, Clone, Copy)]
pub enum Func2 { Min, Max }

/// Parse result: AST + ordered list of variable names.
#[derive(Debug, Clone)]
pub struct ParsedExpression {
    pub variables: Vec<String>,
    pub ast: Expr,
}

impl Default for ParsedExpression {
    fn default() -> Self {
        Self { variables: Vec::new(), ast: Expr::Literal(0.0) }
    }
}

/// Evaluate an expression with the given variable values.
pub fn evaluate(expr: &Expr, vars: &[f32]) -> f32 {
    match expr {
        Expr::Literal(v) => *v,
        Expr::Var(i) => vars.get(*i).copied().unwrap_or(0.0),
        Expr::BinOp(op, a, b) => {
            let va = evaluate(a, vars);
            let vb = evaluate(b, vars);
            match op {
                Op::Add => va + vb,
                Op::Sub => va - vb,
                Op::Mul => va * vb,
                Op::Div => if vb.abs() < 1e-10 { 0.0 } else { va / vb },
                Op::Mod => if vb.abs() < 1e-10 { 0.0 } else { va % vb },
            }
        }
        Expr::Neg(a) => -evaluate(a, vars),
        Expr::Func(f, a) => {
            let v = evaluate(a, vars);
            match f {
                Func::Sin => v.sin(),
                Func::Cos => v.cos(),
                Func::Abs => v.abs(),
                Func::Sqrt => v.max(0.0).sqrt(),
            }
        }
        Expr::Func2(f, a, b) => {
            let va = evaluate(a, vars);
            let vb = evaluate(b, vars);
            match f {
                Func2::Min => va.min(vb),
                Func2::Max => va.max(vb),
            }
        }
    }
}

/// Parse an expression string. Returns Err with description on failure.
pub fn parse(input: &str) -> Result<ParsedExpression, String> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        pos: 0,
        variables: Vec::new(),
    };
    let ast = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!("Unexpected token at position {}", parser.pos));
    }
    Ok(ParsedExpression { variables: parser.variables, ast })
}

// ── Tokenizer ──

#[derive(Debug, Clone)]
enum Token {
    Num(f32),
    Ident(String),
    Op(char),
    LParen,
    RParen,
    Comma,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' => { i += 1; }
            '+' | '-' | '*' | '/' | '%' => {
                tokens.push(Token::Op(chars[i]));
                i += 1;
            }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
                let s: String = chars[start..i].iter().collect();
                let v = s.parse::<f32>().map_err(|_| format!("Invalid number: {s}"))?;
                tokens.push(Token::Num(v));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(s));
            }
            c => return Err(format!("Unexpected character: {c}")),
        }
    }
    Ok(tokens)
}

// ── Recursive Descent Parser ──

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    variables: Vec<String>,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.advance() {
            Some(Token::RParen) => Ok(()),
            _ => Err("Expected ')'".into()),
        }
    }

    fn var_index(&mut self, name: &str) -> usize {
        if let Some(i) = self.variables.iter().position(|v| v == name) {
            i
        } else {
            self.variables.push(name.to_string());
            self.variables.len() - 1
        }
    }

    // expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Op('+')) => { self.advance(); let right = self.parse_term()?; left = Expr::BinOp(Op::Add, Box::new(left), Box::new(right)); }
                Some(Token::Op('-')) => { self.advance(); let right = self.parse_term()?; left = Expr::BinOp(Op::Sub, Box::new(left), Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    // term = unary (('*' | '/' | '%') unary)*
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Op('*')) => { self.advance(); let right = self.parse_unary()?; left = Expr::BinOp(Op::Mul, Box::new(left), Box::new(right)); }
                Some(Token::Op('/')) => { self.advance(); let right = self.parse_unary()?; left = Expr::BinOp(Op::Div, Box::new(left), Box::new(right)); }
                Some(Token::Op('%')) => { self.advance(); let right = self.parse_unary()?; left = Expr::BinOp(Op::Mod, Box::new(left), Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    // unary = '-' unary | atom
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Some(Token::Op('-')) = self.peek() {
            self.advance();
            let inner = self.parse_unary()?;
            Ok(Expr::Neg(Box::new(inner)))
        } else {
            self.parse_atom()
        }
    }

    // atom = number | '(' expr ')' | func '(' expr ')' | func2 '(' expr ',' expr ')' | variable
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(Token::Num(v)) => Ok(Expr::Literal(v)),
            Some(Token::LParen) => {
                let inner = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(inner)
            }
            Some(Token::Ident(name)) => {
                // Check for function call
                if let Some(Token::LParen) = self.peek() {
                    self.advance(); // consume '('
                    match name.as_str() {
                        "sin" | "cos" | "abs" | "sqrt" => {
                            let arg = self.parse_expr()?;
                            self.expect_rparen()?;
                            let f = match name.as_str() {
                                "sin" => Func::Sin,
                                "cos" => Func::Cos,
                                "abs" => Func::Abs,
                                "sqrt" => Func::Sqrt,
                                _ => unreachable!(),
                            };
                            Ok(Expr::Func(f, Box::new(arg)))
                        }
                        "min" | "max" => {
                            let a = self.parse_expr()?;
                            match self.advance() {
                                Some(Token::Comma) => {}
                                _ => return Err("Expected ',' in function call".into()),
                            }
                            let b = self.parse_expr()?;
                            self.expect_rparen()?;
                            let f = match name.as_str() {
                                "min" => Func2::Min,
                                "max" => Func2::Max,
                                _ => unreachable!(),
                            };
                            Ok(Expr::Func2(f, Box::new(a), Box::new(b)))
                        }
                        _ => Err(format!("Unknown function: {name}")),
                    }
                } else {
                    // Variable
                    let idx = self.var_index(&name);
                    Ok(Expr::Var(idx))
                }
            }
            Some(t) => Err(format!("Unexpected token: {t:?}")),
            None => Err("Unexpected end of expression".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        let p = parse("42.5").unwrap();
        assert!(p.variables.is_empty());
        assert!((evaluate(&p.ast, &[]) - 42.5).abs() < 0.001);
    }

    #[test]
    fn arithmetic() {
        let p = parse("3 + 4 * 2").unwrap();
        assert!((evaluate(&p.ast, &[]) - 11.0).abs() < 0.001);
    }

    #[test]
    fn variables() {
        let p = parse("a * 0.5 + b").unwrap();
        assert_eq!(p.variables, vec!["a", "b"]);
        assert!((evaluate(&p.ast, &[10.0, 3.0]) - 8.0).abs() < 0.001);
    }

    #[test]
    fn functions() {
        let p = parse("sin(0) + abs(-5)").unwrap();
        assert!((evaluate(&p.ast, &[]) - 5.0).abs() < 0.001);
    }

    #[test]
    fn nested() {
        let p = parse("max(a, min(b, 10))").unwrap();
        assert_eq!(p.variables, vec!["a", "b"]);
        assert!((evaluate(&p.ast, &[5.0, 20.0]) - 10.0).abs() < 0.001);
    }

    #[test]
    fn negation() {
        let p = parse("-a + 1").unwrap();
        assert!((evaluate(&p.ast, &[3.0]) - (-2.0)).abs() < 0.001);
    }
}
