use countme::Count;

use crate::green::SyntaxKind;

struct GreenTokenHead {
    kind: SyntaxKind,
    _c: Count<GreenToken>,
}
