use std::{fmt, hash::Hasher, iter, ptr::NonNull, rc::Rc};

use text_size::{TextRange, TextSize};

use crate::{
    cursor::{
        NodeData, WeakGreenElement, element::SyntaxElement, node::SyntaxNode, trivia::SyntaxTrivia,
    },
    green::{
        RawSyntaxKind,
        element::{GreenElement, GreenElementRef},
        token::{GreenToken, GreenTokenData},
    },
    utility_types::{Direction, WalkEvent},
};

#[derive(Clone, Debug)]
pub(crate) struct SyntaxToken {
    ptr: Rc<NodeData>,
}

impl SyntaxToken {
    pub(super) fn new(
        green: &GreenTokenData,
        parent: SyntaxNode,
        index: u32,
        offset: TextSize,
    ) -> Self {
        Self {
            ptr: NodeData::new(
                super::NodeKind::Child {
                    green: WeakGreenElement::new(GreenElementRef::Token(green)),
                    parent: parent.ptr,
                },
                index,
                offset,
            ),
        }
    }

    pub(crate) fn new_detached(green: GreenToken) -> Self {
        Self {
            ptr: NodeData::new(
                super::NodeKind::Root {
                    green: GreenElement::Token(green),
                },
                0,
                0.into(),
            ),
        }
    }

    #[inline]
    pub(crate) fn green(&self) -> &GreenTokenData {
        match self.data().green().as_token() {
            Some(token) => token,
            None => {
                panic!(
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
            }
        }
    }

    pub(crate) fn key(&self) -> (NonNull<()>, TextSize) {
        self.data().key()
    }

    #[inline]
    pub(super) fn data(&self) -> &NodeData {
        &self.ptr
    }

    pub(super) fn into_green(self) -> GreenElement {
        self.ptr.into_green()
    }

    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data().kind()
    }

    #[inline]
    pub fn text_range(&self) -> TextRange {
        self.data().text_range()
    }

    pub fn text_trimmed_range(&self) -> TextRange {
        let green_token = self.green();
        let leading_len = green_token.leading_trivia().text_len();
        let trailing_len = green_token.trailing_trivia().text_len();

        let range = self.text_range();
        TextRange::new(range.start() + leading_len, range.end() - trailing_len)
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.data().slot() as usize
    }

    #[inline]
    pub fn text(&self) -> &str {
        self.green().text()
    }

    #[inline]
    pub fn text_trimmed(&self) -> &str {
        self.green().text_trimmed()
    }

    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }

    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        std::iter::successors(self.parent(), SyntaxNode::parent)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().next_sibling_or_token()
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement> {
        self.data().prev_sibling_or_token()
    }

    pub fn siblings_with_token(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement> + use<> {
        let next = move |el: &SyntaxElement| match direction {
            Direction::Next => el.next_sibling_or_token(),
            Direction::Prev => el.prev_sibling_or_token(),
        };

        let me: SyntaxElement = self.clone().into();

        iter::successors(next(&me), next)
    }

    pub fn next_token(&self) -> Option<Self> {
        self.next_token_impl(Direction::Next)
    }

    pub fn prev_token(&self) -> Option<Self> {
        self.next_token_impl(Direction::Prev)
    }

    #[must_use = "syntax elements are immutable, the result of update must be propagated to have any effect"]
    pub fn detach(self) -> Self {
        Self {
            ptr: self.ptr.detach(),
        }
    }

    #[inline]
    pub fn leading_trivia(&self) -> SyntaxTrivia {
        SyntaxTrivia::leading(self.clone())
    }

    #[inline]
    pub fn trailing_trivia(&self) -> SyntaxTrivia {
        SyntaxTrivia::trailing(self.clone())
    }

    /// Returns the token preceding or following this token depending on the passed direction.
    fn next_token_impl(&self, direction: Direction) -> Option<Self> {
        let mut current: WalkEvent<SyntaxElement> =
            WalkEvent::Leave(SyntaxElement::Token(self.clone()));
        loop {
            current = match current {
                WalkEvent::Enter(element) => match element {
                    SyntaxElement::Token(token) => break Some(token),
                    SyntaxElement::Node(node) => {
                        let first_child = match direction {
                            Direction::Next => node.first_child_or_token(),
                            Direction::Prev => node.last_child_or_token(),
                        };
                        match first_child {
                            // If node is empty, leave parent
                            None => WalkEvent::Leave(SyntaxElement::Node(node)),
                            // Otherwise traverse the full sub-tree
                            Some(child) => WalkEvent::Enter(child),
                        }
                    }
                },
                WalkEvent::Leave(element) => {
                    let mut current_element = element;
                    loop {
                        // Only traverse the left(pref) / right (next) sibling of the parent
                        // to avoid traversing into the same children again.
                        let sibling = match direction {
                            Direction::Next => current_element.next_sibling_or_token(),
                            Direction::Prev => current_element.prev_sibling_or_token(),
                        };

                        match sibling {
                            // Traverse all children of the sibling
                            Some(sibling) => break WalkEvent::Enter(sibling),
                            None => {
                                match current_element.parent() {
                                    Some(node) => {
                                        current_element = SyntaxElement::Node(node);
                                    }
                                    None => {
                                        return None; // Reached root, no token found
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Identity semantics for hash & eq
impl PartialEq for SyntaxToken {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Eq for SyntaxToken {}

impl std::hash::Hash for SyntaxToken {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}

impl fmt::Display for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.text(), f)
    }
}
