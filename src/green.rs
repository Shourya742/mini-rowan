mod builder;
mod element;
mod node;
mod node_cache;
mod token;



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);