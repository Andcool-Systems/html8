use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::code_tree::types::{
    ArgStruct, AssignEnum, AssignStruct, BlockType, ForStruct, ServiceBlockType,
};
use crate::{
    code_tree::types::{
        CallStruct, DataType, DefinitionType, FunctionDefinitionStruct, NodeType,
        VariableDefinitionStruct,
    },
    libs::std::Std,
    math::ExprToken,
};

const C_KEYWORDS: &[&str] = &[
    "return", "int", "char", "void", "if", "else", "while", "for", "do", "break", "continue",
    "println", "print",
];

pub struct CLang {
    pub tree: NodeType,
}

impl CompilerCodegen for CLang {
    fn new(tree: NodeType) -> Self {
        Self { tree }
    }
    fn compile(&mut self) -> String {
        let statements: String = self._compile(self.tree.clone());
        format!(
            "#include <stdio.h>\n#include <math.h>\nint main(void){{\n{}return 0;\n}}",
            statements
        )
    }
}

impl CLang {
    fn convert_types(data_type: DataType) -> String {
        match data_type {
            DataType::Int => String::from("int"),
            DataType::Bool => String::from("int"),
            DataType::Str => String::from("char"),
            DataType::Void => String::from("void"),
            _ => String::from("int"),
        }
    }

    fn _compile(&mut self, node: NodeType) -> String {
        match node {
            NodeType::BLOCK(block_struct) => {
                let mut statements = String::new();

                let brace_needed = match block_struct.tag {
                    BlockType::Div | BlockType::For => {
                        statements.push_str("{\n");
                        true
                    }
                    _ => false,
                };

                block_struct
                    .children
                    .into_iter()
                    .for_each(|child: Box<NodeType>| {
                        if matches!(
                            *child, NodeType::DEFINITION(DefinitionType::Function(ref fds))
                            if !fds.must_be_compiled
                        ) {
                            return;
                        }
                        let stmt: String = self._compile(*child);
                        (!stmt.is_empty()).then(|| {
                            statements.push_str(&stmt);
                            statements.push('\n');
                        });
                    });

                if brace_needed {
                    statements.push('}');
                }
                statements
            }
            NodeType::DEFINITION(definition_type) => match definition_type {
                DefinitionType::Function(fds) => self.compile_fn(fds),
                DefinitionType::Variable(vds) => self.compile_var(vds),
            },
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("println") => {
                Std::compile_println(call_struct)
            }
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("print") => {
                Std::compile_print(call_struct)
            }
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("return") => {
                Std::compile_return(call_struct)
            }
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("inc") => {
                Std::compile_inc(call_struct)
            }
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("dec") => {
                Std::compile_dec(call_struct)
            }
            NodeType::CALL(call_struct) => self.compile_call(call_struct),
            NodeType::ASSIGN(assign_struct) => self.compile_assign(assign_struct),
            NodeType::ServiceBlock(sbt) => match sbt {
                ServiceBlockType::For(for_struct) => self.compile_for(for_struct),
            },
        }
    }

    fn compile_for(&mut self, for_struct: ForStruct) -> String {
        let mut children: Vec<String> = Vec::new();
        for_struct
            .children
            .into_iter()
            .for_each(|child: Box<NodeType>| {
                let stmt_string = self._compile(*child);
                (!stmt_string.is_empty()).then(|| children.push(stmt_string));
            });
        let random_name = Self::random_string(7);

        format!(
            "{}\nloop_{}:\nif({}>={}){{goto end_{};}}\n{}\n{}++;\ngoto loop_{};\nend_{}:",
            children[0],
            random_name,
            for_struct.iter_name,
            Self::process_expr_token(for_struct.end),
            random_name,
            children[1..].join("\n"),
            for_struct.iter_name,
            random_name,
            random_name
        )
    }

    fn compile_var(&mut self, v: VariableDefinitionStruct) -> String {
        let arr: String = (v.data_type == DataType::Str)
            .then(|| String::from("*"))
            .unwrap_or_else(String::new);

        let value = match v.value {
            AssignEnum::Expr(expr_token) => Self::process_expr_token(expr_token),
            AssignEnum::Call(node_type) => match *node_type {
                NodeType::CALL(call_struct) => self.compile_call(call_struct),
                _ => unreachable!(),
            },
            AssignEnum::None => unreachable!(),
        };

        let is_const = if v.is_const { "const " } else { "" };

        format!(
            "{}{} {}{} = {};",
            is_const,
            Self::convert_types(v.data_type),
            arr,
            v.name,
            value
        )
    }

    fn compile_fn(&mut self, f: FunctionDefinitionStruct) -> String {
        #[allow(clippy::obfuscated_if_else)]
        let fn_name: String = is_c_keyword(&f.name)
            .then(|| format!("{}_func", f.name))
            .unwrap_or(f.name.clone());

        let mut args: Vec<String> = f
            .args
            .iter()
            .map(|arg: &ArgStruct| {
                let pointer: String = (arg.data_type == DataType::Str)
                    .then(|| String::from("*"))
                    .unwrap_or_else(String::new);

                format!(
                    "{}{} {}",
                    Self::convert_types(arg.data_type.clone()),
                    pointer,
                    arg.name
                )
            })
            .collect::<Vec<String>>();
        args.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let mut children: Vec<String> = Vec::new();
        f.children.into_iter().for_each(|child: Box<NodeType>| {
            let stmt_string = self._compile(*child);
            (!stmt_string.is_empty()).then(|| children.push(stmt_string));
        });

        format!(
            "{} {}({}) {{\n{}\n}}",
            Self::convert_types(f.data_type),
            fn_name,
            args.join(", "),
            children.join("\n")
        )
    }

    fn compile_call(&mut self, call: CallStruct) -> String {
        let calling_name = is_c_keyword(&call.calling_name)
            .then(|| format!("{}_func", call.calling_name))
            .unwrap_or_else(|| call.calling_name.clone());

        let mut args: Vec<String> = call
            .args
            .iter()
            .map(|a| Self::process_expr_token(a.value.clone().unwrap()))
            .collect::<Vec<String>>();
        args.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        format!("{}({});", calling_name, args.join(", "))
    }

    fn compile_assign(&mut self, assign_struct: AssignStruct) -> String {
        match assign_struct.body {
            AssignEnum::Expr(expr_token) => {
                format!(
                    "{} = {};",
                    assign_struct.name,
                    Self::process_expr_token(expr_token)
                )
            }
            AssignEnum::Call(node_type) => match *node_type {
                NodeType::CALL(call_struct) => {
                    format!(
                        "{} = {};",
                        assign_struct.name,
                        self.compile_call(call_struct)
                    )
                }
                _ => unreachable!(),
            },
            AssignEnum::None => unreachable!(),
        }
    }

    pub fn process_expr_token(token: ExprToken) -> String {
        match token {
            ExprToken::Number(n) => format!("{}", n),
            ExprToken::Variable(v) => v.name,
            ExprToken::Literal(l) => format!("\"{}\"", l),
            ExprToken::Add(l, r) => {
                let l_token = Self::process_expr_token(*l);
                let r_token = Self::process_expr_token(*r);
                format!("{} + {}", l_token, r_token)
            }
            ExprToken::Sub(l, r) => {
                let l_token = Self::process_expr_token(*l);
                let r_token = Self::process_expr_token(*r);
                format!("{} - {}", l_token, r_token)
            }
            ExprToken::Mul(l, r) => {
                let l_token = Self::process_expr_token(*l);
                let r_token = Self::process_expr_token(*r);
                format!("{} * {}", l_token, r_token)
            }
            ExprToken::Div(l, r) => {
                let l_token = Self::process_expr_token(*l);
                let r_token = Self::process_expr_token(*r);
                format!("{} / {}", l_token, r_token)
            }
            ExprToken::Pow(l, r) => {
                let l_token = Self::process_expr_token(*l);
                let r_token = Self::process_expr_token(*r);
                format!("pow({}, {})", l_token, r_token)
            }
        }
    }

    fn random_string(length: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

#[inline(always)]
fn is_c_keyword(name: &str) -> bool {
    C_KEYWORDS.contains(&name)
}

pub trait CompilerCodegen {
    fn new(tree: NodeType) -> Self
    where
        Self: Sized;

    fn compile(&mut self) -> String;
}
