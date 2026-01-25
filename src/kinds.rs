use crate::SyntaxKind;

// Nodes
pub(crate) const FN: SyntaxKind = SyntaxKind(1);
pub(crate) const FN_KW: SyntaxKind = SyntaxKind(2);
pub(crate) const PARAM_LIST: SyntaxKind = SyntaxKind(5);
pub(crate) const BIN_EXPR: SyntaxKind = SyntaxKind(4);

// Tokens
pub(crate) const WHITESPACE: SyntaxKind = SyntaxKind(099);
pub(crate) const IDENT: SyntaxKind = SyntaxKind(100);
pub(crate) const NAME: SyntaxKind = SyntaxKind(101);
pub(crate) const INT: SyntaxKind = SyntaxKind(102);
pub(crate) const PLUS: SyntaxKind = SyntaxKind(103);
pub(crate) const STAR: SyntaxKind = SyntaxKind(104);
