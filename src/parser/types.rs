#[derive(Debug)]
pub enum PropType {
    Literal(String),
    Var(String),
}

#[derive(Debug)]
pub struct ASTProp {
    pub id: usize,
    pub name: String,
    pub value: Option<PropType>,
}

#[derive(Debug)]
pub enum ASTBody {
    String(String),
    Tag(Box<ASTNode>),
}

#[derive(Debug)]
pub struct ASTNode {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub name: String,
    pub children: Vec<ASTBody>,
    pub props: Vec<ASTProp>,
    pub self_closing: bool,
}
