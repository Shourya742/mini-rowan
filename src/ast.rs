use std::marker::PhantomData;

use crate::{
    green::RawSyntaxKind,
    syntax::{Language, SyntaxKind},
};

/// Represents a set of [SyntaxKind] as a bitfield, with each bit representing
/// whether the corresponding [RawSyntaxKind] value is contained in the set.
///
/// This is similar to the `TokenSet` struct, with the bitfield here being twice
/// as large as it needs to cover all nodes as well as all tokens kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxKindSet<L: Language>([u128; 5], PhantomData<L>);

impl<L> SyntaxKindSet<L>
where
    L: Language,
{
    /// Create a new [SyntaxKindSet] containing only the provided [RawSyntaxKind]
    pub fn of(kind: L::Kind) -> Self {
        Self::from_raw(kind.to_raw())
    }

    /// Create a new [SyntaxKindSet] containing only the provided [RawSyntaxKind]
    ///
    /// Unlike `SyntaxKindSet::of` this function can be evaluated in constants,
    /// and will result in a compile-time error if the value overflows:
    pub const fn from_raw(kind: RawSyntaxKind) -> Self {
        let RawSyntaxKind(kind) = kind;

        let index = kind as usize / u128::BITS as usize;
        let shift = kind % u128::BITS as u16;
        let mask = 1 << shift;
        let mut bits = [0; 5];
        bits[index] = mask;
        Self(bits, PhantomData)
    }

    /// Returns the union of the two sets `self` and `other`
    pub const fn union(self, other: Self) -> Self {
        Self(
            [
                self.0[0] | other.0[0],
                self.0[1] | other.0[1],
                self.0[2] | other.0[2],
                self.0[3] | other.0[3],
                self.0[4] | other.0[4],
            ],
            PhantomData,
        )
    }

    /// Returns true if `kind` is contained in this set
    pub fn matches(self, kind: L::Kind) -> bool {
        let RawSyntaxKind(kind) = kind.to_raw();

        let index = kind as usize / u128::BITS as usize;
        let shift = kind % u128::BITS as u16;
        let mask = 1 << shift;

        self.0[index] & mask != 0
    }

    /// Returns an iterator over all the [SyntaxKind] contained in this set
    pub fn iter(self) -> impl Iterator<Item = L::Kind> {
        self.0.into_iter().enumerate().flat_map(|(index, item)| {
            let index = index as u16 * u128::BITS as u16;
            (0..u128::BITS).filter_map(move |bit| {
                if (item & (1 << bit)) != 0 {
                    let raw = index + bit as u16;
                    let raw = RawSyntaxKind(raw);
                    Some(<L::Kind as SyntaxKind>::from_raw(raw))
                } else {
                    None
                }
            })
        })
    }
}

/// The main trait to go from untyped `SyntaxNode` to a typed ast. The
/// conversion itself has zero runtime cost: ast and Syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
///
/// The only exception to this is for Dynamic nodes, which allow the fields
/// of the AstNode to be mapped to any slot of the SyntaxNode using an additional
/// `slot_map`. This must get build every time the untyped syntax node is
/// converted into the typed ast node, and is determined by the order of fields
/// in the original grammar. Even still, this cost is relatively low and should
/// not be considered prohibitive, as the only work done is checking
/// [AstNode::can_cast] for each of the children to their respective slots.
pub trait AstNode: Clone {
    type Language: Language;

    const KIND_SET: SyntaxKindSet<Self::Language>;

    /// Returns `true` if a node with the give kind can be cased to this AST node.
    fn can_cast(kind: <Self::Language as Language>::Kind) -> bool;
}
