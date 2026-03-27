use std::marker::PhantomData;

use crate::{cursor, syntax::Language};





#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxNode<L: Language> {
    raw: cursor::node::SyntaxNode,
    _p: PhantomData<L>,
}
