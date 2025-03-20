use std::collections::HashMap;

use crate::{code_tree::types::DataType, definitions::Defined, iter::Iter};

use super::errors::DefinitionNotFound;

#[derive(Debug, Clone)]
pub enum ExprToken {
    Number(i32),
    Variable(String),
    Literal(String),
    Add(Box<ExprToken>, Box<ExprToken>),
    Sub(Box<ExprToken>, Box<ExprToken>),
    Mul(Box<ExprToken>, Box<ExprToken>),
    Div(Box<ExprToken>, Box<ExprToken>),
    Pow(Box<ExprToken>, Box<ExprToken>),
}

pub struct MathParser {
    iter: Iter<char>,
}

impl MathParser {
    pub fn new<I: IntoIterator<Item = char>>(iter: I) -> Self {
        Self {
            iter: Iter::from(iter),
        }
    }

    fn parse_primary(&mut self) -> ExprToken {
        match self.iter.peek() {
            Some('.') => todo!("Float not yet implemented"),
            Some('(') | Some(')') => todo!("Parentheses not implemented yet"),
            Some('"') => self.process_literal(),
            Some(ch) if ch.is_whitespace() => {
                self.iter.next();
                self.parse_primary()
            }
            Some(ch) if ch.is_numeric() => self.process_number(),
            Some(ch) if ch.is_alphabetic() => self.process_var(),
            Some(ch) => panic!("Unexpected char: {}", ch),
            None => panic!("Unexpected EOI"),
        }
    }

    pub fn parse_expr(&mut self) -> ExprToken {
        let mut node = self.parse_term();

        while let Some(char) = self.iter.peek() {
            match char {
                '+' | '-' => {
                    self.iter.next();
                    let r = self.parse_term();
                    node = match char {
                        '+' => ExprToken::Add(Box::new(node), Box::new(r)),
                        '-' => ExprToken::Sub(Box::new(node), Box::new(r)),
                        _ => unreachable!(),
                    };
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn parse_term(&mut self) -> ExprToken {
        let mut node = self.parse_exponent();

        while let Some(char) = self.iter.peek() {
            match char {
                '*' | '/' => {
                    self.iter.next();
                    let r = self.parse_exponent();
                    node = match char {
                        '/' => ExprToken::Div(Box::new(node), Box::new(r)),
                        '*' => ExprToken::Mul(Box::new(node), Box::new(r)),
                        _ => unreachable!(),
                    };
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn parse_exponent(&mut self) -> ExprToken {
        let mut node = self.parse_primary();

        while let Some(char) = self.iter.peek() {
            match char {
                '^' => {
                    self.iter.next();
                    let r = self.parse_primary();
                    node = ExprToken::Pow(Box::new(node), Box::new(r));
                }
                ch if ch.is_whitespace() => {
                    self.iter.next();
                    continue;
                }
                _ => break,
            }
        }

        node
    }

    fn process_number(&mut self) -> ExprToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_numeric()) {
            buf.push(self.iter.next().unwrap());
        }

        ExprToken::Number(buf.parse().expect("Invalid number"))
    }

    fn process_var(&mut self) -> ExprToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_alphanumeric()) {
            buf.push(self.iter.next().unwrap());
        }

        ExprToken::Variable(buf)
    }

    fn process_literal(&mut self) -> ExprToken {
        self.iter.next();
        let mut buf = String::new();
        while let Some(ch) = self.iter.peek() {
            match ch {
                '"' => {
                    return ExprToken::Literal(buf);
                }
                ch => {
                    self.iter.next();
                    buf.push(ch)
                }
            }
        }

        panic!("Unclosed `\"` for literal `{}...`", buf);
    }
}

impl ExprToken {
    pub fn get_type(&self, scope: &HashMap<String, Defined>) -> DataType {
        match self {
            ExprToken::Number(_) => DataType::Int,
            ExprToken::Literal(_) => DataType::Str,
            ExprToken::Variable(var) => ExprToken::get_var_type(var.to_string(), scope),
            ExprToken::Add(lhs, rhs)
            | ExprToken::Sub(lhs, rhs)
            | ExprToken::Mul(lhs, rhs)
            | ExprToken::Div(lhs, rhs)
            | ExprToken::Pow(lhs, rhs) => {
                let lhs_type = lhs.get_type(scope);
                let rhs_type = rhs.get_type(scope);
                if lhs_type == rhs_type {
                    lhs_type
                } else {
                    panic!(
                        "Type mismatch for math operation: {:?} and {:?}",
                        lhs_type, rhs_type
                    );
                }
            }
        }
    }

    fn get_var_type(var: String, scope: &HashMap<String, Defined>) -> DataType {
        match scope.get(&var).unwrap() {
            Defined::Variable(vds) => vds.data_type.clone(),
            Defined::Function(fds) => fds.data_type.clone(),
        }
    }

    /// Check definitions in math AST
    pub fn check_def(&self, scope: &HashMap<String, Defined>) -> Result<(), DefinitionNotFound> {
        let mut def = Vec::new();
        ExprToken::recursive_math_def_check(self.clone(), &mut def);

        for d in def.iter() {
            if scope.get(d).is_none() {
                return Err(DefinitionNotFound::new(d));
            }
        }

        Ok(())
    }

    fn recursive_math_def_check(token: ExprToken, def: &mut Vec<String>) {
        match token {
            ExprToken::Variable(n) => def.push(n),
            ExprToken::Add(a, b)
            | ExprToken::Sub(a, b)
            | ExprToken::Mul(a, b)
            | ExprToken::Div(a, b)
            | ExprToken::Pow(a, b) => {
                ExprToken::recursive_math_def_check(*a, def);
                ExprToken::recursive_math_def_check(*b, def);
            }
            _ => {}
        }
    }

    pub fn optimize(&mut self, scope: &HashMap<String, Defined>) {
        *self = self.clone().optimize_rec(scope);
    }

    fn optimize_rec(self, scope: &HashMap<String, Defined>) -> Self {
        match self {
            ExprToken::Number(_) | ExprToken::Literal(_) => self,
            ExprToken::Variable(n) => {
                if let Some(Defined::Variable(variable)) = scope.get(&n) {
                    if variable.is_const {
                        return variable.value.clone();
                    }
                }
                ExprToken::Variable(n)
            }
            ExprToken::Add(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (ExprToken::Number(left), ExprToken::Number(right)) = (&a, &b) {
                    return ExprToken::Number(left + right);
                }
                ExprToken::Add(Box::new(a), Box::new(b))
            }
            ExprToken::Sub(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (ExprToken::Number(left), ExprToken::Number(right)) = (&a, &b) {
                    return ExprToken::Number(left - right);
                }
                ExprToken::Sub(Box::new(a), Box::new(b))
            }
            ExprToken::Mul(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (ExprToken::Number(left), ExprToken::Number(right)) = (&a, &b) {
                    return ExprToken::Number(left * right);
                }
                ExprToken::Mul(Box::new(a), Box::new(b))
            }
            ExprToken::Div(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (ExprToken::Number(left), ExprToken::Number(right)) = (&a, &b) {
                    return ExprToken::Number(left / right);
                }
                ExprToken::Div(Box::new(a), Box::new(b))
            }
            ExprToken::Pow(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (ExprToken::Number(left), ExprToken::Number(right)) = (&a, &b) {
                    return ExprToken::Number(left.pow(*right as u32));
                }
                ExprToken::Pow(Box::new(a), Box::new(b))
            }
        }
    }
}
