//! Word encoding shared by reader.rs and eval.rs, mirrored from
//! asm/nucleus-win64.s / asm/nucleus.s (wsm-os-target::Tag, WORD_BITS=64,
//! TAG_BITS=3, PAYLOAD_BITS=61): Cons=0, Nil=1, True=2 (unused, see
//! nucleus.s), Fixnum=3, Symbol=4, Closure=5, Capability=6.
//!
//! Boxed values (currently just String): RATIFIED 2026-09-10 as
//! `Tag::Boxed = 7` in `wsm-target-contract` (contract v3, commit
//! `bb6e119`, `docs/migration-2026-09-10-boxed-tag.md`) -- no longer
//! this crate's own tentative proposal. `TAG_BOXED` below is now a
//! straight re-export of `wsm_os_target::Tag::Boxed`, not a local
//! constant duplicating the number by hand (per that migration note's
//! own instruction to this crate, and wsm-my-lisp#1's acceptance
//! criterion against uncoordinated local copies of ABI constants).
//!
//! History, for context: this was originally proposed narrowly as
//! `TAG_STRING`. my-lisp confirmed (2026-09-10, reading
//! crates/my-lisp/src/value.rs directly) strings are a distinct
//! `Value::String(Rc<str>)` variant, structurally identical to
//! `Value::Symbol(Rc<str>)` but semantically NOT interned (two equal
//! string literals are independent allocations -- my-lisp compares
//! strings structurally via `equal?`, not by identity via `eq?`, unlike
//! symbols). cml then reviewed the `TAG_STRING` proposal (checking both
//! their c_backend.rs and x86_freestanding.rs directly -- neither had an
//! existing string-layout of their own to conflict with) and recommended
//! generalizing it instead: rather than spend the ABI's last free 3-bit
//! tag value on String specifically, use it as a generic "boxed/ref" tag,
//! with the boxed object's own kind (String today, Vector/NumericBuffer
//! later if ever needed) carried as a discriminant inside the table entry
//! itself, not encoded into the tag bits. `wsm-target-contract` ratified
//! exactly that shape, and additionally fixed the scope as **session-local**
//! (a `Boxed` handle indexes a runtime-owned table created fresh per host
//! session -- it does not survive across sessions/processes, unlike
//! `Symbol`/`Closure` which are image-local) -- matching this crate's own
//! `BoxedTable` design already (it lives in `ffi.rs`'s per-session
//! `Session`, not anywhere image-local).

pub const TAG_BITS: u64 = wsm_os_target::TAG_BITS as u64;
pub const TAG_MASK: u64 = wsm_os_target::TAG_MASK;
pub const TAG_CONS: u64 = wsm_os_target::Tag::Cons as u64;
pub const TAG_NIL: u64 = wsm_os_target::Tag::Nil as u64;
pub const TAG_FIXNUM: u64 = wsm_os_target::Tag::Fixnum as u64;
pub const TAG_SYMBOL: u64 = wsm_os_target::Tag::Symbol as u64;
/// Ratified -- see module doc above. `wsm_os_target::Tag::Boxed as u64`,
/// not a hand-copied number.
pub const TAG_BOXED: u64 = wsm_os_target::Tag::Boxed as u64;

pub const WORD_NIL: u64 = TAG_NIL;

/// `wsm_os_target::SYMBOL_ID_MAX`, reserved in nucleus.s/nucleus-win64.s
/// as the sentinel id for canonical `t` -- see nucleus.s's own header
/// comment for why this is a sentinel, not a proven-unique id.
pub const SYM_T_ID: u64 = wsm_os_target::SYMBOL_ID_MAX;
/// `wsm_os_target::CANONICAL_T` -- same value, now imported rather than
/// hand-recomputed from `SYM_T_ID`/`TAG_SYMBOL`.
pub const SYM_T_WORD: u64 = wsm_os_target::CANONICAL_T;

pub fn tag_of(word: u64) -> u64 {
    word & TAG_MASK
}

pub fn is_truthy(word: u64) -> bool {
    // Every non-Nil word is truthy, including Fixnum 0 (encoded word 3,
    // never TAG_NIL's word 1) -- confirmed against my-lisp's own
    // conformance.my fact: `(cond (0 (quote truthy)) (t (quote wrong)))`
    // => `truthy`. This falls straight out of the tag encoding: 0's word
    // is `(0 << 3) | TAG_FIXNUM` = 3, which never collides with
    // WORD_NIL = 1.
    word != WORD_NIL
}

pub fn encode_fixnum(value: i64) -> u64 {
    ((value << 3) | TAG_FIXNUM as i64) as u64
}

pub fn decode_fixnum(word: u64) -> i64 {
    debug_assert_eq!(tag_of(word), TAG_FIXNUM);
    (word as i64) >> 3
}

/// Per-evaluator symbol table: interns names to sequential ids starting
/// at 1, matching cml's own convention (cml/src/x86_freestanding.rs
/// assigns each compiled program's quoted symbols sequential ids from a
/// sorted BTreeSet, starting at 1) -- noted in nucleus.s's header as
/// "image-local-interned", not a shared global table. `t` is special-cased
/// to SYM_T_WORD and never consumes a sequential id.
#[derive(Default)]
pub struct SymbolTable {
    name_to_id: std::collections::HashMap<String, u64>,
    next_id: u64,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { name_to_id: std::collections::HashMap::new(), next_id: 1 }
    }

    pub fn intern(&mut self, name: &str) -> u64 {
        if name == "t" {
            return SYM_T_WORD;
        }
        if let Some(&id) = self.name_to_id.get(name) {
            return (id << 3) | TAG_SYMBOL;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.name_to_id.insert(name.to_string(), id);
        (id << 3) | TAG_SYMBOL
    }

    /// Reverse lookup, for error messages and printing. O(n) -- fine for
    /// the small symbol counts a one-shot console command has; revisit if
    /// that stops being true.
    pub fn name_of(&self, word: u64) -> Option<&str> {
        if word == SYM_T_WORD {
            return Some("t");
        }
        let id = word >> 3;
        self.name_to_id
            .iter()
            .find(|&(_, &v)| v == id)
            .map(|(k, _)| k.as_str())
    }
}

/// A boxed value's kind, carried inside the table entry itself (cml's
/// recommendation -- see this module's header) rather than in the tag
/// bits. Only `Str` exists today; this enum is exactly where a future
/// `Vector`/`NumericBuffer` variant would be added, per cml's review,
/// without needing a new primary tag or touching word encoding at all.
pub enum BoxedValue {
    Str(String),
}

/// Per-evaluator boxed-value table, ratified shape (see this module's
/// header): append-only, session-local, handle-in-word via
/// `wsm_os_target::encode_boxed`/`decode_boxed` (same pattern as
/// `SymbolTable`'s own id-in-word, but through the canonical helpers now
/// rather than a hand-rolled shift), deliberately NOT deduplicating
/// strings -- my-lisp confirmed two equal string literals in one program
/// are independent allocations, unlike symbols.
///
/// `encode_boxed` rejects a zero handle (reserved, per
/// `wsm_os_target::BOXED_HANDLE_MAX`'s own doc), so entries are handled
/// 1-based (`handle = index + 1`) rather than `SymbolTable`'s 0-based-id
/// convention -- the two tables intentionally don't share a numbering
/// scheme, only the general "small int in the word, real data in a side
/// table" shape.
#[derive(Default)]
pub struct BoxedTable {
    values: Vec<BoxedValue>,
}

impl BoxedTable {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Always allocates a new entry, even for a value equal to one
    /// already present -- no lookup/dedup, unlike `SymbolTable::intern`.
    pub fn add_string(&mut self, value: String) -> u64 {
        let handle = self.values.len() as u64 + 1;
        self.values.push(BoxedValue::Str(value));
        wsm_os_target::encode_boxed(handle).expect("handle is non-zero and within BOXED_HANDLE_MAX by construction")
    }

    pub fn get_string(&self, word: u64) -> Option<&str> {
        let handle = wsm_os_target::decode_boxed(word)?;
        let index = (handle - 1) as usize;
        match self.values.get(index) {
            Some(BoxedValue::Str(s)) => Some(s.as_str()),
            None => None,
        }
    }
}
