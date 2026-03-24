use core::fmt;
use std::{
    hash::{Hash, Hasher},
    iter::FusedIterator,
    rc::Rc,
};

use text_size::{TextRange, TextSize};

use crate::{
    cursor::{NodeData, WeakGreenElement, element::SyntaxElement, token::SyntaxToken},
    green::{
        RawSyntaxKind,
        element::{GreenElement, GreenElementRef},
        node::{Child, Children, GreenNode, GreenNodeData, Slot},
    },
    utility_types::{Direction, WalkEvent},
};

#[derive(Clone)]
pub(crate) struct SyntaxNode {
    pub(super) ptr: Rc<NodeData>,
}

impl SyntaxNode {
    pub(crate) fn new_root(green: GreenNode) -> Self {
        Self {
            ptr: NodeData::new(
                super::NodeKind::Root {
                    green: GreenElement::Node(green),
                },
                0,
                0.into(),
            ),
        }
    }

    pub(crate) fn new_child(
        green: &GreenNodeData,
        parent: Self,
        slot: u32,
        offset: TextSize,
    ) -> Self {
        Self {
            ptr: NodeData::new(
                super::NodeKind::Child {
                    green: WeakGreenElement::new(GreenElementRef::Node(green)),
                    parent: parent.ptr,
                },
                slot,
                offset,
            ),
        }
    }

    pub fn clone_subtree(&self) -> Self {
        Self::new_root(self.green().into())
    }

    #[inline]
    pub(super) fn data(&self) -> &NodeData {
        self.ptr.as_ref()
    }

    #[inline]
    pub(super) fn offset(&self) -> TextSize {
        self.data().offset()
    }

    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data().kind()
    }

    #[inline]
    pub fn text_range(&self) -> TextRange {
        self.data().text_range()
    }

    #[inline]
    pub(crate) fn green(&self) -> &GreenNodeData {
        self.data().green().into_node().unwrap()
    }
}

impl PartialEq for SyntaxNode {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Eq for SyntaxNode {}

impl Hash for SyntaxNode {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}

impl fmt::Debug for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxNode")
            .field("kind", &self.kind())
            .field("text_range", &self.text_range())
            .finish()
    }
}

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SyntaxNodeChildren {
    next: Option<SyntaxNode>,
}

impl Iterator for SyntaxNodeChildren {
    type Item = SyntaxNode;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().inspect(|_next| todo!())
    }
}

impl FusedIterator for SyntaxNodeChildren {}

#[derive(Clone, Debug, Default)]
pub(crate) struct SyntaxElementChildren {
    next: Option<SyntaxElement>,
}

impl SyntaxElementChildren {
    fn new(parent: SyntaxNode) -> Self {
        todo!()
    }
}

impl Iterator for SyntaxElementChildren {
    type Item = SyntaxElement;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().inspect(|next| todo!())
    }
}

impl FusedIterator for SyntaxElementChildren {}

pub(crate) struct Preorder {
    start: SyntaxNode,
    next: Option<WalkEvent<SyntaxNode>>,
    skip_subtree: bool,
}

impl Preorder {
    fn new(start: SyntaxNode) -> Self {
        let next = Some(WalkEvent::Enter(start.clone()));
        Self {
            start,
            next,
            skip_subtree: false,
        }
    }

    pub fn skip_subtree(&mut self) {
        self.skip_subtree = true;
    }

    #[cold]
    fn do_skip(&mut self) {
        todo!()
    }
}

impl Iterator for Preorder {
    type Item = WalkEvent<SyntaxNode>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip_subtree {
            self.do_skip();
            self.skip_subtree = false;
        }
        let next = self.next.take();
        todo!()
    }
}

impl FusedIterator for Preorder {}

pub(crate) struct PreorderWithTokens {
    start: SyntaxElement,
    next: Option<WalkEvent<SyntaxElement>>,
    skip_subtree: bool,
    direction: Direction,
}

impl PreorderWithTokens {
    fn new(start: SyntaxNode, direction: Direction) -> Self {
        let next = Some(WalkEvent::Enter(start.clone().into()));
        Self {
            start: start.into(),
            next,
            direction,
            skip_subtree: false,
        }
    }

    pub fn skip_subtree(&mut self) {
        self.skip_subtree = true;
    }

    #[cold]
    fn do_skip(&mut self) {
        todo!()
    }
}

impl Iterator for PreorderWithTokens {
    type Item = WalkEvent<SyntaxElement>;

    fn next(&mut self) -> Option<WalkEvent<SyntaxElement>> {
        if self.skip_subtree {
            self.do_skip();
            self.skip_subtree = false;
        }
        let next = self.next.take();
        todo!()
    }
}

impl FusedIterator for PreorderWithTokens {}

pub(crate) struct PreorderTokens {
    next: Option<SyntaxToken>,
    direction: Direction,
}

impl PreorderTokens {
    fn new(start: SyntaxNode, direction: Direction) -> Self {
        todo!()
    }
}

impl Iterator for PreorderTokens {
    type Item = SyntaxToken;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        todo!()
    }
}

impl FusedIterator for PreorderTokens {}

/// Represents a cursor to a green node slot. A slot either contains an element or is empty
/// if the child isn't present in the source.
#[derive(Debug, Clone)]
pub(crate) enum SyntaxSlot {
    Node(SyntaxNode),
    Token(SyntaxToken),
    Empty { parent: SyntaxNode, index: u32 },
}

impl From<SyntaxElement> for SyntaxSlot {
    fn from(element: SyntaxElement) -> Self {
        match element {
            SyntaxElement::Node(node) => Self::Node(node),
            SyntaxElement::Token(token) => Self::Token(token),
        }
    }
}

impl SyntaxSlot {
    pub fn map<F, R>(self, mapper: F) -> Option<R>
    where
        F: FnOnce(SyntaxElement) -> R,
    {
        match self {
            Self::Node(node) => Some(mapper(SyntaxElement::Node(node))),
            Self::Token(token) => Some(mapper(SyntaxElement::Token(token))),
            Self::Empty { .. } => None,
        }
    }
}

/// Iterator over a node's slots
#[derive(Debug, Clone)]
pub(crate) struct SyntaxSlots {
    /// Position of the next element to return.
    pos: u32,
    /// Position of the last returned element from the back.
    /// Initially points one element past the last slot.
    ///
    /// [nth back]: https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html#method.nth_back]
    back_pos: u32,
    parent: SyntaxNode,
}

impl SyntaxSlots {
    #[inline]
    fn new(parent: SyntaxNode) -> Self {
        Self {
            pos: 0,
            back_pos: parent.green().slice().len() as u32,
            parent,
        }
    }

    /// Returns a slice containing the remaining elements to iterate over an empty
    /// slice if the iterator reached the end.
    fn slice(&self) -> &[Slot] {
        if self.pos < self.back_pos {
            &self.parent.green().slice()[self.pos as usize..self.back_pos as usize]
        } else {
            &[]
        }
    }

    fn map_slot(&self, slot: &Slot, slot_index: u32) -> SyntaxSlot {
        match slot {
            Slot::Empty { .. } => SyntaxSlot::Empty {
                parent: self.parent.clone(),
                index: slot_index,
            },
            Slot::Token { rel_offset, token } => SyntaxSlot::Token(SyntaxToken::new(
                token,
                self.parent.clone(),
                slot_index,
                self.parent.offset() + rel_offset,
            )),
            Slot::Node { rel_offset, node } => SyntaxSlot::Node(SyntaxNode::new_child(
                node,
                self.parent.clone(),
                slot_index,
                self.parent.offset() + rel_offset,
            )),
        }
    }
}

impl Iterator for SyntaxSlots {
    type Item = SyntaxSlot;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slice().first()?;
        let mapped = self.map_slot(slot, self.pos);
        self.pos += 1;
        Some(mapped)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice().len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.pos += n as u32;
        self.next()
    }
}

impl ExactSizeIterator for SyntaxSlots {
    #[inline]
    fn len(&self) -> usize {
        self.slice().len()
    }
}

impl FusedIterator for SyntaxSlots {}

impl DoubleEndedIterator for SyntaxSlots {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let slot = self.slice().last()?;
        let mapped = self.map_slot(slot, self.back_pos - 1);
        self.back_pos -= 1;
        Some(mapped)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.back_pos -= n as u32;
        self.next_back()
    }
}

/// Iterator to visit a node's slots in pre-order
pub(crate) struct SlotsPreorder {
    start: SyntaxNode,
    next: Option<WalkEvent<SyntaxSlot>>,
}

impl SlotsPreorder {
    fn new(start: SyntaxNode) -> Self {
        let next = Some(WalkEvent::Enter(SyntaxSlot::Node(start.clone())));
        Self { start, next }
    }
}

impl Iterator for SlotsPreorder {
    type Item = WalkEvent<SyntaxSlot>;

    fn next(&mut self) -> Option<WalkEvent<SyntaxSlot>> {
        let next = self.next.take();
        todo!()
    }
}

impl FusedIterator for SlotsPreorder {}

#[derive(Debug, Clone)]
pub(crate) struct Siblings<'a> {
    parent: &'a GreenNodeData,
    start_slot: u32,
}

impl<'a> Siblings<'a> {
    pub fn new(parent: &'a GreenNodeData, start_slot: u32) -> Self {
        assert!(
            (start_slot as usize) < parent.slots().len(),
            "Start slot {} out of bounds {}",
            start_slot,
            parent.slots().len()
        );
        Self { parent, start_slot }
    }

    /// Creates an iterator over the siblings following the start node.
    /// For example, the following siblings of the if statement's condition are
    /// * the consequence
    /// * potentially the else clause
    pub fn following(&self) -> Children<'a> {
        let mut slots = self.parent.slots().enumerate();
        slots.nth(self.start_slot as usize);
        Children::new(slots)
    }

    /// Creates an iterator over the siblings preceding the start node in reverse order.
    /// For example, the preceding sublings of the if statement's condition are:
    /// * opening parentheses: (
    /// * if keyword: if
    pub fn previous(&self) -> impl Iterator<Item = Child<'a>> + use<'a> {
        let mut slots = self.parent.slots().enumerate();

        // Navigate to the start slot from the back so that calling `next_back` (or rev().next()) returns
        // the first slot preceding the start node
        slots.nth_back(slots.len() - 1 - self.start_slot as usize);

        Children::new(slots).rev()
    }
}
