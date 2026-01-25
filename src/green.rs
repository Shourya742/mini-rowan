use std::{fmt::Display, iter, sync::Arc};

use crate::{NodeOrToken, SyntaxKind};

pub type GreenElement = NodeOrToken<GreenNode, GreenToken>;

impl From<GreenNode> for GreenElement {
    fn from(value: GreenNode) -> Self {
        NodeOrToken::Node(value)
    }
}

impl From<GreenToken> for GreenElement {
    fn from(value: GreenToken) -> Self {
        NodeOrToken::Token(value)
    }
}

pub type GreenToken = Arc<GreenTokenData>;

#[derive(Clone, Debug)]
pub struct GreenTokenData {
    kind: SyntaxKind,
    text: String,
}

impl Display for GreenTokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.text(), f)
    }
}

pub type GreenNode = Arc<GreenNodeData>;

#[derive(Clone, Debug)]
pub struct GreenNodeData {
    kind: SyntaxKind,
    children: Vec<GreenElement>,
    len: usize,
}

impl Display for GreenNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for child in self.children() {
            Display::fmt(&child, f);
        }
        Ok(())
    }
}

impl GreenTokenData {
    pub fn new(kind: SyntaxKind, text: String) -> GreenTokenData {
        GreenTokenData { kind, text }
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn text_len(&self) -> usize {
        self.text().len()
    }
}

impl GreenNodeData {
    pub fn new(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNodeData {
        let len = children.iter().map(|it| it.text_len()).sum();
        GreenNodeData {
            kind,
            children,
            len,
        }
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    pub fn text_len(&self) -> usize {
        self.len
    }

    pub fn children(&self) -> impl Iterator<Item = GreenElement> {
        self.children.iter().cloned()
    }

    pub fn replace_child(&self, idx: usize, new_child: GreenElement) -> GreenNodeData {
        assert!(idx < self.children.len());

        let left_children = self.children().take(idx);
        let right_children = self.children().skip(idx + 1);
        let new_children: Vec<_> = left_children
            .chain(iter::once(new_child))
            .chain(right_children)
            .collect();

        GreenNodeData::new(self.kind, new_children)
    }
}

impl GreenElement {
    pub fn text_len(&self) -> usize {
        match self {
            NodeOrToken::Node(node) => node.text_len(),
            NodeOrToken::Token(token) => token.text_len(),
        }
    }
}

impl Display for GreenElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeOrToken::Node(it) => Display::fmt(it, f),
            NodeOrToken::Token(it) => Display::fmt(it, f),
        }
    }
}
