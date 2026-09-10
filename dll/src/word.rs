//! Word encoding shared by reader.rs and eval.rs, mirrored from
//! asm/nucleus-win64.s / asm/nucleus.s (wsm-os-target::Tag, WORD_BITS=64,
//! TAG_BITS=3, PAYLOAD_BITS=61): Cons=0, Nil=1, True=2 (unused, see
//! nucleus.s), Fixnum=3, Symbol=4, Closure=5, Capability=6.
//!
//! Boxed values (currently just String): TENTATIVE, NOT an official part
//! of wsm-target-contract yet. my-lisp confirmed (2026-09-10, reading
//! crates/my-lisp/src/value.rs directly) strings are a distinct
//! `Value::String(Rc<str>)` variant, structurally identical to
//! `Value::Symbol(Rc<str>)` but semantically NOT interned (two equal
//! string literals are independent allocations -- my-lisp compares
//! strings structurally via `equal?`, not by identity via `eq?`, unlike
//! symbols).
//!
//! `TAG_BOXED = 7` (this crate's proposal, the only unused value
//! TAG_BITS=3 leaves: Cons=0/Nil=1/True=2/Fixnum=3/Symbol=4/Closure=5/
//! Capability=6) was originally going to be `TAG_STRING`, narrowly. cml
//! reviewed that (2026-09-10, checking both their c_backend.rs and
//! x86_freestanding.rs directly -- neither has any existing string-layout
//! of their own to conflict with) and recommended generalizing it instead:
//! rather than spend the ABI's last free 3-bit tag value on String
//! specifically, use it as a generic "boxed/ref" tag, with the boxed
//! object's own kind (String today, Vector/NumericBuffer later if ever
//! needed -- cml's own review of my-lisp's Value enum found those are the
//! only other variants shaped like this: variable-size blobs needing
//! offset+length in a side table, not an inline payload; Rational/Bool/
//! Macro don't need a primary tag at all by cml's analysis) carried as a
//! discriminant inside the table entry itself, not encoded into the tag
//! bits. This keeps TAG_BITS=3 viable for longer without a bigger,
//! harder-to-reverse ABI change (widening TAG_BITS itself, which every
//! consumer -- x86_freestanding.rs, asm/nucleus.s, any future FPGA-style
//! word format -- would have to move on together). Still asked cml/
//! wsm-target-contract to confirm the actual number, not decided
//! unilaterally -- do not treat TAG_BOXED=7 as canonical until that's
//! confirmed.

pub const TAG_BITS: u64 = 3;
pub const TAG_MASK: u64 = 7;
pub const TAG_CONS: u64 = 0;
pub const TAG_NIL: u64 = 1;
pub const TAG_FIXNUM: u64 = 3;
pub const TAG_SYMBOL: u64 = 4;
/// TENTATIVE -- see module doc above. Not yet reserved in
/// wsm-target-contract; do not treat as a stable cross-repo ABI value.
pub const TAG_BOXED: u64 = 7;

pub const WORD_NIL: u64 = TAG_NIL;

/// wsm_os_target::SYMBOL_ID_MAX, reserved in nucleus.s/nucleus-win64.s as
/// the sentinel id for canonical `t` -- see nucleus.s's own header comment
/// for why this is a sentinel, not a proven-unique id.
pub const SYM_T_ID: u64 = 0x1FFF_FFFF_FFFF_FFFF;
pub const SYM_T_WORD: u64 = (SYM_T_ID << 3) | TAG_SYMBOL;

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

/// Per-evaluator boxed-value table (TENTATIVE, see this module's header):
/// append-only, index-in-word (same pattern as `SymbolTable`), but
/// deliberately NOT deduplicating strings -- my-lisp confirmed two equal
/// string literals in one program are independent allocations, unlike
/// symbols.
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
        let index = self.values.len() as u64;
        self.values.push(BoxedValue::Str(value));
        (index << TAG_BITS) | TAG_BOXED
    }

    pub fn get_string(&self, word: u64) -> Option<&str> {
        if tag_of(word) != TAG_BOXED {
            return None;
        }
        let index = (word >> TAG_BITS) as usize;
        match self.values.get(index) {
            Some(BoxedValue::Str(s)) => Some(s.as_str()),
            None => None,
        }
    }
}
