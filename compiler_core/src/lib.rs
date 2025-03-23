use crate::math::ExprToken;

pub mod definitions;
pub mod iter;
pub mod libs;
pub mod math;

pub fn process_expr_token(token: ExprToken) -> String {
    match token {
        ExprToken::Number(n) => format!("{}", n),
        ExprToken::Variable(v) => v.name,
        ExprToken::Literal(l) => format!("\"{}\"", l),
        ExprToken::Add(l, r) => format!("{} + {}", process_expr_token(*l), process_expr_token(*r)),
        ExprToken::Sub(l, r) => format!("{} - {}", process_expr_token(*l), process_expr_token(*r)),
        ExprToken::Mul(l, r) => format!("{} * {}", process_expr_token(*l), process_expr_token(*r)),
        ExprToken::Div(l, r) => format!("{} / {}", process_expr_token(*l), process_expr_token(*r)),
        ExprToken::Pow(l, r) => format!(
            "pow({}, {})",
            process_expr_token(*l),
            process_expr_token(*r)
        ),
    }
}

pub mod backend {
    use super::types::NodeType;

    pub trait Backend {
        fn generate_code(&mut self, node: &NodeType) -> String;
        fn save_code(&self, path: Option<&str>) -> Result<String, String>;
        fn compile(&self) -> Result<(), String>;
        fn run(&self) -> Result<String, String>;
        fn supports_feature(&self, feature: &str) -> bool;
    }

    #[repr(C)]
    pub struct BackendInfo {
        pub name: String,
        pub compiler: String,
        pub compiler_args: Vec<String>,
        pub input_file: String, // Теперь файлы хранятся в инфо
        pub output_file: String,
    }
}

pub mod types {
    use crate::math::ExprToken;

    // Possible data types
    #[repr(C)]
    #[derive(Debug, Clone, PartialEq)]
    pub enum DataType {
        Int,
        Bool,
        Str,
        Void,

        Any, // Internal type, cannot be accessed from code
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    #[allow(dead_code, clippy::upper_case_acronyms)]
    pub enum NodeType {
        BLOCK(BlockStruct),
        DEFINITION(DefinitionType),
        CALL(CallStruct),
        ASSIGN(AssignStruct),
        ServiceBlock(ServiceBlockType),
    }

    // -------------- Block Type ---------------
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct BlockStruct {
        pub tag: BlockType,
        #[allow(clippy::vec_box)]
        pub children: Vec<Box<NodeType>>,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BlockType {
        Html,
        Head,
        Main,
        Div,
        For,
    }

    // ----------- Definition Type -------------
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub enum DefinitionType {
        Function(FunctionDefinitionStruct),
        Variable(VariableDefinitionStruct),
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct ArgStruct {
        pub name: String,
        pub data_type: DataType,
    }

    impl ArgStruct {
        pub fn new(name: String, data_type: DataType) -> ArgStruct {
            Self { name, data_type }
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct VariableDefinitionStruct {
        pub data_type: DataType,
        pub name: String,
        pub value: AssignEnum,
        pub is_const: bool,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct FunctionDefinitionStruct {
        pub data_type: DataType,
        pub name: String,
        #[allow(clippy::vec_box)]
        pub children: Vec<Box<NodeType>>,
        pub args: Vec<ArgStruct>,
        pub must_be_compiled: bool,
    }

    impl FunctionDefinitionStruct {
        pub fn new_internal(
            name: String,
            data_type: DataType,
            args: Vec<ArgStruct>,
            must_be_compiled: bool,
        ) -> FunctionDefinitionStruct {
            Self {
                data_type,
                name,
                children: Vec::new(),
                args,
                must_be_compiled,
            }
        }
    }

    // ----------- Call Type ---------------
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct CallArgStruct {
        pub name: String,
        pub value: Option<ExprToken>,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct CallStruct {
        pub calling_name: String,
        pub args: Vec<CallArgStruct>,
    }

    // ----------- Assign Type ---------------
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub enum AssignEnum {
        Expr(ExprToken),
        Call(Box<NodeType>),
        None,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct AssignStruct {
        pub name: String,
        pub body: AssignEnum,
    }

    impl AssignStruct {
        pub fn new(name: String) -> Self {
            AssignStruct {
                name,
                body: AssignEnum::None,
            }
        }
    }

    // ----------- Service Block Type ---------------
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub enum ServiceBlockType {
        For(ForStruct),
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct ForStruct {
        pub start: ExprToken,
        pub end: ExprToken,
        pub iter_name: String,
        pub children: Vec<Box<NodeType>>,
    }
}
