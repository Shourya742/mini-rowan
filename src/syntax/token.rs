use std::{fmt, marker::PhantomData};

use text_size::{TextLen, TextRange, TextSize};

use crate::{
    cursor,
    green::{token::GreenToken, trivia::GreenTrivia},
    syntax::{
        Language, SyntaxKind,
        element::{SyntaxElement, SyntaxElementKey},
        node::SyntaxNode,
        trivia::{
            SyntaxTrivia, SyntaxTriviaPiece, TriviaPiece, TriviaPieceKind, chain_trivia_pieces,
            trim_leading_trivia_pieces, trim_trailing_trivia_pieces,
        },
    },
    utility_types::{Direction, NodeOrToken},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
    raw: cursor::token::SyntaxToken,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxToken<L> {
    /// Create a new token detached from any tree
    ///
    /// This is mainly useful for creating a small number of individual tokens
    /// when mutating an existing tree, the bulk of the tokens in a given file
    /// should be created through the [crate::TreeBuilder] abstraction instead
    /// as it will efficiently cache and reuse the created tokens.
    pub fn new_detached<Leading, Trailing>(
        kind: L::Kind,
        text: &str,
        leading: Leading,
        trailing: Trailing,
    ) -> Self
    where
        Leading: IntoIterator<Item = TriviaPiece>,
        Leading::IntoIter: ExactSizeIterator,
        Trailing: IntoIterator<Item = TriviaPiece>,
        Trailing::IntoIter: ExactSizeIterator,
    {
        Self {
            raw: cursor::token::SyntaxToken::new_detached(GreenToken::with_trivia(
                kind.to_raw(),
                text,
                GreenTrivia::new(leading),
                GreenTrivia::new(trailing),
            )),
            _p: PhantomData,
        }
    }

    pub(super) fn green_token(&self) -> GreenToken {
        self.raw.green().to_owned()
    }

    pub fn key(&self) -> SyntaxElementKey {
        let (node_data, offset) = self.raw.key();
        SyntaxElementKey::new(node_data, offset)
    }

    pub fn kind(&self) -> L::Kind {
        L::Kind::from_raw(self.raw.kind())
    }

    pub fn text_range(&self) -> TextRange {
        self.raw.text_range()
    }

    pub fn text_trimmed_range(&self) -> TextRange {
        self.raw.text_trimmed_range()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    /// Returns the text of the token, including all trivia.
    pub fn text(&self) -> &str {
        self.raw.text()
    }

    /// Returns the text of a token, including all trivia as an owned value.
    pub fn token_text(&self) -> TokenText {
        self.raw.token_text()
    }

    pub fn token_text_trimmed(&self) -> TokenText {
        self.raw.token_text_trimmed()
    }

    /// Returns the text of the token, excluding all trivia
    pub fn text_trimmed(&self) -> &str {
        self.raw.text_trimmed()
    }

    pub fn parent(&self) -> Option<SyntaxNode<L>> {
        self.raw.parent().map(SyntaxNode::from)
    }

    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.raw.ancestors().map(SyntaxNode::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn sibling_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement<L>> + use<L> {
        self.raw
            .siblings_with_token(direction)
            .map(SyntaxElement::from)
    }

    /// Next token in the tree (i.e, not necessary a sibling)
    pub fn next_token(&self) -> Option<Self> {
        self.raw.next_token().map(Self::from)
    }

    /// Previous token in the tree (i.e, not necessary a sibling)
    pub fn prev_token(&self) -> Option<Self> {
        self.raw.prev_token().map(Self::from)
    }

    /// Return a new version of this token detached from its parent node
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn detach(self) -> Self {
        Self {
            raw: self.raw.detach(),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token its leading trivia replace with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_leading_trivia<'a, I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = (TriviaPieceKind, &'a str)>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut token_text = String::new();
        let trivia = trivia.into_iter().map(|(kind, text)| {
            token_text.push_str(text);
            TriviaPiece::new(kind, TextSize::of(text))
        });

        let leading = GreenTrivia::new(trivia);

        // Copy over token text and trailing trivia
        let leading_len = self.raw.green().text_len();
        token_text.push_str(&self.text()[usize::from(leading_len)..]);

        Self {
            raw: cursor::token::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &token_text,
                leading,
                self.green_token().trailing_trivia().clone(),
            )),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token with its leading trivia replaced with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_leading_trivia_pieces<I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = SyntaxTriviaPiece<L>>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut token_text = String::new();
        let trivia = trivia.into_iter().map(|piece| {
            token_text.push_str(piece.text());
            piece.into_raw_piece()
        });

        let leading = GreenTrivia::new(trivia);

        // Copy over token text and trailing trivia
        let leading_len = self.raw.green().leading_trivia().text_len();
        token_text.push_str(&self.text()[usize::from(leading_len)..]);

        Self {
            raw: cursor::token::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &token_text,
                leading,
                self.green_token().trailing_trivia().clone(),
            )),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token with its trailing trivia replaced with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_trailing_trivia<'a, I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = (TriviaPieceKind, &'a str)>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut token_text = String::new();

        // copy over leading trivia and token text
        let trailing_len = self.green_token().trailing_trivia().text_len();
        token_text.push_str(&self.text()[..usize::from(self.text().text_len() - trailing_len)]);

        let trivia = trivia.into_iter().map(|(kind, text)| {
            token_text.push_str(text);
            TriviaPiece::new(kind, TextSize::of(text))
        });

        let trailing = GreenTrivia::new(trivia);

        Self {
            raw: cursor::token::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &token_text,
                self.green_token().leading_trivia().clone(),
                trailing,
            )),
            _p: PhantomData,
        }
    }

    /// Return a new version of this token with its trailing trivia replaced with `trivia`
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn with_trailing_trivia_pieces<I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = SyntaxTriviaPiece<L>>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut token_text = String::new();

        let trailing_len = self.green_token().trailing_trivia().text_len();
        token_text.push_str(&self.text()[..usize::from(self.text().text_len() - trailing_len)]);

        let trivia = trivia.into_iter().map(|piece| {
            token_text.push_str(piece.text());
            piece.into_raw_piece()
        });

        let trailing = GreenTrivia::new(trivia);

        Self {
            raw: cursor::token::SyntaxToken::new_detached(GreenToken::with_trivia(
                self.kind().to_raw(),
                &token_text,
                self.green_token().leading_trivia().clone(),
                trailing,
            )),
            _p: PhantomData,
        }
    }

    /// Return a new version of this toke with `trivia` prepended to its leading trivia
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn prepend_trivia_pieces<I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = SyntaxTriviaPiece<L>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.with_leading_trivia_pieces(chain_trivia_pieces(
            trivia.into_iter(),
            self.leading_trivia().pieces(),
        ))
    }

    /// Return a new version of this token with `trivia` appended to its trailing trivia.
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn append_trivia_pieces<I>(&self, trivia: I) -> Self
    where
        I: IntoIterator<Item = SyntaxTriviaPiece<L>>,
        I::IntoIter: ExactSizeIterator,
    {
        self.with_trailing_trivia_pieces(chain_trivia_pieces(
            self.trailing_trivia().pieces(),
            trivia.into_iter(),
        ))
    }

    /// Return a new version of this token without leading newlines and whitespace
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn trim_leading_trivia(&self) -> Self {
        self.with_leading_trivia_pieces(trim_leading_trivia_pieces(self.leading_trivia().pieces()))
    }

    /// Return a new version of this token without trailing whitespaces
    #[must_use = "syntax elements are immutable, the result of update methods must be propagated to have any effect"]
    pub fn trim_trailing_trivia(&self) -> Self {
        self.with_trailing_trivia_pieces(trim_trailing_trivia_pieces(
            self.trailing_trivia().pieces(),
        ))
    }

    /// Return whitespaces that juxtapose the token until the first non-whitespace item.
    pub fn indentation_trivia_piece(
        &self,
    ) -> impl ExactSizeIterator<Item = SyntaxTriviaPiece<L>> + Clone + use<L> {
        let leading_trivia = self.leading_trivia().pieces();
        let skip_count = leading_trivia.len()
            - leading_trivia
                .rev()
                .position(|x| !x.is_whitespace())
                .map(|pos| pos + 1)
                .unwrap_or(0);
        self.leading_trivia().pieces().skip(skip_count)
    }

    /// Returns the token's leading trivia.
    ///
    /// Looking backward in the text, a token owns all of its preceding trivia up to and including the first newline character.
    pub fn leading_trivia(&self) -> SyntaxTrivia<L> {
        SyntaxTrivia::new(self.raw.leading_trivia())
    }

    /// Returns the token's trailing trivia.
    ///
    /// A token owns all of its following trivia up to, but not including, the next newline character.
    pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {
        SyntaxTrivia::new(self.raw.trailing_trivia())
    }

    /// Checks if the token has any trailing trivia that is a whitespace
    pub fn has_trailing_whitespace(&self) -> bool {
        self.trailing_trivia()
            .pieces()
            .any(|piece| piece.is_whitespace())
    }

    /// Checks if the token has any trailing trivia that is whitespace
    pub fn has_trailing_comments(&self) -> bool {
        self.trailing_trivia()
            .pieces()
            .any(|piece| piece.is_comments())
    }

    /// Checks if the current token has leading comments
    pub fn has_leading_comments(&self) -> bool {
        self.leading_trivia()
            .pieces()
            .any(|piece| piece.is_comments())
    }

    /// Checks if the token has any leading trivia that is a whitespace or a line break
    pub fn has_leading_whitespace_or_newline(&self) -> bool {
        self.leading_trivia()
            .pieces()
            .any(|piece| piece.is_whitespace() || piece.is_newline())
    }

    /// Checks if the current token has leading newline
    pub fn has_leading_newline(&self) -> bool {
        self.leading_trivia()
            .pieces()
            .any(|piece| piece.is_newline())
    }
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}&{:?} {:?}",
            self.kind(),
            self.text_range(),
            self.text_trimmed()
        )?;
        self.leading_trivia().fmt(f)?;
        write!(f, " ")?;
        self.trailing_trivia().fmt(f)
    }
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl<L: Language> From<SyntaxToken<L>> for cursor::token::SyntaxToken {
    fn from(value: SyntaxToken<L>) -> Self {
        value.raw
    }
}

impl<L: Language> From<cursor::token::SyntaxToken> for SyntaxToken<L> {
    fn from(value: cursor::token::SyntaxToken) -> Self {
        Self {
            raw: value,
            _p: PhantomData,
        }
    }
}

/// A syntax token that contains an offset
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SyntaxTokenWithOffset<L: Language> {
    pub token: SyntaxToken<L>,
    pub offset: TextSize,
}

impl<L: Language> SyntaxTokenWithOffset<L> {
    pub fn new(token: SyntaxToken<L>, offset: TextSize) -> Self {
        Self { token, offset }
    }

    /// Returns the trimmed text range, adjusted for base offset
    pub fn text_trimmed_range(&self) -> TextRange {
        let range = self.token.text_trimmed_range();
        TextRange::new(range.start() + self.offset, range.end() + self.offset)
    }

    /// Returns the text range including all trivia, adjusted for base offset
    pub fn text_range(&self) -> TextRange {
        let range = self.token.text_range();
        TextRange::new(range.start() + self.offset, range.end() + self.offset)
    }
}
