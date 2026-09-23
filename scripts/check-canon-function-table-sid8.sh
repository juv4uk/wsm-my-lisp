#!/usr/bin/env bash
# SID8-ONLY guard for docs/canon-function-table.md (wsm-my-lisp#49,
# ecosystem#16, 2026-09-23): function identity is a bare eight-bit binary
# token, never a decimal/hex/short alias. A prior revision of that table
# rendered ids as zero-padded decimal ("0004" for cons's real id
# 00000100) -- exactly the alternate functional-identity spelling the
# SID8-ONLY law forbids. Width, leading zeros and bit order are
# load-bearing and must never be reformatted for display.
#
# Factored out of check-lisp-first-authority.sh/.lisp so both parity
# implementations (docs/parity/lisp-first-guard-parity.md) run the exact
# same check instead of two hand-maintained copies of the same awk.
#
# Prints nothing and exits 0 if docs/canon-function-table.md is absent or
# every id column is already a bare 8-bit token. Otherwise prints each
# offending "<label> -> \"<value>\"" line and exits 1.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TABLE="$ROOT/docs/canon-function-table.md"

[[ -f "$TABLE" ]] || exit 0

bad_ids=$(awk -F'|' '
  /^\| [a-z]/ {
    gsub(/^[ \t]+|[ \t]+$/, "", $2);
    gsub(/^[ \t]+|[ \t]+$/, "", $3);
    if ($2 == "canonical identity") next;
    if ($3 ~ /^-+$/) next;
    if ($3 !~ /^[01]{8}$/) print $2 " -> \"" $3 "\"";
  }
' "$TABLE")

if [[ -n "$bad_ids" ]]; then
  echo "$bad_ids"
  exit 1
fi
