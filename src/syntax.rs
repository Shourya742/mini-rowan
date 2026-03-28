use core::fmt;

use crate::{
    ast::AstNode,
    green::RawSyntaxKind,
    syntax::node::{SyntaxNode, SyntaxSlot, SyntaxSlots},
};

pub mod element;
pub mod node;
pub mod rewriter;
pub mod token;
pub mod trivia;

/// Type tag for each node or token of a language
pub trait SyntaxKind: fmt::Debug + PartialEq + Copy {
    const TOMBSTONE: Self;
    const EOF: Self;

    /// Returns `true` if this is a kind of a bogus node.
    fn is_bogus(&self) -> bool;

    /// Converts this into the best matching bogus node kind.
    fn to_bogus(&self) -> Self;

    /// Converts this kind to a raw syntax kind.
    fn to_raw(&self) -> RawSyntaxKind;

    /// Creates a syntax kind from a raw kind.
    fn from_raw(raw: RawSyntaxKind) -> Self;

    /// Returns `true` if this kind is for a root node.
    fn is_root(&self) -> bool;

    /// Returns `true` if this kind is a list node.
    fn is_list(&self) -> bool;

    /// Returns `true` if this kind is a trivia.
    fn is_trivia(self) -> bool;

    /// Returns a string for keywords, punctuation tokens, and the `EOL` token,
    /// or `None` otherwise.
    fn to_string(&self) -> Option<&'static str>;

    /// Returns 'true' if this kind is allowed to precede file suppression comments.
    fn is_allowed_before_suppressions(&self) -> bool {
        false
    }
}

pub trait Language: Sized + Clone + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
    type Kind: SyntaxKind;
    type Root: AstNode<Language = Self> + Clone + Eq + fmt::Debug;
}

/// A list of `SyntaxNode's` and/or `SyntaxToken`s
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SyntaxList<L: Language> {
    list: SyntaxNode<L>,
}

impl<L: Language> SyntaxList<L> {
    /// Creates a new list wrapping a List `SyntaxNode`
    fn new(node: SyntaxNode<L>) -> Self {
        Self { list: node }
    }

    /// Iterates over the elements in the list.
    pub fn iter(&self) -> SyntaxSlots<L> {
        self.list.slots()
    }

    /// Returns the number of items in this list
    pub fn len(&self) -> usize {
        self.list.slots().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn first(&self) -> Option<SyntaxSlot<L>> {
        self.list.slots().next()
    }

    pub fn last(&self) -> Option<SyntaxSlot<L>> {
        self.list.slots().last()
    }

    pub fn node(&self) -> &SyntaxNode<L> {
        self.list.as_ref()
    }

    pub fn into_node(self) -> SyntaxNode<L> {
        self.list
    }
}

impl<L: Language> IntoIterator for &SyntaxList<L> {
    type Item = SyntaxSlot<L>;
    type IntoIter = SyntaxSlots<L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<L: Language> IntoIterator for SyntaxList<L> {
    type Item = SyntaxSlot<L>;
    type IntoIter = SyntaxSlots<L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
