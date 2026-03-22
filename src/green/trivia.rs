use core::fmt;
use std::mem;

use text_size::TextSize;

use crate::{
    arc::{Arc, HeaderSlice, ThinArc},
    syntax::trivia::TriviaPiece,
};

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct GreenTriviaHead {
    #[cfg(feature = "countme")]
    _c: countme::Count<GreenTrivia>,
}

#[cfg(feature = "countme")]
pub(crate) fn has_live() -> bool {
    countme::get::<GreenTrivia>().live > 0
}

type ReprThin = HeaderSlice<GreenTriviaHead, [TriviaPiece; 0]>;

pub(crate) struct GreenTriviaData {
    data: ReprThin,
}


impl PartialEq for GreenTriviaData {
    fn eq(&self, other: &Self) -> bool {
        self.pieces() == other.pieces()
    }
}

impl fmt::Debug for GreenTriviaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.pieces().iter()).finish()
    }
}

impl GreenTriviaData {
    #[expect(unused)]
    #[inline]
    pub fn header(&self) -> &GreenTriviaHead {
        &self.data.header
    }

    #[inline]
    pub fn pieces(&self) -> &[TriviaPiece] {
        self.data.slice()
    }
}

/// List of trivia. Used to store either the leading or trailing trivia of a token.
/// The identity of a trivia is defined by the kinds and lengths of its items but not
/// the text of an individual piece. That means, the `\r` and `\n` can both be represented
/// by the same trivia, a trivia with a single `LINEBREAK` piece with the length 1.
/// This is safe because the text is stored on the token to which the trivia belongs and
/// `a\n` and `a\r` never resolve to the same tokens. Thus, they only share the trivia but are
/// other tow different tokens
#[derive(Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    ptr: Option<ThinArc<GreenTriviaHead, TriviaPiece>>,
}

impl fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self.pieces(), f)
    }
}

impl GreenTrivia {
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = TriviaPiece>,
        I::IntoIter: ExactSizeIterator,
    {
        let data = ThinArc::from_header_and_iter(
            GreenTriviaHead {
                #[cfg(feature = "countme")]
                _c: countme::Count::new(),
            },
            pieces.into_iter(),
        );
        Self { ptr: Some(data) }
    }

    /// Creates an empty trivia
    pub fn empty() -> Self {
        Self { ptr: None }
    }

    /// Returns the total length of all pieces
    pub fn text_len(&self) -> TextSize {
        let mut len = TextSize::default();

        for piece in self.pieces() {
            len += piece.length
        }

        len
    }

    /// Returns the pieces count
    pub fn len(&self) -> usize {
        match &self.ptr {
            None => 0,
            Some(ptr) => ptr.len(),
        }
    }

    pub fn pieces(&self) -> &[TriviaPiece] {
        match &self.ptr {
            None => &[],
            Some(ptr) => ptr.slice(),
        }
    }

    pub fn get_piece(&self, index: usize) -> Option<&TriviaPiece> {
        self.pieces().get(index)
    }

    pub(crate) fn into_raw(self) -> *mut GreenTriviaData {
        self.ptr.map_or_else(std::ptr::null_mut, |ptr| {
            Arc::from_thin(ptr).into_raw().cast().as_ptr()
        })
    }

    pub(crate) unsafe fn from_raw(ptr: *mut GreenTriviaData) -> Self {
        unsafe {
            if let Some(ptr) = ptr.as_ref() {
                let arc = Arc::from_raw(&ptr.data as *const ReprThin);
                let arc =
                    mem::transmute::<Arc<ReprThin>, ThinArc<GreenTriviaHead, TriviaPiece>>(arc);
                Self { ptr: Some(arc) }
            } else {
                Self { ptr: None }
            }
        }
    }
}
