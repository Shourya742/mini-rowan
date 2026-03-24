use std::{
    hash::{BuildHasherDefault, Hash},
    marker::PhantomData,
    ops::Not,
    ptr::NonNull,
};

use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;
use text_size::TextSize;

use crate::{
    green::{
        RawSyntaxKind,
        element::{GreenElement, GreenElementRef},
        node::{GreenNode, GreenNodeData},
        token::{GreenToken, GreenTokenData},
        trivia::{GreenTrivia, GreenTriviaData},
    },
    syntax::trivia::{TriviaPiece, TriviaPieceKind},
    utility_types::NodeOrToken,
};

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// Trait implemented for types that can be turned into a raw pointer, and
/// reconstructed back from it. Used by (GenerationalPointer) internally.
trait IntoRawPointer {
    type Pointee;
    fn into_raw(self) -> *mut Self::Pointee;
    unsafe fn from_raw(ptr: *mut Self::Pointee) -> Self;
}

impl IntoRawPointer for GreenToken {
    type Pointee = GreenTokenData;

    fn into_raw(self) -> *mut Self::Pointee {
        Self::into_raw(self).as_ptr()
    }

    unsafe fn from_raw(ptr: *mut Self::Pointee) -> Self {
        unsafe { Self::from_raw(NonNull::new(ptr).unwrap()) }
    }
}

impl IntoRawPointer for GreenTrivia {
    type Pointee = GreenTriviaData;

    fn into_raw(self) -> *mut Self::Pointee {
        Self::into_raw(self)
    }

    unsafe fn from_raw(ptr: *mut Self::Pointee) -> Self {
        unsafe { Self::from_raw(ptr) }
    }
}

impl IntoRawPointer for GreenNode {
    type Pointee = GreenNodeData;

    fn into_raw(self) -> *mut Self::Pointee {
        Self::into_raw(self).as_ptr()
    }

    unsafe fn from_raw(ptr: *mut Self::Pointee) -> Self {
        unsafe { Self::from_raw(NonNull::new(ptr).unwrap()) }
    }
}

/// Represents a "generation" in the garbage collection scheme of the node
/// cache. For our purpose we only need to track two generations at most (the
/// previous and next generation) so this is represented as an enum with two
/// variants
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Generation {
    #[default]
    A = 0,
    B = 1,
}

impl Not for Generation {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

use std::hash::Hasher;
fn token_hash_of(kind: RawSyntaxKind, text: &str) -> u64 {
    let mut h = FxHasher::default();
    kind.hash(&mut h);
    text.hash(&mut h);
    h.finish()
}

fn token_hash(token: &GreenTokenData) -> u64 {
    token_hash_of(token.kind(), token.text())
}

fn element_id(element: GreenElementRef<'_>) -> *const () {
    match element {
        NodeOrToken::Node(it) => it as *const GreenNodeData as *const (),
        NodeOrToken::Token(it) => it as *const GreenTokenData as *const (),
    }
}

/// Internal representation for a green pointer and a generation index in the
/// cache, packed into a single `usize`. This relies on the fact that "green
/// elements" (tokens, nodes, and trivia) have memory alignment constraints that
/// exceed a single byte (and thus the lower bits of the pointer will always be
/// zero), while the generation index only needs a single bit of storage
struct GenerationalPointer<T: IntoRawPointer> {
    data: usize,
    _ty: PhantomData<T>,
}

impl<T: IntoRawPointer> GenerationalPointer<T> {
    fn new(value: T, generation: Generation) -> Self {
        let ptr = value.into_raw();
        let mut data = ptr as usize;
        debug_assert!(data & 1 == 0);
        data |= generation as usize;
        Self {
            data,
            _ty: PhantomData,
        }
    }

    fn value(&self) -> &T::Pointee {
        // SAFETY: This clears the least significant bit from 'data'. This bit
        // should have been set to zero in the original pointer due to the
        // alignment requirements of the underlying data (this is checked by an
        // assertion on debug builds), so this essentially extracts the pointer
        // value from the bit field. Said point is safe to dereference at this point
        // since we're holding a valid reference to `self` which guarantees
        // `Drop` has not been called and the memory associated with the
        // pointer has not been released yet.
        let data = self.data & !1;
        let ptr = data as *const T::Pointee;
        unsafe { &*ptr }
    }

    fn generation(&self) -> Generation {
        match self.data & 1 {
            0 => Generation::A,
            1 => Generation::B,
            // SAFETY: The `& 1` operation above ensures only the least
            // significant bit can be set
            _ => unreachable!(),
        }
    }

    fn set_generation(&mut self, generation: Generation) {
        let data = self.data & !1;
        self.data = data | generation as usize;
    }
}

impl<T: IntoRawPointer> std::fmt::Debug for GenerationalPointer<T>
where
    T::Pointee: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("GenerationalPointer")
            .field("value", self.value())
            .field("generation", &self.generation())
            .finish()
    }
}

impl<T: IntoRawPointer> Drop for GenerationalPointer<T> {
    fn drop(&mut self) {
        let ptr = self.value() as *const _ as *mut _;
        let value = unsafe { T::from_raw(ptr) };
        drop(value);
    }
}

/// A token stored in the `NodeCache`.
/// Does intentionally not implement `Hash` to have compile-time guarantees that the `NodeCache`
/// use the correct hash.
#[derive(Debug)]
struct CachedToken(GenerationalPointer<GreenToken>);

/// A node stored in the `NodeCache`. It stores a pre-computed hash
/// because re-computing the hash requires traversing the whole sub-tree.
/// The hash also differs from the `GreenNode` hash implementation as it
/// only hashes occupied slots and exclude empty slots
///
/// Does intentionally not implement `Hash` to have compile time guarantees that the `NodeCache`
/// uses the correct hash.
#[derive(Debug)]
struct CacheNode {
    node: GenerationalPointer<GreenNode>,
    // store the hash as it's expensive to re-compute
    // involves re-computing the hash of the whole sub-tree
    hash: u64,
}

/// A cached [GreenTrivia].
/// Deliberately doesn't implement `Hash` to make sure all usages
/// go through the custom `FxHasher`
#[derive(Debug)]
struct CachedTrivia(GenerationalPointer<GreenTrivia>);

#[derive(Debug)]
struct TriviaCache {
    /// Generic cache for trivia
    cache: HashMap<CachedTrivia, ()>,
    /// Cached single whitespace trivia
    whitespace: GreenTrivia,
}

impl Default for TriviaCache {
    fn default() -> Self {
        Self {
            cache: Default::default(),
            whitespace: GreenTrivia::new([TriviaPiece::whitespace(1)]),
        }
    }
}

impl TriviaCache {
    fn trivia_hash_of(pieces: &[TriviaPiece]) -> u64 {
        let mut h = FxHasher::default();
        pieces.len().hash(&mut h);

        for piece in pieces {
            piece.hash(&mut h);
        }
        h.finish()
    }

    /// Tries to retrieve a [GreenTrivia] with the given pieces from the cache or creates a new one and caches
    /// it for further calls.
    fn get(&mut self, generation: Generation, pieces: &[TriviaPiece]) -> GreenTrivia {
        match pieces {
            [] => GreenTrivia::empty(),
            [
                TriviaPiece {
                    kind: TriviaPieceKind::Whitespace,
                    length,
                },
            ] if *length == TextSize::from(1) => self.whitespace.clone(),
            _ => {
                let hash = Self::trivia_hash_of(pieces);

                let entry = self
                    .cache
                    .raw_entry_mut()
                    .from_hash(hash, |trivia| trivia.0.value().pieces() == pieces);

                match entry {
                    RawEntryMut::Occupied(mut entry) => {
                        entry.key_mut().0.set_generation(generation);
                        entry.key().0.value().to_owned()
                    }
                    RawEntryMut::Vacant(entry) => {
                        let trivia = GreenTrivia::new(pieces.iter().copied());
                        entry.insert_with_hasher(
                            hash,
                            CachedTrivia(GenerationalPointer::new(trivia.clone(), generation)),
                            (),
                            |cached| Self::trivia_hash_of(cached.0.value().pieces()),
                        );
                        trivia
                    }
                }
            }
        }
    }
}

/// Interner for GreenTokens and GreenNodes
/// The impl is a bit tricky. As usual when writing interners, we want to
/// store all values in one HashSet.
///
/// However, hashing trees is fun: hash of the tree is recursively defined. We
/// maintain an invariant -- if the tree is interned, then all of its children
/// are interned as well.
///
/// That means that computing the hash naively is wasteful -- we just *know*
/// hashes of children, and we can re-use those.
///
/// So here we use *raw* API of hashbrown and provide the hashes manually,
/// instead of going via a `Hash` impl. Our manual `Hash` and the
/// `#[derive(Hash)]` are actually different! At some point we had a fun bug,
/// where we accidentally mixed the two hashes, which made the cache much less
/// efficient.
///
/// To fix that, we additionally wrap the data in `Cached*` wrappers, to make sure
/// we don't accidently use the wrong hash~
#[derive(Default, Debug)]
pub struct NodeCache {
    nodes: HashMap<CacheNode, ()>,
    tokens: HashMap<CacheNode, ()>,
    trivia: TriviaCache,
    generation: Generation,
}
