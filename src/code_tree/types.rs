use crate::math::math::MathToken;

// Possible data types
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Bool,
    Str,
    Void,

    Any, // Internal type, cannot be accessed from code
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NodeType {
    BLOCK(BlockStruct),
    DEFINITION(DefinitionType),
    CALL(CallStruct),
    ASSIGN(String),
}

// -------------- Block Type ---------------
#[derive(Debug, Clone)]
pub struct BlockStruct {
    pub tag: BlockType,
    pub children: Vec<Box<NodeType>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Html,
    Head,
    Main,
    Div,
}

// ----------- Definition Type -------------

#[derive(Debug, Clone)]
pub enum DefinitionType {
    Function(FunctionDefinitionStruct),
    Variable(VariableDefinitionStruct),
}

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

#[derive(Debug, Clone)]
pub struct VariableDefinitionStruct {
    pub data_type: DataType,
    pub name: String,
    pub value: MathToken,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinitionStruct {
    pub data_type: DataType,
    pub name: String,
    pub children: Vec<Box<NodeType>>,
    pub args: Vec<ArgStruct>,
}

impl FunctionDefinitionStruct {
    pub fn new(
        name: String,
        data_type: DataType,
        args: Vec<ArgStruct>,
    ) -> FunctionDefinitionStruct {
        Self {
            data_type,
            name,
            children: Vec::new(),
            args,
        }
    }
}

// ----------- Call Type ---------------

#[derive(Debug, Clone)]
pub struct CallArgStruct {
    pub name: String,
    pub value: Option<MathToken>,
}

#[derive(Debug, Clone)]
pub struct CallStruct {
    pub calling_name: String,
    pub args: Vec<CallArgStruct>,
}
