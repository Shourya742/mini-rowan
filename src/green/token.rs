use std::{
    borrow::Borrow,
    fmt,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use text_size::TextSize;

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    green::{RawSyntaxKind, trivia::GreenTrivia},
};

#[derive(PartialEq, Eq, Hash)]
struct GreenTokenHead {
    kind: RawSyntaxKind,
    leading: GreenTrivia,
    trailing: GreenTrivia,
    #[cfg(feature = "countme")]
    _c: countme::Count<GreenToken>,
}

#[cfg(feature = "countme")]
pub(crate) fn has_live() -> bool {
    countme::get::<GreenToken>().live > 0
}

type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

#[repr(transparent)]
pub(crate) struct GreenTokenData {
    data: ReprThin,
}

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            let repr: &Repr = &self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenTokenData>(repr)
        }
    }
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        unsafe {
            let green = GreenToken::from_raw(ptr::NonNull::from(self));
            let green = ManuallyDrop::new(green);
            GreenToken::clone(&green)
        }
    }
}

impl Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("text", &self.text())
            .field("leading", &self.leading_trivia())
            .field("trailing", &self.trailing_trivia())
            .finish()
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl GreenTokenData {
    /// kind of this token
    #[inline]
    pub fn kind(&self) -> RawSyntaxKind {
        self.data.header.kind
    }

    /// Whole text of this Token, including all trivia
    #[inline]
    pub fn text(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.data.slice()) }
    }

    pub(crate) fn leading_trailing_total_len(&self) -> (TextSize, TextSize, TextSize) {
        let leading_len = self.data.header.leading.text_len();
        let trailing_len = self.data.header.trailing.text_len();
        let total_len = self.data.slice().len() as u32;
        (leading_len, trailing_len, total_len.into())
    }

    /// Text of this Token, excluding all trivia.
    #[inline]
    pub fn text_trimmed(&self) -> &str {
        let (leading_len, trailing_len, total_len) = self.leading_trailing_total_len();
        let start: usize = leading_len.into();
        let end: usize = (total_len - trailing_len).into();
        let text = unsafe { std::str::from_utf8_unchecked(self.data.slice()) };
        &text[start..end]
    }

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn text_len(&self) -> TextSize {
        TextSize::of(self.text())
    }

    #[inline]
    pub fn leading_trivia(&self) -> &GreenTrivia {
        &self.data.header.leading
    }

    #[inline]
    pub fn trailing_trivia(&self) -> &GreenTrivia {
        &self.data.header.trailing
    }
}

impl GreenToken {
    pub fn new_raw(kind: RawSyntaxKind, text: &str) -> Self {
        let leading = GreenTrivia::empty();
        let trailing = leading.clone();

        Self::with_trivia(kind, text, leading, trailing)
    }

    #[inline]
    pub fn with_trivia(
        kind: RawSyntaxKind,
        text: &str,
        leading: GreenTrivia,
        trailing: GreenTrivia,
    ) -> Self {
        let head = GreenTokenHead {
            kind,
            leading,
            trailing,
            #[cfg(feature = "countme")]
            _c: countme::Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, text.bytes());
        Self { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(self) -> ptr::NonNull<GreenTokenData> {
        Arc::from_thin(self.ptr).into_raw().cast()
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> Self {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenTokenHead, u8>>(arc)
        };
        Self { ptr: arc }
    }
}
