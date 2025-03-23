use compiler_core::backend::{Backend, BackendInfo};
use compiler_core::libs::std::Std;
use compiler_core::process_expr_token;
use compiler_core::types::{
    AssignEnum, AssignStruct, BlockType, CallStruct, DataType, DefinitionType, ForStruct,
    FunctionDefinitionStruct, NodeType, ServiceBlockType, VariableDefinitionStruct,
};
use rand::Rng;
use rand::distr::Alphanumeric;

const C_KEYWORDS: &[&str] = &[
    "return", "int", "char", "void", "if", "else", "while", "for", "do", "break", "continue",
    "println", "print",
];

pub struct CLang;

impl Backend for CLang {
    fn generate_code(&mut self, node: &NodeType) -> String {
        format!(
            "#include <stdio.h>\n#include <math.h>\n\nint main(void) {{\n{}return 0;\n}}",
            self.compile_node(node.clone())
        )
    }

    fn save_code(&self, _path: Option<&str>) -> Result<String, String> {
        Err("CLang не поддерживает сохранение напрямую".to_string())
    }

    fn compile(&self) -> Result<(), String> {
        Err("CLang не поддерживает компиляцию напрямую".to_string())
    }

    fn run(&self) -> Result<String, String> {
        Err("CLang не поддерживает запуск напрямую".to_string())
    }

    fn supports_feature(&self, feature: &str) -> bool {
        matches!(feature, "c" | "loops" | "functions" | "variables")
    }
}

impl CLang {
    pub fn new() -> Self {
        CLang {}
    }

    fn convert_types(data_type: DataType) -> String {
        match data_type {
            DataType::Int => String::from("int"),
            DataType::Bool => String::from("int"),
            DataType::Str => String::from("char"),
            DataType::Void => String::from("void"),
            DataType::Any => String::from("int"),
        }
    }

    fn compile_node(&self, node: NodeType) -> String {
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

                block_struct.children.into_iter().for_each(|child| {
                    if matches!(
                        *child, NodeType::DEFINITION(DefinitionType::Function(ref fds))
                        if !fds.must_be_compiled
                    ) {
                        return;
                    }
                    let stmt = self.compile_node(*child);
                    if !stmt.is_empty() {
                        statements.push_str(&stmt);
                        statements.push('\n');
                    }
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

    fn compile_for(&self, for_struct: ForStruct) -> String {
        let mut children: Vec<String> = Vec::new();
        for_struct.children.into_iter().for_each(|child| {
            let stmt_string = self.compile_node(*child);
            (!stmt_string.is_empty()).then(|| {
                children.push(stmt_string);
            });
        });
        let random_name = Self::random_string(7);

        format!(
            "{}\nloop_{}:\nif({}>={}){{goto end_{};}}\n{}\n{}++;\ngoto loop_{};\nend_{}:",
            children.get(0).unwrap_or(&String::new()),
            random_name,
            for_struct.iter_name,
            process_expr_token(for_struct.end),
            random_name,
            children[1..].join("\n"),
            for_struct.iter_name,
            random_name,
            random_name
        )
    }

    fn compile_var(&self, v: VariableDefinitionStruct) -> String {
        let arr = if v.data_type == DataType::Str {
            "*"
        } else {
            ""
        };
        let value = match v.value {
            AssignEnum::Expr(expr_token) => process_expr_token(expr_token),
            AssignEnum::Call(node_type) => match *node_type {
                NodeType::CALL(call_struct) => self.compile_call(call_struct),
                _ => unreachable!(),
            },
            AssignEnum::None => unreachable!(),
        };

        format!(
            "{} {}{} = {};",
            Self::convert_types(v.data_type),
            arr,
            v.name,
            value
        )
    }

    fn compile_fn(&self, f: FunctionDefinitionStruct) -> String {
        let fn_name = if is_c_keyword(&f.name) {
            format!("{}_func", f.name)
        } else {
            f.name.clone()
        };

        let args = f
            .args
            .iter()
            .map(|arg| {
                let pointer = if arg.data_type == DataType::Str {
                    "*"
                } else {
                    ""
                };
                format!(
                    "{}{} {}",
                    Self::convert_types(arg.data_type.clone()),
                    pointer,
                    arg.name
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        let mut children = Vec::new();
        f.children.into_iter().for_each(|child| {
            let stmt_string = self.compile_node(*child);
            if !stmt_string.is_empty() {
                children.push(stmt_string);
            }
        });

        format!(
            "{} {}({}) {{\n{}\n}}",
            Self::convert_types(f.data_type),
            fn_name,
            args,
            children.join("\n")
        )
    }

    fn compile_call(&self, call: CallStruct) -> String {
        let calling_name = if is_c_keyword(&call.calling_name) {
            format!("{}_func", call.calling_name)
        } else {
            call.calling_name.clone()
        };

        let args = call
            .args
            .iter()
            .map(|a| process_expr_token(a.value.clone().unwrap()))
            .collect::<Vec<String>>()
            .join(", ");

        format!("{}({});", calling_name, args)
    }

    fn compile_assign(&self, assign_struct: AssignStruct) -> String {
        match assign_struct.body {
            AssignEnum::Expr(expr_token) => {
                format!(
                    "{} = {};",
                    assign_struct.name,
                    process_expr_token(expr_token)
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

    fn random_string(length: usize) -> String {
        rand::rng()
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_backend_info() -> *mut BackendInfo {
    Box::into_raw(Box::new(BackendInfo {
        name: "C-lang".to_string(),
        compiler: "gcc".to_string(),
        compiler_args: vec![
            "-O2".to_string(),
            "-Wall".to_string(),
            "{input}".to_string(), // Шаблон для входного файла
            "-o".to_string(),
            "{output}".to_string(), // Шаблон для выходного файла
        ],
        input_file: "./output/in.c".to_string(),
        output_file: "./output/out".to_string(),
    }))
}

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create_backend() -> *mut dyn Backend {
    let backend = CLang::new();
    Box::into_raw(Box::new(backend)) as *mut dyn Backend
}

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn destroy_backend(backend: *mut dyn Backend) {
    if !backend.is_null() {
        unsafe {
            let _ = Box::from_raw(backend);
        };
    }
}
