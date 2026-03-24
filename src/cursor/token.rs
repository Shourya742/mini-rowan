use std::{fmt, hash::Hasher, ptr::NonNull, rc::Rc};

use text_size::{TextRange, TextSize};

use crate::{
    cursor::{NodeData, WeakGreenElement, node::SyntaxNode},
    green::{
        self, RawSyntaxKind,
        element::{GreenElement, GreenElementRef},
        token::{GreenToken, GreenTokenData},
    },
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
    pub fn index(&self) -> &str {
        self.green().text()
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
