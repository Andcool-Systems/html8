#[derive(Debug, Clone)]
pub enum PropType {
    Literal(String),
    Var(String),
}

#[derive(Debug, Clone)]
pub struct ASTProp {
    pub name: String,
    pub value: Option<PropType>,
}

#[derive(Debug, Clone)]
pub enum ASTBody {
    String(String),
    Tag(Box<ASTNode>),
}

#[derive(Debug, Clone)]
pub struct ASTNode {
    pub name: String,
    pub children: Vec<ASTBody>,
    pub props: Vec<ASTProp>,
    pub self_closing: bool,
}
