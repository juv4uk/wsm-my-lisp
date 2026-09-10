# Optimization proposals for my-lisp — based on my-lisp's own measured findings

Status: **proposals only, no code written**, per the owner's explicit
constraint (simple ideas, not architecture rebuilds; addressed to
my-lisp for them to evaluate/decide, not implemented here or by wsm-my-lisp).

These are NOT based on wsm-my-lisp's own architecture (this crate's
`dll/` is a structurally different, much smaller interpreter — no
rationals, a flat single-scope `Env`, not a linked-frame chain — so its
own benchmarks don't transfer). Instead, these read directly from
my-lisp's own real measurements, committed same-day
(`docs/benchmarks.md` §5, commit `a5d726b`): rational-vs-fixnum tax
~14-18×, symbol-lookup-deep-vs-shallow ~3-3.5×, and a real stack
overflow found in `warm/vector-fill-500` on Windows.

## 1. Lexical address caching for repeated variable references (addresses §5's ~3-3.5× lookup-depth finding)

**The measured problem**: `Environment` is a linked-parent-frame chain;
looking up a binding 4 frames away costs ~3-3.5× a same-frame lookup,
because the interpreter re-walks the chain (presumably a string/symbol
key comparison per frame) on every single reference, every iteration of
a loop, even when the reference always resolves to the exact same frame
and slot every time.

**Proposed simple fix**: a well-established technique (used in Scheme/
Lisp interpreters for decades, not a novel idea) — cache the resolved
location (frame-depth, slot-index) the first time a specific *reference
site* (not the whole program — just that one occurrence of `far-away`
in the source) is looked up, and reuse the cached location on every
subsequent evaluation of that same site instead of re-walking the
string-keyed chain. In a hot loop like the benchmark's own
`symbol-lookup-deep` case, this turns an O(depth) walk into an O(1)
cache hit after the first iteration.

**Why this counts as "simple," not a rebuild**: it doesn't change
`Environment`'s own representation (still a linked chain, still correct
for the general case) — it only adds a small memoization at each AST
reference node (or an equivalent call-site cache keyed by source
location), populated lazily and invalidated the same way any other
per-node cache would be if the surrounding lexical structure ever
changes (e.g. inside a macro-expansion path, if that's a concern this
implementation would need to check).

## 2. Fast-path / deferred normalization for the exact-rational tax (addresses §5's ~14-18× finding)

**The measured problem**: every rational arithmetic step pays a full
normalize/gcd cost, ~14-18× a same-shape integer (Fixnum) operation.
This is NOT proposed as something to eliminate — the owner's own
standing decision ("РАЦІОНАЛЬНІ ЧИСЛА ЦЕ НАША ФІШКА") makes exactness
non-negotiable, so any proposal that trades correctness for speed is out
of scope by design, not just unwise.

**Proposed simple fix, in two independent parts, either alone is a small change**:

- **(a) Integer fast path**: when both the numerator and denominator of
  a `Rational` operation result already reduce to an integer (denominator
  == 1) and both fit a machine word, skip the general bignum gcd/reduce
  path entirely and use plain integer arithmetic, promoting back to the
  full `Rational` representation only if a result later needs a genuine
  fraction. This costs nothing in correctness (still exact, still the
  same value) and directly targets the common case where a chain of
  rational operations happens to stay integer-valued for many steps
  before ever producing a real fraction -- the fixnum-loop-100 control in
  §5 is exactly this case, already ~14-18× cheaper, so promoting the
  common integer sub-case to that same cost path where it already
  applies is a direct, targeted win, not a guess.
- **(b) Deferred gcd/normalization**: rather than reducing to lowest
  terms after every single intermediate operation in a chain (e.g. a
  loop accumulator), defer the gcd/reduce step until the value is
  actually observed (printed, compared, or otherwise leaves the
  arithmetic chain) -- a known technique in some computer-algebra
  systems for exactly this reason. This one needs more care than (a):
  un-reduced numerators/denominators can grow faster between reductions,
  so it trades some intermediate memory/bignum-width for fewer gcd calls
  -- worth measuring against §5's own `warm/rational-chain-100` case
  specifically before deciding it's a net win, not assumed here to be
  one unconditionally.

## 3. Explicit larger stack for the evaluator thread (addresses §5's found stack overflow, not a fix of the underlying non-tail-recursion)

**The measured problem**: `warm/vector-fill-500` reliably stack-overflows
on Windows (confirmed against the unmodified file, so not something the
new benchmarks introduced) -- `eval_program`'s handling of that shape has
a real non-tail-recursive Rust call path, and Windows' default main-thread
stack (1 MiB) is much smaller than a typical Linux default (8 MiB),
which is why the same program shape likely doesn't crash there.

**Proposed simple mitigation, NOT a fix of the underlying recursion
shape**: run the evaluator on a `std::thread::Builder::new().stack_size(N)`
spawned thread with an explicitly larger stack (matching or exceeding the
Linux default, e.g. 8 MiB or more) instead of relying on the OS-default
main-thread stack size, wherever my-lisp's entry points (CLI, benchmark
harness, any future embedding) start evaluation. This is a small,
localized, well-understood mitigation (not a redesign of `eval_program`'s
own recursion shape) -- it raises the ceiling rather than removing the
underlying non-tail-recursive path, which my-lisp's own commit already
correctly flagged as a separate, larger task out of scope for a
benchmarking pass.

## Not proposed here

- Nothing about my-lisp's core semantics, exactness guarantees, or
  language surface -- these are pure implementation-detail proposals
  that don't change what any program observes.
- No claim that any of these are "the" fix, or estimated to close the
  full 14-18×/3-3.5× gaps -- each is a partial, targeted improvement for
  my-lisp to measure independently before committing to, per the same
  "verify, don't assume" discipline this whole cross-repo effort has
  held throughout.
