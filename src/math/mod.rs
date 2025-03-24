use std::collections::HashMap;

use crate::code_tree::types::AssignEnum;
use crate::errors::simple::SimpleError;
use crate::errors::ErrorKind;
use crate::math::errors::DefinitionNotFound;
use crate::{code_tree::types::DataType, definitions::Defined, iter::Iter};

pub mod errors;

#[derive(Debug, Clone)]
pub struct VariableType {
    pub name: String,
    pub data_type: DataType,
    pub is_func: bool,
}

impl VariableType {
    pub fn new(name: String, data_type: DataType, is_func: bool) -> Self {
        Self {
            name,
            data_type,
            is_func,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprToken {
    Number(i32),
    Variable(VariableType),
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
            Some(ch) => SimpleError::error(
                &format!("Unexpected char: {}", ch),
                ErrorKind::MathProcessing,
            ),
            None => SimpleError::error("Unexpected EOI", ErrorKind::MathProcessing),
        }
    }

    pub fn parse_expr(&mut self) -> ExprToken {
        let mut node: ExprToken = self.parse_term();

        while let Some(char) = self.iter.peek() {
            match char {
                '+' | '-' => {
                    self.iter.next();
                    let r: ExprToken = self.parse_term();
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
        let mut node: ExprToken = self.parse_exponent();

        while let Some(char) = self.iter.peek() {
            match char {
                '*' | '/' => {
                    self.iter.next();
                    let r: ExprToken = self.parse_exponent();
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
        let mut node: ExprToken = self.parse_primary();

        while let Some(char) = self.iter.peek() {
            match char {
                '^' => {
                    self.iter.next();
                    let r: ExprToken = self.parse_primary();
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
        let mut buf: String = String::new();

        buf.extend(std::iter::from_fn(|| {
            self.iter
                .peek()
                .and_then(|ch: char| ch.is_numeric().then(|| self.iter.next().unwrap()))
        }));

        ExprToken::Number(buf.parse().unwrap_or_else(|_| {
            SimpleError::error(
                &format!("Invalid number {}", buf),
                ErrorKind::MathProcessing,
            )
        }))
    }

    fn process_var(&mut self) -> ExprToken {
        let mut buf: String = String::new();

        buf.extend(std::iter::from_fn(|| {
            self.iter
                .peek()
                .and_then(|ch: char| ch.is_alphanumeric().then(|| self.iter.next().unwrap()))
        }));

        ExprToken::Variable(VariableType::new(buf, DataType::Any, false))
    }

    fn process_literal(&mut self) -> ExprToken {
        self.iter.next();
        let mut buf: String = String::new();

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

        SimpleError::error(
            &format!("Unclosed `\"` for literal `{}...`", buf),
            ErrorKind::MathProcessing,
        );
    }
}

impl ExprToken {
    pub fn get_type(&mut self, scope: &HashMap<String, Defined>) -> DataType {
        match self {
            ExprToken::Number(_) => DataType::Int,
            ExprToken::Literal(_) => DataType::Str,
            ExprToken::Variable(var) => {
                var.data_type = ExprToken::get_var_type(var.name.to_string(), scope);
                var.is_func = scope
                    .get(&var.name)
                    .map_or(false, |def| matches!(def, Defined::Function(_)));
                var.data_type.clone()
            }
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
                    SimpleError::error(
                        &format!(
                            "Type mismatch for math operation: {:?} and {:?}",
                            lhs_type, rhs_type
                        ),
                        ErrorKind::MathProcessing,
                    );
                }
            }
        }
    }

    fn get_var_type(var: String, scope: &HashMap<String, Defined>) -> DataType {
        match scope.get(&var).unwrap_or_else(|| {
            SimpleError::error(
                &format!("Variable `{}` not defined", var),
                ErrorKind::MathProcessing,
            )
        }) {
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
            ExprToken::Variable(n) => def.push(n.name),
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
                if let Some(Defined::Variable(variable)) = scope.get(&n.name) {
                    if let (AssignEnum::Expr(e), true) = (variable.value.clone(), variable.is_const)
                    {
                        return e.clone();
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
