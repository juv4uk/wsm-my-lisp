#!/usr/bin/env bash
# Machine-checkable authority guard for wsm-my-lisp (owner P0 #15).
#
# Core self-hosting path must stay Lisp + asm (+ bounded C). Rust that
# implements reader/eval/apply semantics is allowed ONLY under dll/
# (frozen Cyberpunk host embed). This script fails closed on drift.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "AUTHORITY FAIL: $*" >&2; exit 1; }
ok() { echo "AUTHORITY OK: $*"; }

# --- 1. Required self-hosting artifacts ---
[[ -f asm/nucleus.s ]] || fail "missing asm/nucleus.s"
[[ -f asm/nucleus-win64.s ]] || fail "missing asm/nucleus-win64.s"
[[ -f docs/AUTHORITY.md ]] || fail "missing docs/AUTHORITY.md"
[[ -f docs/dll-inventory-2026-09-11.md ]] || fail "missing dll inventory"

# --- 2. No Rust semantic modules outside dll/ and harness test helpers ---
# Allowlist of paths where .rs may exist without being "new language authority".
# harness/ is witness drivers (links asm), not an evaluator.
while IFS= read -r -d '' f; do
  rel="${f#./}"
  case "$rel" in
    dll/*) ;;
    harness/*) ;;
    external/*) ;;
    *)
      # Flag known semantic filenames if they appear outside dll/
      base="$(basename "$rel")"
      case "$base" in
        eval.rs|reader.rs|printer.rs|ffi.rs|word.rs)
          fail "semantic Rust module outside dll/: $rel"
          ;;
      esac
      ;;
  esac
done < <(find . -name '*.rs' -not -path './external/*' -print0 2>/dev/null || true)

ok "no eval/reader/printer/ffi/word.rs outside dll/"

# --- 3. dll/ must stay explicitly marked frozen ---
grep -q 'frozen' dll/README.md || fail "dll/README.md must state freeze policy"
grep -qi 'cyberpunk' dll/README.md || fail "dll/README.md must name Cyberpunk host role"

ok "dll/ freeze documentation present"

# --- 4. Known dll semantic surface is exactly the inventory allowlist ---
EXPECTED=(
  dll/src/eval.rs
  dll/src/reader.rs
  dll/src/printer.rs
  dll/src/ffi.rs
  dll/src/word.rs
  dll/src/lib.rs
)
for e in "${EXPECTED[@]}"; do
  [[ -f "$e" ]] || fail "inventory expected file missing: $e"
done

# Any additional .rs under dll/src/ (not bin/) must be reviewed — list them.
extra=0
while IFS= read -r -d '' f; do
  rel="${f#./}"
  known=0
  for e in "${EXPECTED[@]}"; do
    [[ "$rel" == "$e" ]] && known=1 && break
  done
  if [[ $known -eq 0 ]]; then
    echo "AUTHORITY NOTE: extra dll module (allowed only if host/compat, not new Lisp capability): $rel"
    extra=$((extra + 1))
  fi
done < <(find dll/src -name '*.rs' -not -path 'dll/src/bin/*' -print0 2>/dev/null || true)

ok "dll/src inventory checked ($extra extra module(s) beyond baseline — review if new)"

# --- 5. Self-hosting CI must not treat dll as core authority ---
if [[ -f .github/workflows/self-hosting-authority.yml ]]; then
  if grep -E 'dll/' .github/workflows/self-hosting-authority.yml | grep -v '^\s*#' >/dev/null 2>&1; then
    # dll mention in comments is ok; path trigger on dll would couple host to core
    if grep -E '^\s*- "dll/' .github/workflows/self-hosting-authority.yml >/dev/null 2>&1; then
      fail "self-hosting-authority.yml must not path-trigger on dll/"
    fi
  fi
  ok "self-hosting workflow stays independent of dll/"
fi

echo "Lisp-first authority guard passed."
