#!/usr/bin/env bash
# Machine-checkable LOCAL PROVENANCE guard (CI_GLUE_NO_SEMANTICS).
#
# Core self-hosting path is Lisp-first. C and Rust are permitted as explicitly
# proven bootstrap/runtime/tooling substrates. The Rust host embed that used to
# live under dll/ (frozen Cyberpunk mirror) was deleted at Phase D of the
# migration to my-lisp-cyberpunk/host-runtime — this repo now carries no
# Rust reader/eval/apply implementation at all. This script fails closed on
# drift back toward that.

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

# --- 2. C/Rust substrates are allowed; semantic authority remains explicit ---
ok "C and Rust substrates are permitted with explicit provenance"

# --- 4. dll/ must be gone (Phase D complete) ---
[[ ! -d dll ]] || fail "dll/ still present — Phase D (delete Rust host embed) not complete"

ok "dll/ absent — Phase D complete"

# --- 5. Documentation authority: one active entry point, archive stays non-normative ---
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

# --- 6. Self-hosting CI must not reference dll as core authority ---
if [[ -f .github/workflows/self-hosting-authority.yml ]]; then
  if grep -E '^\s*- "dll/' .github/workflows/self-hosting-authority.yml >/dev/null 2>&1; then
    fail "self-hosting-authority.yml must not path-trigger on dll/ (deleted)"
  fi
  ok "self-hosting workflow carries no dll/ path trigger"
fi


# --- 7. Historical W8 table projection: retain byte-exact provenance of
# the old semantic-registry export. This does NOT establish semantic identity
# or override the current exact-width SENS domain ladder.
bad_ids=$(scripts/check-canon-function-table-sid8.sh) || \
  fail "historical W8 projection contains malformed coordinate:
$bad_ids"
ok "docs/canon-function-table.md: historical W8 coordinates preserved; no semantic authority"

echo "LOCAL_SOURCE_AUDIT=PASS executor=bash role=CI_GLUE_NO_SEMANTICS; C/Rust substrates allowed with provenance"
