use core::fmt;

use crate::{ast::AstNode, green::RawSyntaxKind};

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
