pub mod element;
pub mod node;
pub mod node_cache;
pub mod token;
pub mod trivia;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawSyntaxKind(pub u16);
