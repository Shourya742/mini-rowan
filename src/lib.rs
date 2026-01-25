/*

Requirements:

* full-fidelity (whitespace and comments are part of the tree)
* resilient & semi-structured (Can represent arbitrary invalid code)
* immutable value-type - the syntax tree contains the exactly the same information as underline string and that it doesn't have any identity, any position in the project and that you can compare in equality, like when you work with traditional compiler often wants to assign identity to trees, hey like this a particular expression which live's at particular offset, with a particular type  and a certain value, that works great in compilers because tree is static, for Ide's it doesn't work great because you want to do refactors. When you are implementing refactoring, you need to manipulate syntax tree's start ot duplicate functions or move expression from one node to another node you kinda lose this strict sense of identity and that's why you just want to treat syntax tree just a value.
* Cheaply updatable (for refactors & incremental reparse)
* conveniently updatable
* Easy to navigate (go from node to children, parent, siblings)

*/

// Idea 1: "dynamically typed" tree: a homogeneous tree on nodes.

mod ast;
mod green;
mod kinds;
mod ptr;
mod red;

use std::sync::Arc;

use crate::{
    green::{GreenNodeData, GreenTokenData},
    red::RedNodeData,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct SyntaxKind(u16);

#[derive(Clone, Debug)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

impl<N, T> NodeOrToken<N, T> {
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrToken::Node(n) => Some(n),
            NodeOrToken::Token(_) => None,
        }
    }

    pub fn into_token(self) -> Option<T> {
        match self {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(t) => Some(t),
        }
    }
}

#[test]
fn smoke() {
    let ws = Arc::new(GreenTokenData::new(kinds::WHITESPACE, " ".to_string()));
    let one = Arc::new(GreenTokenData::new(kinds::INT, "1".to_string()));
    let star = Arc::new(GreenTokenData::new(kinds::STAR, "*".to_string()));
    let two = Arc::new(GreenTokenData::new(kinds::INT, "2".to_string()));
    let three = Arc::new(GreenTokenData::new(kinds::INT, "3".to_string()));

    // 1 + 2
    let multiplication = Arc::new(GreenNodeData::new(
        kinds::BIN_EXPR,
        vec![
            one.into(),
            ws.clone().into(),
            star.into(),
            ws.clone().into(),
            two.into(),
        ],
    ));

    let plus = Arc::new(GreenTokenData::new(kinds::PLUS, "+".to_string()));

    // 1 * 2 + 1 * 2
    let addition = Arc::new(GreenNodeData::new(
        kinds::BIN_EXPR,
        vec![
            multiplication.clone().into(),
            ws.clone().into(),
            plus.into(),
            ws.into(),
            multiplication.into(),
        ],
    ));

    eprintln!("addition = {:?}", addition);
    eprintln!("{}", addition);

    let addition = RedNodeData::new_root(addition);
    let mul2 = addition.children().nth(4).unwrap().into_node().unwrap();
    let new_root = mul2.replace_child(0, three.into());

    eprintln!("{}", new_root);
}
