#!/usr/bin/env bash
set -euo pipefail

root_dir=$(git rev-parse --show-toplevel)
sens_repo=${SENS_REPO:-/home/agents/GitHub/sens}
mccarthy_repo=${MCCARTHY_EVAL_REPO:-/home/agents/GitHub/mccarthy-eval}
seed_rev=6031f92652066825a245c806c0773e9e524257bd
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT

git -C "$mccarthy_repo" cat-file -e "$seed_rev^{commit}"
git -C "$mccarthy_repo" show "$seed_rev:mccarthy-kernel.s" > "$scratch_dir/mccarthy-kernel.s"
gcc -no-pie -O0 -s -o "$scratch_dir/mccarthy-kernel" "$scratch_dir/mccarthy-kernel.s"

fixture="$scratch_dir/portable-sid8.lisp"
{
  cat "$sens_repo/lib/core1.lisp"
  cat "$sens_repo/lib/core1-sid8-bootstrap-overlay.lisp"
  printf '%s\n' '(C1-EVAL-PROGRAM-THEN'
  printf '%s\n' '  (QUOTE ('
  sed '/^[[:space:]]*;/d' "$root_dir/lib/compiler.lisp"
  printf '%s\n' '  ))'
  printf '%s\n' '  (QUOTE (list'
  printf '%s\n' '    (compiler-form 00001100)'
  printf '%s\n' '    (compiler-form (quote FOO))'
  printf '%s\n' '    (compiler-form (quote +))'
  printf '%s\n' '    (compiler-form (quote (cons (quote A) (quote B)))))))'
} > "$fixture"

"$scratch_dir/mccarthy-kernel" "$fixture" \
  > "$scratch_dir/actual.out" \
  2> "$scratch_dir/actual.err"

actual=$(tail -n 1 "$scratch_dir/actual.out")
expected='((sid 00001100) (var FOO) (var +) (prim 00000100 ((quote A) (quote B))))'
test "$actual" = "$expected"

# The pinned SID8-aware S0 currently emits this diagnostic even for the
# upstream self-carry baseline; it is not introduced by this compiler slice.
test "$(cat "$scratch_dir/actual.err")" = 'CONDITION kind=UNBOUND name=SID'

awk '/^\(def compiler-primitive-sens/{flag=1} flag{print} flag && /^$/{exit}' \
  "$root_dir/lib/compiler.lisp" > "$scratch_dir/primitive-def.lisp"
{
  cat "$sens_repo/lib/core1.lisp"
  cat "$sens_repo/lib/core1-sid8-bootstrap-overlay.lisp"
  printf '%s\n' '(C1-EVAL-PROGRAM-THEN'
  printf '%s\n' '  (QUOTE ('
  sed '/^[[:space:]]*;/d' "$root_dir/lib/compiler.lisp"
  printf '%s\n' '  ))'
  printf '%s\n' '  (QUOTE (compiler-form (quote'
  cat "$scratch_dir/primitive-def.lisp"
  printf '%s\n' '  ))))'
} > "$scratch_dir/self-source.lisp"

"$scratch_dir/mccarthy-kernel" "$scratch_dir/self-source.lisp" \
  > "$scratch_dir/self-source.out" 2> "$scratch_dir/self-source.err"
self_source=$(tail -n 1 "$scratch_dir/self-source.out")
grep -F '(sid 00001100)' <<<"$self_source" >/dev/null
! grep -F '(var 00001100)' <<<"$self_source"

printf '%s\n' "$actual"
printf 'CORE1-PORTABLE-SID8-SELF-SOURCE-PASS\n'
printf 'CORE1-PORTABLE-SID8-PASS seed=%s\n' "$seed_rev"
