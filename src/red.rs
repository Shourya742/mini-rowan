// Goals:
//
// 1. .parent()
// 2. .text_offset()
//
// In the green node we only know the length of the node itself, but we do not know its relative position in the syntax tree
// For red node we want to get the offset of a node from the start of syntax tree

use std::{fmt::Display, rc::Rc, sync::Arc};

use crate::{
    NodeOrToken, SyntaxKind,
    green::{GreenElement, GreenNode, GreenToken},
};

pub type RedNode = Rc<RedNodeData>;

#[derive(Clone)]
pub struct RedNodeData {
    parent: Option<RedNode>,
    index_in_parent: usize,
    text_offset: usize,
    green: GreenNode,
}

pub type RedToken = Rc<RedTokenData>;

#[derive(Clone)]
pub struct RedTokenData {
    parent: Option<RedNode>,
    index_in_parent: usize,
    text_offset: usize,
    green: GreenToken,
}

pub type RedElement = NodeOrToken<RedNode, RedToken>;

impl From<RedNode> for RedElement {
    fn from(value: RedNode) -> Self {
        NodeOrToken::Node(value)
    }
}

impl From<RedToken> for RedElement {
    fn from(value: RedToken) -> Self {
        NodeOrToken::Token(value)
    }
}

impl RedNodeData {
    pub fn new_root(root: GreenNode) -> RedNode {
        Rc::new(RedNodeData {
            parent: None,
            index_in_parent: 0,
            text_offset: 0,
            green: root,
        })
    }

    fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }

    pub fn text_offset(&self) -> usize {
        self.text_offset
    }

    pub fn text_len(&self) -> usize {
        self.green().text_len()
    }

    pub fn parent(&self) -> Option<&RedNode> {
        self.parent.as_ref()
    }

    pub fn children<'a>(self: &'a RedNode) -> impl Iterator<Item = RedElement> + 'a {
        let mut offset_in_parent = 0;
        self.green().children().enumerate().map(move |(idx, green_child)| {
            let text_offset = self.text_offset() + offset_in_parent;
            offset_in_parent += green_child.text_len();

            match green_child {
                NodeOrToken::Node(node) => Rc::new(RedNodeData {
                    parent: Some(Rc::clone(self)),
                    index_in_parent: idx,
                    text_offset,
                    green: node,
                })
                .into(),
                NodeOrToken::Token(token) => Rc::new(RedTokenData {
                    parent: Some(Rc::clone(self)),
                    index_in_parent: idx,
                    text_offset,
                    green: token,
                })
                .into(),
            }
        })
    }


    pub fn replace_child<'a>(self: &'a RedNode, idx: usize, new_child: GreenElement) -> RedNode {
        let new_green = self.green().replace_child(idx, new_child.into());
        self.replace_ourselves(new_green.into())
    }

    fn replace_ourselves<'a>(self: &'a RedNode, new_green: GreenNode) -> RedNode {
        match self.parent() {
            Some(parent) => parent.replace_child(self.index_in_parent, new_green.into()),
            None => RedNodeData::new_root(new_green)
        }
    }
}

impl Display for RedNodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.green(), f)
    }
}

impl RedTokenData {
    fn green(&self) -> &GreenToken {
        &self.green
    }

    pub fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }

    pub fn text_offset(&self) -> usize {
        self.text_offset
    }

    pub fn text_len(&self) -> usize {
        self.green().text_len()
    }

    pub fn parent(&self) -> Option<&RedNode> {
        self.parent.as_ref()
    }
}

impl Display for RedTokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.green(), f)
    }
}

impl RedElement {
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(n) => n.kind(),
            NodeOrToken::Token(t) => t.kind(),
        }
    }

    pub fn text_offset(&self) -> usize {
        match self {
            NodeOrToken::Node(n) => n.text_offset(),
            NodeOrToken::Token(t) => t.text_offset(),
        }
    }

    pub fn text_len(&self) -> usize {
        match self {
            NodeOrToken::Node(n) => n.text_len(),
            NodeOrToken::Token(t) => t.text_len(),
        }
    }

    pub fn parent(&self) -> Option<&RedNode> {
        match self {
            NodeOrToken::Node(n) => n.parent(),
            NodeOrToken::Token(t) => t.parent(),
        }
    }
}

impl Display for RedElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeOrToken::Node(it) => Display::fmt(it, f),
            NodeOrToken::Token(it) => Display::fmt(it, f),
        }
    }
}
