#!/usr/bin/env bash
# Exact consumer-side witness for wsm-my-lisp#43.
# It does not modify my-lisp Core1, CML, or mccarthy-eval.
set -euo pipefail

root_dir=$(git rev-parse --show-toplevel)
my_lisp_repo=${MY_LISP_REPO:-/home/agents/GitHub/my-lisp}
mccarthy_repo=${MCCARTHY_EVAL_REPO:-/home/agents/GitHub/mccarthy-eval}
core1_rev=${CORE1_REV:-20eb4f65e7deb53084c1e7d11461b2d15cf2b7d6}
seed_rev=770ae6ce5d8c13d5a970f2b496fc74d944f480a5
compiler_rev=5b796e41784bb63b43a20349e42cc8586e157a80
core1_blob=c134b01bb37e45e0b9f29c098d7791538565b8e7
sid8_transport_blob=d1bb8cd05c9d8686b5adc0bac9277dc22d122c83
compiler_blob=16d02558b974f86fe28f79940e4fcd7bcac056f7
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT

require_rev() {
  git -C "$1" cat-file -e "$2^{commit}"
}

require_rev "$my_lisp_repo" "$core1_rev"
require_rev "$mccarthy_repo" "$seed_rev"
require_rev "$root_dir" "$compiler_rev"

git -C "$my_lisp_repo" show "$core1_rev:lib/core1.lisp" > "$scratch_dir/core1.lisp"
git -C "$my_lisp_repo" show "$core1_rev:lib/core1-sid8-transport.lisp" > "$scratch_dir/core1-sid8-transport.lisp"
git -C "$root_dir" show "$compiler_rev:lib/compiler.lisp" > "$scratch_dir/compiler.lisp"
test "$(git hash-object "$scratch_dir/core1.lisp")" = "$core1_blob"
test "$(git hash-object "$scratch_dir/core1-sid8-transport.lisp")" = "$sid8_transport_blob"
test "$(git hash-object "$scratch_dir/compiler.lisp")" = "$compiler_blob"

git -C "$mccarthy_repo" archive "$seed_rev" | tar -x -C "$scratch_dir"
gcc -no-pie -O0 -s -o "$scratch_dir/mccarthy-kernel" "$scratch_dir/mccarthy-kernel.s"

write_probe() {
  local name=$1
  local expression=$2
  {
    cat "$scratch_dir/core1.lisp"
    cat "$scratch_dir/core1-sid8-transport.lisp"
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
expect compiler-primitive-sid-cons '(compiler-primitive-sid (quote cons))' '00000100'
expect compiler-primitive-sid-car '(compiler-primitive-sid (quote car))' '00000101'
expect compiler-form-cons '(compiler-form (quote (cons (quote A) (quote B))))' '(prim 00000100 ((quote A) (quote B)))'
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
expect plus-emitted-sid '(compiler-primitive-sid (quote +))' '00001100'
expect minus-emitted-data '(compiler-primitive? (quote -))' 'T'
expect minus-emitted-sid '(compiler-primitive-sid (quote -))' '00001101'

printf 'CORE1-CONSUMER-AUDIT-PASS sid8=bare core1=%s compiler=%s seed=%s\n' \
  "$core1_rev" "$compiler_rev" "$seed_rev"
