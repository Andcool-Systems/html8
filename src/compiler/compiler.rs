use crate::{
    code_tree::types::{
        CallStruct, DataType, DefinitionType, FunctionDefinitionStruct, NodeType,
        VariableDefinitionStruct,
    },
    libs::std::STD,
    math::math::ExprToken,
};

pub struct CLang {
    pub tree: NodeType,
}

impl CLang {
    pub fn new(tree: NodeType) -> Self {
        Self { tree }
    }

    fn convert_types(data_type: DataType) -> String {
        match data_type {
            DataType::Int => String::from("int"),
            DataType::Bool => String::from("bool"),
            DataType::Str => todo!(),
            DataType::Void => String::from("void"),
            _ => String::new(),
        }
    }

    pub fn compile(&mut self) -> String {
        let compiled = self._compile(self.tree.clone());
        format!(
            "#include <stdio.h>\n#include <math.h>\nint main(void){{{}\nreturn 0;\n}}",
            compiled
        )
    }

    fn _compile(&mut self, node: NodeType) -> String {
        match node {
            NodeType::BLOCK(block_struct) => {
                let mut vec: Vec<String> = Vec::new();
                for child in block_struct.children {
                    vec.push(self._compile(*child));
                }
                vec.join("\n")
            }
            NodeType::DEFINITION(definition_type) => match definition_type {
                DefinitionType::Function(fds) => self.compile_fn(fds),
                DefinitionType::Variable(vds) => self.compile_var(vds),
            },
            NodeType::CALL(call_struct) if call_struct.calling_name.eq("println") => {
                STD::compile_println(call_struct)
            }
            NodeType::CALL(call_struct) => self.compile_call(call_struct),
            NodeType::ASSIGN(_) => todo!(),
        }
    }

    fn compile_var(&mut self, v: VariableDefinitionStruct) -> String {
        format!(
            "{} {} = {};",
            Self::convert_types(v.data_type),
            v.name,
            Self::process_expr_token(v.value)
        )
    }

    fn compile_fn(&mut self, f: FunctionDefinitionStruct) -> String {
        if !f.must_be_compiled {
            return String::new();
        }

        let args = f
            .args
            .iter()
            .map(|arg| {
                format!(
                    "{} {}",
                    Self::convert_types(arg.data_type.clone()),
                    arg.name
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let mut children: Vec<String> = Vec::new();
        for child in f.children {
            children.push(self._compile(*child));
        }

        format!(
            "{} {} ({}) {{ {} }}",
            Self::convert_types(f.data_type),
            f.name,
            args,
            children.join("\n")
        )
    }

    fn compile_call(&mut self, call: CallStruct) -> String {
        let args = call
            .args
            .iter()
            .map(|a| Self::process_expr_token(a.value.clone().unwrap()))
            .collect::<Vec<String>>()
            .join("\n");
        format!("{}({});", call.calling_name, args)
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
