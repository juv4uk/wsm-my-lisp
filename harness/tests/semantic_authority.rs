use wsm_os_target::{
    CLOSURE_ALIGNMENT, CLOSURE_BYTES, CLOSURE_DEFINITION_ID_OFFSET,
    CLOSURE_ENVIRONMENT_REF_OFFSET, CANONICAL_T, NIL, SYMBOL_ID_MAX, TAG_BITS, TAG_MASK, Tag,
};

const SYSV_NUCLEUS: &str = include_str!("../../asm/nucleus.s");
const WIN64_NUCLEUS: &str = include_str!("../../asm/nucleus-win64.s");

fn strip_block_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    loop {
        let Some(start) = rest.find("/*") else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("*/") else {
            break;
        };
        rest = &after_start[end + 2..];
    }
    out
}

fn equ_u64(source: &str, name: &str) -> Option<u64> {
    let prefix = format!(".equ {name},");
    source.lines().find_map(|line| {
        let trimmed = line.trim();
        let value = trimmed.strip_prefix(&prefix)?.trim();
        let token = value.split_whitespace().next()?;
        if let Some(hex) = token.strip_prefix("0x") {
            u64::from_str_radix(hex, 16).ok()
        } else {
            token.parse().ok()
        }
    })
}

fn function_body<'a>(source: &'a str, label: &str) -> Option<&'a str> {
    let start = source.find(&format!("\n{label}:"))?;
    let rest = &source[start + label.len() + 2..];
    let end = rest.find("\n    .globl ").unwrap_or(rest.len());
    Some(&rest[..end])
}

fn authority_violations(source: &str) -> Vec<String> {
    let code = strip_block_comments(source);
    let mut violations = Vec::new();

    // Legacy target-contract Tag::True may exist upstream as representation
    // history, але цей runtime не має права називати або емітувати його як
    // окрему Lisp-категорію істини.
    if code.contains("TAG_TRUE") {
        violations.push("local TAG_TRUE declaration/use creates backend truth authority".into());
    }

    match equ_u64(&code, "TAG_NIL") {
        Some(value) if value == NIL => {}
        other => violations.push(format!("TAG_NIL drift: {other:?}, contract={NIL}")),
    }
    match equ_u64(&code, "TAG_SYMBOL") {
        Some(value) if value == Tag::Symbol as u64 => {}
        other => violations.push(format!(
            "TAG_SYMBOL drift: {other:?}, contract={}",
            Tag::Symbol as u64
        )),
    }
    match equ_u64(&code, "TAG_MASK") {
        Some(value) if value == TAG_MASK => {}
        other => violations.push(format!("TAG_MASK drift: {other:?}, contract={TAG_MASK}")),
    }
    match equ_u64(&code, "SYM_T_ID") {
        Some(value) if value == SYMBOL_ID_MAX => {}
        other => violations.push(format!(
            "SYM_T_ID drift: {other:?}, contract={SYMBOL_ID_MAX}"
        )),
    }

    if !code.contains(".equ SYM_T_WORD, (SYM_T_ID << 3) | TAG_SYMBOL") {
        violations.push("SYM_T_WORD must stay a mechanical projection of canonical Symbol(t)".into());
    }

    let projected_t = (SYMBOL_ID_MAX << TAG_BITS) | Tag::Symbol as u64;
    if projected_t != CANONICAL_T {
        violations.push("target-contract CANONICAL_T no longer matches its Symbol projection".into());
    }

    for label in ["wsm_eq", "wsm_atom"] {
        match function_body(&code, label) {
            Some(body) => {
                if !body.contains("$SYM_T_WORD") {
                    violations.push(format!("{label} positive branch must emit canonical Symbol(t)"));
                }
                if body.contains("$TAG_TRUE") {
                    violations.push(format!("{label} must not emit a backend-only true tag"));
                }
            }
            None => violations.push(format!("cannot inspect {label} body")),
        }
    }

    // Closure support is optional per nucleus today, but once a substrate
    // exports it, every representation constant must remain a mechanical
    // projection of the ratified target contract rather than local semantics.
    if code.contains("wsm_closure_new:") {
        match equ_u64(&code, "TAG_CLOSURE") {
            Some(value) if value == Tag::Closure as u64 => {}
            other => violations.push(format!(
                "TAG_CLOSURE drift: {other:?}, contract={}",
                Tag::Closure as u64
            )),
        }
        for (name, expected) in [
            ("CLOSURE_ALIGNMENT", CLOSURE_ALIGNMENT as u64),
            ("CLOSURE_BYTES", CLOSURE_BYTES as u64),
            (
                "CLOSURE_DEFINITION_ID_OFFSET",
                CLOSURE_DEFINITION_ID_OFFSET as u64,
            ),
            (
                "CLOSURE_ENVIRONMENT_REF_OFFSET",
                CLOSURE_ENVIRONMENT_REF_OFFSET as u64,
            ),
        ] {
            match equ_u64(&code, name) {
                Some(value) if value == expected => {}
                other => violations.push(format!(
                    "{name} drift: {other:?}, contract={expected}"
                )),
            }
        }
        for label in [
            "wsm_closure_new",
            "wsm_closure_definition",
            "wsm_closure_environment",
        ] {
            if function_body(&code, label).is_none() {
                violations.push(format!("cannot inspect ratified closure ABI function {label}"));
            }
        }
    }

    violations
}

#[test]
fn both_asm_nuclei_preserve_semantic_authority_boundary() {
    for (name, source) in [("SysV", SYSV_NUCLEUS), ("Win64", WIN64_NUCLEUS)] {
        let violations = authority_violations(source);
        assert!(
            violations.is_empty(),
            "{name} nucleus violates semantic authority:\n{}",
            violations.join("\n")
        );
    }
}

#[test]
fn manufactured_true_fixture_is_rejected_fail_closed() {
    let deliberately_bad_backend = r#"
        .text
        .equ TAG_NIL, 1
        .equ TAG_TRUE, 2
        .equ TAG_SYMBOL, 4
        .equ TAG_MASK, 7
        .equ SYM_T_ID, 0x1FFFFFFFFFFFFFFF
        .equ SYM_T_WORD, (SYM_T_ID << 3) | TAG_SYMBOL
        .globl wsm_eq
wsm_eq:
        movl $TAG_TRUE, %eax
        ret
        .globl wsm_atom
wsm_atom:
        movl $TAG_TRUE, %eax
        ret
        .globl wsm_fail
wsm_fail:
        ret
"#;

    let violations = authority_violations(deliberately_bad_backend);
    assert!(
        violations.iter().any(|v| v.contains("TAG_TRUE"))
            && violations.iter().any(|v| v.contains("wsm_eq")),
        "negative fixture must be rejected specifically for manufactured truth: {violations:?}"
    );
}
