#!/usr/bin/env bash
# Machine-checkable authority guard for wsm-my-lisp (owner P0 #15).
#
# Core self-hosting path must stay Lisp + asm (+ bounded C). The Rust host
# embed that used to live under dll/ (frozen Cyberpunk mirror) was deleted
# at Phase D of the migration to my-lisp-cyberpunk/host-runtime — this repo
# now carries no Rust reader/eval/apply implementation at all. This script
# fails closed on drift back toward that.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "AUTHORITY FAIL: $*" >&2; exit 1; }
ok() { echo "AUTHORITY OK: $*"; }

# --- 1. Required self-hosting artifacts ---
[[ -f asm/nucleus.s ]] || fail "missing asm/nucleus.s"
[[ -f asm/nucleus-win64.s ]] || fail "missing asm/nucleus-win64.s"
[[ -f docs/AUTHORITY.md ]] || fail "missing docs/AUTHORITY.md"
[[ -f docs/archive/completed-plans/dll-inventory-2026-09-11.md ]] || fail "missing archived dll inventory"

# --- 2. No Rust semantic modules anywhere (dll/ is gone; harness/ stays test-only) ---
# harness/ is witness drivers (links asm), not an evaluator.
while IFS= read -r -d '' f; do
  rel="${f#./}"
  case "$rel" in
    harness/*) ;;
    external/*) ;;
    *)
      base="$(basename "$rel")"
      case "$base" in
        eval.rs|reader.rs|printer.rs|ffi.rs|word.rs)
          fail "semantic Rust module found (dll/ is deleted, this repo carries none): $rel"
          ;;
      esac
      ;;
  esac
done < <(find . -name '*.rs' -not -path './external/*' -print0 2>/dev/null || true)

ok "no eval/reader/printer/ffi/word.rs anywhere in this repo"

# --- 3. dll/ must be gone (Phase D complete) ---
[[ ! -d dll ]] || fail "dll/ still present — Phase D (delete Rust host embed) not complete"

ok "dll/ absent — Phase D complete"

# --- 4. Documentation authority: one active entry point, archive stays non-normative ---
[[ -f docs/CURRENT.md ]] || fail "missing docs/CURRENT.md (documentation entry point, wsm-my-lisp#19)"
[[ -f docs/archive/README.md ]] || fail "missing docs/archive/README.md (non-normative warning)"

# Archived docs must self-identify as archived (cheap, mechanical check;
# whether an active doc's claims wrongly rest on archived content is a
# semantic judgment this script does not attempt).
for f in docs/archive/*/*.md; do
  [[ -f "$f" ]] || continue
  grep -qi 'ARCHIVED' "$f" || fail "archived doc missing ARCHIVED marker: $f"
done

ok "docs/CURRENT.md present, archive docs self-identify as archived"

# --- 5. Self-hosting CI must not reference dll as core authority ---
if [[ -f .github/workflows/self-hosting-authority.yml ]]; then
  if grep -E '^\s*- "dll/' .github/workflows/self-hosting-authority.yml >/dev/null 2>&1; then
    fail "self-hosting-authority.yml must not path-trigger on dll/ (deleted)"
  fi
  ok "self-hosting workflow carries no dll/ path trigger"
fi

echo "Lisp-first authority guard passed."
