#!/usr/bin/env bash
# Exact consumer-side witness for wsm-my-lisp#43.
# It does not modify sens Core1, CML, or mccarthy-eval.
set -euo pipefail

root_dir=$(git rev-parse --show-toplevel)
sens_repo=${SENS_REPO:-/home/agents/GitHub/sens}
mccarthy_repo=${MCCARTHY_EVAL_REPO:-/home/agents/GitHub/mccarthy-eval}
core1_rev=${CORE1_REV:-d359c4885e0609a6c8350daf45de157b40cf48f3}
seed_rev=1ae9745b66a1439c1929b0d9038c680567118a58
compiler_rev=4d59936c02e72370448c2b4bd257603865f58648
core1_blob=c134b01bb37e45e0b9f29c098d7791538565b8e7
compiler_blob=8015a4b96040e2c9bf6dd889a91527f5d6b2f9bc
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT

require_rev() {
  git -C "$1" cat-file -e "$2^{commit}"
}

require_rev "$sens_repo" "$core1_rev"
require_rev "$mccarthy_repo" "$seed_rev"
require_rev "$root_dir" "$compiler_rev"

git -C "$sens_repo" show "$core1_rev:lib/core1.lisp" > "$scratch_dir/core1.lisp"
git -C "$root_dir" show "$compiler_rev:lib/compiler.lisp" > "$scratch_dir/compiler.lisp"
test "$(git hash-object "$scratch_dir/core1.lisp")" = "$core1_blob"
test "$(git hash-object "$scratch_dir/compiler.lisp")" = "$compiler_blob"

git -C "$mccarthy_repo" archive "$seed_rev" | tar -x -C "$scratch_dir"
gcc -no-pie -O0 -s -o "$scratch_dir/mccarthy-kernel" "$scratch_dir/mccarthy-kernel.s"

write_probe() {
  local name=$1
  local expression=$2
  {
    cat "$scratch_dir/core1.lisp"
    printf '%s\n' '(C1-EVAL-PROGRAM-THEN'
    printf '%s\n' '  (QUOTE ('
    sed '/^[[:space:]]*;/d' "$scratch_dir/compiler.lisp"
    printf '%s\n' '  ))'
    printf '  (QUOTE %s))\n' "$expression"
  } > "$scratch_dir/$name.lisp"
}

run_probe() {
  "$scratch_dir/mccarthy-kernel" "$scratch_dir/$1.lisp"
}

expect() {
  local name=$1
  local expression=$2
  local expected=$3
  write_probe "$name" "$expression"
  local actual
  actual=$(run_probe "$name")
  printf '%s: %s\n' "$name" "$actual"
  test "$actual" = "$expected"
}

# The smallest whole-program compiler witness: deterministic IR from Core1.
expect compiler-program-empty '(compiler-program (quote ()))' 'NIL'
expect compiler-program-atom '(compiler-program (quote (FOO)))' '((var FOO))'
expect compiler-program-quote '(compiler-program (quote ((quote A))))' '((quote A))'
first=$(run_probe compiler-program-quote)
second=$(run_probe compiler-program-quote)
test "$first" = "$second"

# Real compiler runtime dependencies and their classifications.
expect compiler-form-cons '(compiler-form (quote (cons (quote A) (quote B))))' '(prim cons ((quote A) (quote B)))'
expect core1-clause-derived '(compiler-clause (quote ((quote A) (quote yes))))' '((quote A) (quote yes))'
expect core1-two-part-clause-shape '(compiler-clause-shape? (quote ((quote A) (quote yes))))' 'T'
expect core1-three-part-clause-rejected '(compiler-clause-shape? (quote ((quote A) (quote yes) (quote no))))' 'NIL'
# Keep the historical S0 witness bounded: it proves the Core1 COND compiler
# law directly. The full real compiler-nil? definition is exercised by CML's
# native S4 self-source witness (#237), where the larger tree does not exceed
# the historical seed's bounded stack/reader envelope.
expect compiler-core1-cond \
  '(compiler-form (quote (cond ((quote A) (quote B)))))' \
  '(cond ((quote A) (quote B)))'

# + and - are compiler-recognized quoted names, not arithmetic executed by Core1.
expect plus-emitted-data '(compiler-primitive? (quote +))' 'T'
expect minus-emitted-data '(compiler-primitive? (quote -))' 'T'

printf 'CORE1-CONSUMER-AUDIT-PASS core1=%s compiler=%s seed=%s\n' \
  "$core1_rev" "$compiler_rev" "$seed_rev"
