pub mod element;
pub mod node;
pub mod token;
pub mod trivia;
pub mod node_cache;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSyntaxKind(pub u16);
