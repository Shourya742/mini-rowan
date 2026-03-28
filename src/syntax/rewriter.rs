//! A module that exports utilities to rewrite a syntax trees.

use crate::syntax::{
    Language,
    node::{SyntaxNode, SyntaxSlot},
    token::SyntaxToken,
};

/// A visitor that re-writes a syntax tree while visiting the nodes.
///
/// The rewriter visits the nodes in pre-order from top-down.
/// Meaning, it first visits the `root`, and the visits the children of the root from left to right,
/// recursively traversing into child nodes and calling [`visit_node`](SyntaxRewriter) for every node.
///
/// Inspired by Roslyn's [`CSharpSyntaxRewriter`](https://docs.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.csharp.csharpsyntaxrewriter?view=roslyn-dotnet-4.2.0)
///
/// # Unsupported
///
/// The current implementation does not yet support node removal
///
/// # Examples
///
/// Implementation of a rewritten that replaces all literal expression nodes that contain a number token
/// with a bogus node.
///
/// ```
/// struct ReplaceNumberLiteralRewriter;
///
/// impl SyntaxRewriter for ReplaceNumberLiteralRewriter {
///
///     type Language = RawLanguage;
/// }
/// ```

pub trait SyntaxRewriter {
    type Language: Language;

    /// Recursively transforms the subtree of `node` by calling [`visit_node`](SyntaxRewriter::visit_node)
    /// for every token and [`visit_token`](SyntaxRewriter::visit token) for every token in the subtree.
    ///
    /// Returns a new syntax tree reflecting the changes by the rewriter if it replaced any node and
    /// returns `node` if no changes were made
    fn transform(&mut self, node: SyntaxNode<Self::Language>) -> SyntaxNode<Self::Language>
    where
        Self: Sized,
    {
        match self.visit_node(node) {
            VisitNodeSignal::Replace(updated) => updated,
            VisitNodeSignal::Traverse(node) => traverse(node, self),
        }
    }

    /// Called for every node in the tree. The method should return a signal specifying what should be done with the node
    ///
    /// *[VisitNodeSignal::Traverse]: Recourse into `node` so that [`visit_node`](SyntaxRewriter::visit_node)
    /// gets called for all children of `node`. The `node` will only be replaced if any node in its subtree changes.
    /// * [VisitNodeSignal::Replace]: Replaces `node` with the node specified in the [`Replace`](VisitNodeSignal::Replace) variant.
    /// It's your responsibility to call [`traverse`](SyntaxRewrite::transform) for any child of `node` for which you want the rewriter
    /// to recurse into its content
    fn visit_node(&mut self, node: SyntaxNode<Self::Language>) -> VisitNodeSignal<Self::Language> {
        VisitNodeSignal::Traverse(node)
    }

    /// Called for every token in the tree. Returning a new token changes the token in the parent node.
    fn visit_token(&mut self, token: SyntaxToken<Self::Language>) -> SyntaxToken<Self::Language> {
        token
    }
}

pub enum VisitNodeSignal<L: Language> {
    /// Signals the [SyntaxRewriter] to replace the current node with the specified node.
    Replace(SyntaxNode<L>),
    /// Signals the [SyntaxRewriter] to traverse the current node with the specified node.
    Traverse(SyntaxNode<L>),
}

fn traverse<R>(mut parent: SyntaxNode<R::Language>, rewriter: &mut R) -> SyntaxNode<R::Language>
where
    R: SyntaxRewriter,
{
    for slot in parent.slots() {
        match slot {
            SyntaxSlot::Node(node) => {
                let original_key = node.key();
                let index = node.index();

                let updated = rewriter.transform(node);

                if updated.key() != original_key {
                    parent = parent.splice_slots(index..=index, [Some(updated.into())]);
                }
            }
            SyntaxSlot::Token(token) => {
                let original_key = token.key();
                let index = token.index();

                let updated = rewriter.visit_token(token);

                if updated.key() != original_key {
                    parent = parent.splice_slots(index..=index, [Some(updated.into())]);
                }
            }
            SyntaxSlot::Empty { .. } => {
                // Nothing to visit
            }
        }
    }

    parent
}
