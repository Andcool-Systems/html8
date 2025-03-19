use std::collections::HashMap;

use crate::{code_tree::types::DataType, definitions::Defined, iter::Iter};

use super::errors::DefinitionNotFound;

#[derive(Debug, Clone)]
pub enum MathToken {
    Number(i32),
    Variable(String),
    Literal(String),
    Add(Box<MathToken>, Box<MathToken>),
    Sub(Box<MathToken>, Box<MathToken>),
    Mul(Box<MathToken>, Box<MathToken>),
    Div(Box<MathToken>, Box<MathToken>),
    Pow(Box<MathToken>, Box<MathToken>),
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

    fn parse_primary(&mut self) -> MathToken {
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

    pub fn parse_expr(&mut self) -> MathToken {
        let mut node = self.parse_term();

        while let Some(char) = self.iter.peek() {
            match char {
                '+' | '-' => {
                    self.iter.next();
                    let r = self.parse_term();
                    node = match char {
                        '+' => MathToken::Add(Box::new(node), Box::new(r)),
                        '-' => MathToken::Sub(Box::new(node), Box::new(r)),
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

    fn parse_term(&mut self) -> MathToken {
        let mut node = self.parse_exponent();

        while let Some(char) = self.iter.peek() {
            match char {
                '*' | '/' => {
                    self.iter.next();
                    let r = self.parse_exponent();
                    node = match char {
                        '/' => MathToken::Div(Box::new(node), Box::new(r)),
                        '*' => MathToken::Mul(Box::new(node), Box::new(r)),
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

    fn parse_exponent(&mut self) -> MathToken {
        let mut node = self.parse_primary();

        while let Some(char) = self.iter.peek() {
            match char {
                '^' => {
                    self.iter.next();
                    let r = self.parse_primary();
                    node = MathToken::Pow(Box::new(node), Box::new(r));
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

    fn process_number(&mut self) -> MathToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_numeric()) {
            buf.push(self.iter.next().unwrap());
        }

        MathToken::Number(buf.parse().expect("Invalid number"))
    }

    fn process_var(&mut self) -> MathToken {
        let mut buf = String::new();
        while matches!(self.iter.peek(), Some(ch) if ch.is_alphanumeric()) {
            buf.push(self.iter.next().unwrap());
        }

        MathToken::Variable(buf)
    }

    fn process_literal(&mut self) -> MathToken {
        self.iter.next();
        let mut buf = String::new();
        while let Some(ch) = self.iter.peek() {
            match ch {
                '"' => {
                    return MathToken::Literal(buf);
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

impl MathToken {
    pub fn get_type(&self, scope: &HashMap<String, Defined>) -> DataType {
        match self {
            MathToken::Number(_) => DataType::Int,
            MathToken::Literal(_) => DataType::Str,
            MathToken::Variable(var) => MathToken::get_var_type(var.to_string(), scope),
            MathToken::Add(lhs, rhs)
            | MathToken::Sub(lhs, rhs)
            | MathToken::Mul(lhs, rhs)
            | MathToken::Div(lhs, rhs)
            | MathToken::Pow(lhs, rhs) => {
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
        MathToken::recursive_math_def_check(self.clone(), &mut def);

        for d in def.iter() {
            if scope.get(d).is_none() {
                return Err(DefinitionNotFound::new(d));
            }
        }

        Ok(())
    }

    fn recursive_math_def_check(token: MathToken, def: &mut Vec<String>) {
        match token {
            MathToken::Variable(n) => def.push(n),
            MathToken::Add(a, b)
            | MathToken::Sub(a, b)
            | MathToken::Mul(a, b)
            | MathToken::Div(a, b)
            | MathToken::Pow(a, b) => {
                MathToken::recursive_math_def_check(*a, def);
                MathToken::recursive_math_def_check(*b, def);
            }
            _ => {}
        }
    }

    pub fn optimize(&mut self, scope: &HashMap<String, Defined>) {
        *self = self.clone().optimize_rec(scope);
    }

    fn optimize_rec(self, scope: &HashMap<String, Defined>) -> Self {
        match self {
            MathToken::Number(_) | MathToken::Literal(_) => self,
            MathToken::Variable(n) => {
                if let Some(Defined::Variable(variable)) = scope.get(&n) {
                    if variable.is_const {
                        return variable.value.clone();
                    }
                }
                MathToken::Variable(n)
            }
            MathToken::Add(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (MathToken::Number(left), MathToken::Number(right)) = (&a, &b) {
                    return MathToken::Number(left + right);
                }
                MathToken::Add(Box::new(a), Box::new(b))
            }
            MathToken::Sub(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (MathToken::Number(left), MathToken::Number(right)) = (&a, &b) {
                    return MathToken::Number(left - right);
                }
                MathToken::Sub(Box::new(a), Box::new(b))
            }
            MathToken::Mul(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (MathToken::Number(left), MathToken::Number(right)) = (&a, &b) {
                    return MathToken::Number(left * right);
                }
                MathToken::Mul(Box::new(a), Box::new(b))
            }
            MathToken::Div(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (MathToken::Number(left), MathToken::Number(right)) = (&a, &b) {
                    return MathToken::Number(left / right);
                }
                MathToken::Div(Box::new(a), Box::new(b))
            }
            MathToken::Pow(a, b) => {
                let a = a.optimize_rec(scope);
                let b = b.optimize_rec(scope);
                if let (MathToken::Number(left), MathToken::Number(right)) = (&a, &b) {
                    return MathToken::Number(left.pow(*right as u32));
                }
                MathToken::Pow(Box::new(a), Box::new(b))
            }
        }
    }
}
