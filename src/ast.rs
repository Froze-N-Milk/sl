pub enum ASTNode {
    SExpr(Vec<ASTNode>),
    Atom(String),
    Prefix(String, Box<ASTNode>)
}

fn id_parser(buf: bytes::Bytes) -> Result<(bytes::Bytes, String), ()> {
    todo!()
}

