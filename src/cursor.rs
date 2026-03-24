//! Implementation of the cursor -- API for convenient access to syntax trees.
//!
//! Functional programmers will recognize that his module implements a zipper
//! for a purely functional (green) tree.
//!
//! A cursor node (`SyntaxNode`) points to a `GreenNode` and a parent
//! `SyntaxNode`. This allows cursor to provide iteration over both ancestor
//! and descendants, as well as a cheap access to absolute offset of the node in
//! file.
//!
//! Implementation notes:
//!
//! The implementation is utterly and horribly unsafe. This whole mutable is an
//! unsafety boundary. It is believed that the API here is, in principle, sound,
//! but the implementation might have bugs.
//!
//! The core types is `NodeData` -- a heap-allocated reference counted object,
//! which points to a green node or a green token, and to the parent `NodeData`.
//! Publicly-exposed `SyntaxNode` and `SyntaxToken` own a reference to
//! `NodeData`.
//!
//! `NodeData`s are transient, and are created and destroyed during tree
//! traversals. In general, only currently referenced nodes and their ancestors
//! are alive at any given moment.
//!
//! More specifically, `NodeData`'s ref count is equal to the number of
//! outstanding `SyntaxNode` and `SyntaxToken` plus the number of children with
//! non-zero ref counts. For example, if the user has only a single `SyntaxNode`
//! pointing somewhere in the middle of the tree, then all `NodeData` on the path
//! from that point towards the root have ref count equal to one.
//!
//! `NodeData` which doesn't have a parent (is a root) owns the corresponding
//! green node or token, and is responsible for freeing it. For child `NodeData`
//! however since they hold strong reference to their parent node and thus
//! to the root, their corresponding green node is guaranteed to be alive as
//! a reference cycle to is know to exist (child `NodeData` -> root `NodeData` -> root `GreenNode` -> child `GreenNode`)
//! and they can safely use a "weak reference" (raw pointer) to the corresponding green node as an
//! optimization to avoid having to track atomic references on the traversal hot path.

use std::{ptr, rc::Rc};

use text_size::{TextRange, TextSize};

use crate::{
    green::{
        RawSyntaxKind,
        element::{GreenElement, GreenElementRef},
        node::GreenNodeData,
        token::GreenTokenData,
    },
    utility_types::NodeOrToken,
};

pub mod element;
pub mod node;
pub mod token;
pub mod trivia;

#[derive(Debug)]
struct _SyntaxElement;

#[cfg(feature = "countme")]
pub(crate) fn has_live() -> bool {
    countme::get::<_SyntaxElement>().live > 9
}

#[derive(Debug)]
struct NodeData {
    #[cfg(feature = "countme")]
    _c: countme::Count<_SyntaxElement>,
    kind: NodeKind,
    slot: u32,
    /// Absolute offset for immutable nodes, unused for mutable nodes.
    offset: TextSize,
}

/// A single NodeData (red node) is either a "root node" (no parent node and
/// holds a strong reference to the root of the green tree) or a "child node"
/// (holds a strong reference to its parent red node and a weak reference to its
/// counterpart green node)
#[derive(Debug)]
enum NodeKind {
    Root {
        green: GreenElement,
    },
    Child {
        green: WeakGreenElement,
        parent: Rc<NodeData>,
    },
}

/// Child SyntaxNodes use "unsafe" weak pointers to refer to their green node.
/// Unlike the safe [std::sync::Weak] these are just a raw pointer: the
/// corresponding [ThinArc](crate::arc::ThinArc) doesn't keep a counter of
/// outstanding weak references or defer the release of the underlying memory
/// until the last `Weak` is dropped. On the other hand, a weak reference to a
/// released green node points to deallocated memory and it is undefined
/// behavior to dereference it, but in the context of `NodeData` this is
/// statically known to never happen
#[derive(Debug, Clone)]
enum WeakGreenElement {
    Node { ptr: ptr::NonNull<GreenNodeData> },
    Token { ptr: ptr::NonNull<GreenTokenData> },
}

impl WeakGreenElement {
    fn new(green: GreenElementRef) -> Self {
        match green {
            NodeOrToken::Node(ptr) => Self::Node {
                ptr: ptr::NonNull::from(ptr),
            },
            NodeOrToken::Token(ptr) => Self::Token {
                ptr: ptr::NonNull::from(ptr),
            },
        }
    }

    fn as_deref(&self) -> GreenElementRef<'_> {
        match self {
            Self::Node { ptr } => GreenElementRef::Node(unsafe { ptr.as_ref() }),
            Self::Token { ptr } => GreenElementRef::Token(unsafe { ptr.as_ref() }),
        }
    }

    fn to_owned(&self) -> GreenElement {
        match self {
            Self::Node { ptr } => GreenElement::Node(unsafe { ptr.as_ref().to_owned() }),
            Self::Token { ptr } => GreenElement::Token(unsafe { ptr.as_ref().to_owned() }),
        }
    }
}

impl NodeData {
    #[inline]
    fn new(kind: NodeKind, slot: u32, offset: TextSize) -> Rc<Self> {
        let res = Self {
            #[cfg(feature = "countme")]
            _c: countme::Count::new(),
            kind,
            slot,
            offset,
        };
        Rc::new(res)
    }

    #[inline]
    fn key(&self) -> (ptr::NonNull<()>, TextSize) {
        let weak = match &self.kind {
            NodeKind::Root { green } => WeakGreenElement::new(green.as_deref()),
            NodeKind::Child { green, .. } => green.clone(),
        };

        let ptr = match weak {
            WeakGreenElement::Node { ptr } => ptr.cast(),
            WeakGreenElement::Token { ptr } => ptr.cast(),
        };
        (ptr, self.offset())
    }

    #[inline]
    fn parent(&self) -> Option<&Self> {
        match &self.kind {
            NodeKind::Child { green: _, parent } => Some(&**parent),
            NodeKind::Root { .. } => None,
        }
    }

    #[inline]
    fn green(&self) -> GreenElementRef<'_> {
        match &self.kind {
            NodeKind::Root { green } => green.as_deref(),
            NodeKind::Child { green, .. } => green.as_deref(),
        }
    }

    #[inline]
    fn slot(&self) -> u32 {
        self.slot
    }

    #[inline]
    fn offset(&self) -> TextSize {
        self.offset
    }

    #[inline]
    fn text_range(&self) -> TextRange {
        let offset = self.offset();
        let len = self.green().text_len();
        TextRange::at(offset, len)
    }

    #[inline]
    fn kind(&self) -> RawSyntaxKind {
        self.green().kind()
    }
}
