#!/usr/bin/env bash
# Exact consumer-side witness for wsm-my-lisp#43.
# It does not modify my-lisp Core1, CML, or mccarthy-eval.
set -euo pipefail

root_dir=$(git rev-parse --show-toplevel)
my_lisp_repo=${MY_LISP_REPO:-/home/agents/GitHub/my-lisp}
mccarthy_repo=${MCCARTHY_EVAL_REPO:-/home/agents/GitHub/mccarthy-eval}
core1_rev=${CORE1_REV:-01b67f398bb5bff1aa0ab0e25f7e09a73e51b86b}
seed_rev=1c6acdb9dac890b2a0535ec0453bd83dfdda7248
compiler_rev=a6bd9747196f2d676ab6883cbb4b84c7fcfc6947
core1_sha=e56cbbeb4f3aaededf14a8af6ce164ccc219cd3b535fcaf575329f0f5c8c3b34
compiler_sha=b401cd09a0040c64668aa98369cad4932c06767343de5214a0eae2dae3ce6cce
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT

require_rev() {
  git -C "$1" cat-file -e "$2^{commit}"
}

require_rev "$my_lisp_repo" "$core1_rev"
require_rev "$mccarthy_repo" "$seed_rev"
require_rev "$root_dir" "$compiler_rev"

git -C "$my_lisp_repo" show "$core1_rev:lib/core1.lisp" > "$scratch_dir/core1.lisp"
git -C "$root_dir" show "$compiler_rev:lib/compiler.lisp" > "$scratch_dir/compiler.lisp"
test "$(sha256sum "$scratch_dir/core1.lisp" | cut -d' ' -f1)" = "$core1_sha"
test "$(sha256sum "$scratch_dir/compiler.lisp" | cut -d' ' -f1)" = "$compiler_sha"

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
expect list-derived '(compiler-clause (quote ((quote A) (quote yes) (quote no))))' '((quote A) (quote yes) (quote no))'
expect not-derived '(compiler-clause-shape? (quote ((quote A) (quote yes) (quote no))))' 'T'

# + and - are compiler-recognized quoted names, not arithmetic executed by Core1.
expect plus-emitted-data '(compiler-primitive? (quote +))' 'T'
expect minus-emitted-data '(compiler-primitive? (quote -))' 'T'

printf 'CORE1-CONSUMER-AUDIT-PASS core1=%s compiler=%s seed=%s\n' \
  "$core1_rev" "$compiler_rev" "$seed_rev"
