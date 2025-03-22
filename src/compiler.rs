use crate::code_tree::types::ArgStruct;
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
        let (functions, statements): (String, String) = self._compile(self.tree.clone());
        format!(
            "#include <stdio.h>\n#include <math.h>\n\n{}\nint main(void){{\n{}return 0;\n}}",
            functions, statements
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

    fn _compile(&mut self, node: NodeType) -> (String, String) {
        match node {
            NodeType::BLOCK(block_struct) => {
                let (mut functions, mut statements): (String, String) =
                    (String::new(), String::new());

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
                        let (func, stmt): (String, String) = self._compile(*child);

                        (!func.is_empty()).then(|| {
                            functions.push_str(&func);
                            functions.push('\n');
                        });

                        (!stmt.is_empty()).then(|| {
                            statements.push_str(&stmt);
                            statements.push('\n');
                        });
                    });

                (functions, statements)
            }
            NodeType::DEFINITION(definition_type) => match definition_type {
                DefinitionType::Function(fds) => (self.compile_fn(fds), String::new()),
                DefinitionType::Variable(vds) => (String::new(), self.compile_var(vds)),
            },
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("println") => {
                (String::new(), Std::compile_println(call_struct))
            }
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("print") => {
                (String::new(), Std::compile_print(call_struct))
            }
            NodeType::CALL(call_struct) => (String::new(), self.compile_call(call_struct)),
            NodeType::ASSIGN(_) => todo!("UNIMPLEMENTED"),
        }
    }

    fn compile_var(&mut self, v: VariableDefinitionStruct) -> String {
        let arr: String = (v.data_type == DataType::Str)
            .then(|| String::from("*"))
            .unwrap_or_else(String::new);

        format!(
            "{} {}{} = {};",
            Self::convert_types(v.data_type),
            v.name,
            arr,
            Self::process_expr_token(v.value)
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
            let (_, stmt): (String, String) = self._compile(*child);
            (!stmt.is_empty()).then(|| children.push(stmt));
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

    pub fn process_expr_token(token: ExprToken) -> String {
        match token {
            ExprToken::Number(n) => format!("{}", n),
            ExprToken::Variable(v) => v,
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
