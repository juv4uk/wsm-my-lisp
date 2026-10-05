#!/usr/bin/env bash
set -euo pipefail

root_dir=$(git rev-parse --show-toplevel)
sens_repo=${SENS_REPO:-/home/agents/GitHub/sens}
mccarthy_repo=${MCCARTHY_EVAL_REPO:-/home/agents/GitHub/mccarthy-eval}
seed_rev=6031f92652066825a245c806c0773e9e524257bd
# Contract dependency: Core1 source and compiler-facing resolver are consumed
# from one exact SENS revision.  Do not read a mutable adjacent checkout as
# semantic authority.
sens_rev=e274400c7ef16d826b9220321094d624b450269c
scratch_dir=$(mktemp -d)
trap 'rm -rf "$scratch_dir"' EXIT

git -C "$mccarthy_repo" cat-file -e "$seed_rev^{commit}"
git -C "$sens_repo" cat-file -e "$sens_rev^{commit}"
git -C "$mccarthy_repo" show "$seed_rev:mccarthy-kernel.s" > "$scratch_dir/mccarthy-kernel.s"
gcc -no-pie -O0 -s -o "$scratch_dir/mccarthy-kernel" "$scratch_dir/mccarthy-kernel.s"

fixture="$scratch_dir/portable-sid8.lisp"
{
  git -C "$sens_repo" show "$sens_rev:lib/core1.lisp" | sed '/^[[:space:]]*;/d'
  printf '%s\n' '(C1-EVAL-PROGRAM-THEN'
  printf '%s\n' '  (QUOTE ('
  # Current Core1's S0 boundary is raw load followed by a Core1 self-load.
  # Every consumer source form then shares the same Core1 global environment.
  git -C "$sens_repo" show "$sens_rev:lib/core1.lisp" | sed '/^[[:space:]]*;/d'
  git -C "$sens_repo" show "$sens_rev:lib/core1-compiler-sid-resolver.lisp" | sed "/^[[:space:]]*;/d"
  sed '/^[[:space:]]*;/d' "$root_dir/lib/compiler.lisp"
  printf '%s\n' '  ))'
  printf '%s\n' '  (QUOTE (list'
  printf '%s\n' '    (compiler-form 00001100)'
  printf '%s\n' '    (compiler-form (quote FOO))'
  printf '%s\n' '    (compiler-form (quote +))'
  printf '%s\n' '    (compiler-form (quote (cons (quote A) (quote B)))))))'
} > "$fixture"

# A compiler consumer may resolve primitive *surfaces* only through SENS.
# A downstream table would create a competing semantic authority.
! grep -F 'compiler-primitive-sens' "$root_dir/lib/compiler.lisp" >/dev/null

"$scratch_dir/mccarthy-kernel" "$fixture" \
  > "$scratch_dir/actual.out" \
  2> "$scratch_dir/actual.err"

actual=$(tail -n 1 "$scratch_dir/actual.out")
expected='((sid 00001100) (var FOO) (var +) (prim 00000100 ((quote A) (quote B))))'
test "$actual" = "$expected"

test ! -s "$scratch_dir/actual.err"

awk '/^\(def compiler-primitive\?/{flag=1} flag{print} flag && /^$/{exit}' \
  "$root_dir/lib/compiler.lisp" > "$scratch_dir/primitive-def.lisp"
{
  git -C "$sens_repo" show "$sens_rev:lib/core1.lisp" | sed '/^[[:space:]]*;/d'
  printf '%s\n' '(C1-EVAL-PROGRAM-THEN'
  printf '%s\n' '  (QUOTE ('
  git -C "$sens_repo" show "$sens_rev:lib/core1.lisp" | sed '/^[[:space:]]*;/d'
  git -C "$sens_repo" show "$sens_rev:lib/core1-compiler-sid-resolver.lisp" | sed "/^[[:space:]]*;/d"
  sed '/^[[:space:]]*;/d' "$root_dir/lib/compiler.lisp"
  printf '%s\n' '  ))'
  printf '%s\n' '  (QUOTE (compiler-form (quote'
  cat "$scratch_dir/primitive-def.lisp"
  printf '%s\n' '  ))))'
} > "$scratch_dir/self-source.lisp"

"$scratch_dir/mccarthy-kernel" "$scratch_dir/self-source.lisp" \
  > "$scratch_dir/self-source.out" 2> "$scratch_dir/self-source.err"
self_source=$(tail -n 1 "$scratch_dir/self-source.out")
grep -F '(app (var C1-COMPILER-SID-FOR-SURFACE)' <<<"$self_source" >/dev/null
! grep -F 'compiler-primitive-sens' <<<"$self_source"

printf '%s\n' "$actual"
printf 'CORE1-PORTABLE-SID8-SELF-SOURCE-PASS\n'
printf 'CORE1-PORTABLE-SID8-PASS seed=%s sens=%s\n' "$seed_rev" "$sens_rev"
