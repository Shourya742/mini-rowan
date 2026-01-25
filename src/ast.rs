use std::sync::Arc;

use crate::{
    SyntaxKind,
    green::{GreenElement, GreenNode, GreenNodeData, GreenToken, GreenTokenData},
    kinds,
    red::{RedElement, RedNode, RedNodeData},
};

trait AstNode {
    fn cast(node: RedNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &RedNode;

    fn child_of_type<C: AstNode>(&self) -> Option<C> {
        self.syntax()
            .children()
            .filter_map(RedElement::into_node)
            .find_map(C::cast)
    }
}

struct Struct(RedNode);

impl AstNode for Struct {
    fn cast(node: RedNode) -> Option<Self> {
        if node.kind() == kinds::STRUCT {
            Some(Struct(node))
        } else {
            None
        }
    }

    fn syntax(&self) -> &RedNode {
        &self.0
    }
}

impl Struct {
    fn name(&self) -> Option<Name> {
        self.child_of_type()
    }

    fn fields<'a>(&'a self) -> impl Iterator<Item = Field> + 'a {
        self.syntax()
            .children()
            .filter_map(RedElement::into_node)
            .filter_map(Field::cast)
    }
}

struct Field(RedNode);

impl AstNode for Field {
    fn cast(node: RedNode) -> Option<Self> {
        if node.kind() == kinds::FIELD {
            Some(Field(node))
        } else {
            None
        }
    }

    fn syntax(&self) -> &RedNode {
        &self.0
    }
}

impl Field {
    fn name(&self) -> Option<Name> {
        self.child_of_type()
    }
}

struct Name(RedNode);

impl AstNode for Name {
    fn cast(node: RedNode) -> Option<Self> {
        if node.kind() == kinds::NAME {
            Some(Name(node))
        } else {
            None
        }
    }

    fn syntax(&self) -> &RedNode {
        &self.0
    }
}

impl Name {
    fn name(&self) -> Option<Name> {
        self.child_of_type()
    }
}

fn make_token(kind: SyntaxKind, text: &str) -> GreenToken {
    Arc::new(GreenTokenData::new(kind, text.to_string()))
}

fn make_node(kind: SyntaxKind, children: Vec<GreenElement>) -> GreenNode {
    Arc::new(GreenNodeData::new(kind, children))
}

fn make_whitespace(ws: &str) -> GreenToken {
    make_token(kinds::WHITESPACE, ws)
}

fn make_name(ws: &str) -> GreenNode {
    make_node(kinds::NAME, vec![make_token(kinds::WHITESPACE, ws).into()])
}

fn make_field(name: &str, ty: &str) -> GreenNode {
    Arc::new(GreenNodeData::new(
        kinds::FIELD,
        vec![
            make_whitespace("  ").into(),
            make_name(name).into(),
            make_token(kinds::COLON, ":").into(),
            make_whitespace(" ").into(),
            make_node(kinds::TYPE, vec![make_token(kinds::IDENT, ty).into()]).into(),
            make_token(kinds::COMMA, ",").into(),
            make_whitespace("\n").into(),
        ],
    ))
}

fn make_struct(name: &str, fields: Vec<GreenNode>) -> GreenNode {
    let mut children: Vec<GreenElement> = Vec::new();
    children.push(make_token(kinds::STRUCT_KW, "struct").into());
    children.push(make_whitespace(" ").into());
    children.push(make_name(name).into());
    children.push(make_whitespace(" ").into());
    children.push(make_token(kinds::L_CURLY, "{").into());
    children.push(make_whitespace("\n").into());
    children.extend(fields.into_iter().map(GreenElement::from));
    children.push(make_token(kinds::R_CURLY, "}").into());
    make_node(kinds::STRUCT, children)
}

#[test]
fn ask_smoke_test() {
    let strukt = make_struct(
        "Foo",
        vec![make_field("foo", "String"), make_field("bar", "IpAddr")],
    );
    let strukt = Struct::cast(RedNodeData::new_root(strukt)).unwrap();

    eprintln!("{}", strukt.name().unwrap().0);

    for (i, field) in strukt.fields().enumerate() {
        println!("field {}: {}", i, field.0)
    }
}
