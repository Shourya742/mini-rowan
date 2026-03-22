use std::marker::PhantomData;

use text_size::TextSize;

use crate::syntax::Language;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TriviaPieceKind {
    /// A line break (`\n`, `\r`, `\r\n`, ...)
    Newline,
    /// Any whitespace character
    Whitespace,
    /// Comment that does not contain any line breaks
    SingleLineComment,
    /// Comment that contains at least one line break
    MultiLineComment,
    /// Token that the parser skipped for some reason.
    Skipped,
}

impl TriviaPieceKind {
    pub const fn is_newline(&self) -> bool {
        matches!(self, Self::Newline)
    }

    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    pub const fn is_single_line_comment(&self) -> bool {
        matches!(self, Self::SingleLineComment)
    }

    pub const fn is_multiline_comment(&self) -> bool {
        matches!(self, Self::MultiLineComment)
    }

    pub const fn is_comment(&self) -> bool {
        self.is_single_line_comment() || self.is_multiline_comment()
    }

    pub const fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TriviaPiece {
    pub(crate) kind: TriviaPieceKind,
    pub(crate) length: TextSize,
}

impl TriviaPiece {
    /// Creates a new whitespace trivia piece with the given length
    pub fn whitespace<L: Into<TextSize>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Whitespace, len)
    }

    /// Creates a new newline trivia piece with the given text length
    pub fn newline<L: Into<TextSize>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Newline, len)
    }

    /// Creates a new comment trivia piece that does not contain any line breaks.
    pub fn single_line_comment<L: Into<TextSize>>(len: L) -> Self {
        Self::new(TriviaPieceKind::SingleLineComment, len)
    }

    /// Creates a new comment trivia piece that contains at least one line breaks.
    /// For example, a JavaScript `/* ... */` comment that spawns at least two lines (contains at least one line break character).
    pub fn multi_line_comment<L: Into<TextSize>>(len: L) -> Self {
        Self::new(TriviaPieceKind::MultiLineComment, len)
    }

    pub fn new<L: Into<TextSize>>(kind: TriviaPieceKind, length: L) -> Self {
        Self {
            kind,
            length: length.into(),
        }
    }

    /// Returns the trivia's length
    pub fn text_len(&self) -> TextSize {
        self.length
    }

    /// Returns the trivia's kind
    pub fn kind(&self) -> TriviaPieceKind {
        self.kind
    }
}

/// [SyntaxTriviaPiece] gives access  to the most granular information about the trivia
/// that was specified by the lexer at the token creation time.
///
/// For example:
///
/// ```no_test
/// builder.token_with_trivia(RawSyntaxKind(1), "\n\t /**/let \t\t", &[TriviaPiece::whitespace(3), TriviaPiece::single_line_comment(4)], &[TriviaPiece::whitespace(3)])
/// ```
/// This token has two pieces in the leading trivia, and one piece at the trailing trivia. Each
/// piece is defined by the [TriviaPiece]; its content is irrelevant.
#[derive(Clone)]
pub struct SyntaxTriviaPiece<L: Language> {
    // raw: cursor::SyntaxTrivia,
    // Absolute offset from the beginning of the file.
    offset: TextSize,
    trivia: TriviaPiece,
    _p: PhantomData<L>,
}
