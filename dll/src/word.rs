//! Word encoding shared by reader.rs and eval.rs, mirrored from
//! asm/nucleus-win64.s / asm/nucleus.s (wsm-os-target::Tag, WORD_BITS=64,
//! TAG_BITS=3, PAYLOAD_BITS=61): Cons=0, Nil=1, True=2 (unused, see
//! nucleus.s), Fixnum=3, Symbol=4, Closure=5, Capability=6.
//!
//! String literals are NOT part of this encoding -- wsm-os-target::Tag has
//! no String variant, and no fixture seen so far says how one should be
//! represented. reader.rs therefore does not parse quoted strings yet;
//! that is an open question for my-lisp to confirm, not a guess made here.

pub const TAG_MASK: u64 = 7;
pub const TAG_CONS: u64 = 0;
pub const TAG_NIL: u64 = 1;
pub const TAG_FIXNUM: u64 = 3;
pub const TAG_SYMBOL: u64 = 4;

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
