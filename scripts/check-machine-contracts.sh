#!/usr/bin/env bash
# Mechanical CI provenance gate, NOT a Lisp evaluator or semantic oracle.
# Source of language/machine authority: pinned external/sens contracts.
set -euo pipefail

report=".guard-report.txt"
: > "$report"

pass() { printf 'MACHINE_IMPORT_PASS check=%s evidence=%s\n' "$1" "$2"; }
fail() {
  printf 'MACHINE_IMPORT_FAILURE check=%s detail=%s\n' "$1" "$2" | tee -a "$report" >&2
  exit 1
}
require_file() {
  [[ -f "$2" ]] || fail "$1" "required pinned SENS file missing: $2"
  pass "$1" "$2"
}
require_literal() {
  local check="$1" path="$2" literal="$3"
  require_file "$check:file" "$path"
  grep -Fq -- "$literal" "$path" ||
    fail "$check" "required authority clause absent: $literal in $path"
  pass "$check" "$literal"
}

echo 'MACHINE_IMPORT_EXECUTOR=bash ROLE=CI_GLUE_NO_SEMANTICS'
echo 'MACHINE_IMPORT_AUTHORITY=pinned-SENS NO_SEMANTIC_VERDICT'
submodule=external/sens
[[ -d "$submodule" ]] || fail submodule "external/sens missing"
expected="$(git rev-parse 'HEAD:external/sens')" ||
  fail gitlink "cannot read recorded gitlink"
actual="$(git -C "$submodule" rev-parse HEAD)" ||
  fail gitlink "submodule checkout missing"
[[ "$expected" == "$actual" ]] ||
  fail gitlink "recorded=$expected checked_out=$actual"
[[ "$(git -C "$submodule" cat-file -t "$actual")" == "commit" ]] ||
  fail gitlink "pinned object is not a commit"
pass gitlink "$actual"

boundary="$submodule/machine-lowering-boundary.lisp"
layout="$submodule/memory-layout-contract.lisp"
lowering="$submodule/lib/machine/lowering/semantic-x86-64.lisp"
encoder="$submodule/lib/machine/encoding/x86-64.lisp"
require_file boundary "$boundary"
require_file layout "$layout"
require_file lowering "$lowering"
require_file encoder "$encoder"
require_literal boundary.schema "$boundary" '(schema machine-lowering-boundary/2)'
require_literal layout.version "$layout" '(version . (1 0))'

# Contract is owned by the pinned SENS, NOT by the local WSM/assembly.
require_literal boundary.semantic-owner "$boundary" '(semantic-authority sens)'
require_literal boundary.direction "$boundary" '(lowering-direction semantic-to-machine)'
require_literal boundary.no-isa-semantic-ids "$boundary" '(semantic-id-from-isa forbidden)'
require_literal boundary.effect-layer "$boundary" '(canonical-machine-effect-layer lib/machine/effect-boundary.lisp)'
require_literal boundary.no-reverse-authority "$boundary" '(reverse-authority forbidden)'

# Compatibility profile is checked as a PINNED HISTORICAL PROJECTION.
# Its W8 spellings are evidence of that transport snapshot only:
# they do not create present-day domain IDs or semantic admission.
for historical_word in 00000010 00000100 00000101 00000110; do
  grep -Fq -- "$historical_word" "$lowering" ||
    fail legacy-projection "missing pinned historical coordinate $historical_word"
  pass historical-projection "$historical_word"
done

# Reject locally invented truth-tags. The asm nucleus is a mechanism only.
require_file nucleus asm/nucleus.s
if grep -Eq '^[[:space:]]*\.equ[[:space:]]+TAG_TRUE([[:space:],]|$)' asm/nucleus.s; then
  fail invented-truth 'asm/nucleus.s defines local .equ TAG_TRUE'
fi
grep -Eq 'hand-written|Not generated' asm/nucleus.s ||
  fail nucleus-provenance 'assembly must declare hand-written mechanism status'
pass nucleus-provenance 'hand-written mechanism, no local truth-tag'

sha256sum "$boundary" "$layout" "$lowering" "$encoder"
echo "MACHINE_IMPORT_VERDICT=PASS role=CI_GLUE_NO_SEMANTICS gitlink=$actual"
