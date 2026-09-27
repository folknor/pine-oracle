// Command-line spellings of structured observation parts:
//   diagnostic: `CODE|message|detail`, detail optional. Detail is a
//               comma-separated list of items: a `line:col-line:col` span,
//               `bar=N` (the bar a runtime error fired on), or any other
//               `key=value` pair, kept as template context (`ctx`).
//   candidate:  `name|model`, model optional.

use anyhow::{Result, bail};
use std::collections::BTreeMap;

use super::{Candidate, CandidateState, Diag};

/// Parse a `CODE|message|detail` diagnostic spec. Code format is checked by
/// validation (it depends on the question kind), not here.
pub fn parse_diag(spec: &str) -> Result<Diag> {
    let mut parts = spec.splitn(3, '|');
    let code = parts.next().unwrap_or_default().trim();
    let Some(message) = parts.next().map(str::trim) else {
        bail!("diagnostic `{spec}` must be CODE|message or CODE|message|detail");
    };
    if code.is_empty() || message.is_empty() {
        bail!("diagnostic `{spec}` needs both a code and a message");
    }
    let mut diag = Diag {
        code: code.to_string(),
        message: message.to_string(),
        span: None,
        bar: None,
        ctx: BTreeMap::new(),
    };
    let detail = parts.next().unwrap_or_default();
    for item in detail.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if is_span(item) {
            if diag.span.is_some() {
                bail!("diagnostic `{spec}` has more than one span");
            }
            diag.span = Some(item.to_string());
            continue;
        }
        let Some((key, value)) = item.split_once('=') else {
            bail!("diagnostic detail `{item}` is neither a line:col-line:col span nor key=value");
        };
        let (key, value) = (key.trim(), value.trim());
        if key.is_empty() || value.is_empty() {
            bail!("diagnostic detail `{item}` needs a key and a value");
        }
        if key == "bar" {
            let Ok(bar) = value.parse::<u64>() else {
                bail!("diagnostic detail `bar={value}` must be a non-negative integer");
            };
            if diag.bar.replace(bar).is_some() {
                bail!("diagnostic `{spec}` has more than one bar");
            }
        } else if diag
            .ctx
            .insert(key.to_string(), value.to_string())
            .is_some()
        {
            bail!("diagnostic `{spec}` repeats ctx key `{key}`");
        }
    }
    Ok(diag)
}

/// Parse a `name|model` candidate spec into an undecided candidate.
pub fn parse_candidate(spec: &str) -> Result<Candidate> {
    let (name, model) = match spec.split_once('|') {
        Some((name, model)) => (name.trim(), Some(model.trim())),
        None => (spec.trim(), None),
    };
    if name.is_empty() {
        bail!("candidate `{spec}` needs a name");
    }
    Ok(Candidate {
        name: name.to_string(),
        model: model.filter(|m| !m.is_empty()).map(str::to_string),
        state: CandidateState::Undecided,
    })
}

/// `line:col-line:col`, all positive decimal integers.
/// `line:col-line:col`: 1-based positions, start not after end.
pub(super) fn is_span(s: &str) -> bool {
    let Some((start, end)) = s.split_once('-') else {
        return false;
    };
    match (position(start), position(end)) {
        (Some(a), Some(b)) => a <= b,
        _ => false,
    }
}

fn position(s: &str) -> Option<(u64, u64)> {
    let (line, col) = s.split_once(':')?;
    let (line, col) = (number(line)?, number(col)?);
    (line >= 1 && col >= 1).then_some((line, col))
}

fn number(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diag_with_span() {
        let d = parse_diag("CE10099|Only libraries can contain exported functions.|9:1-10:5")
            .expect("valid");
        assert_eq!(d.code, "CE10099");
        assert_eq!(d.message, "Only libraries can contain exported functions.");
        assert_eq!(d.span.as_deref(), Some("9:1-10:5"));
        assert_eq!(d.bar, None);
        assert!(d.ctx.is_empty());
    }

    #[test]
    fn diag_with_bar_and_ctx() {
        let d = parse_diag("RE10044|halted|bar=100").expect("valid");
        assert_eq!(d.bar, Some(100));
        let d = parse_diag("CE10260|Cannot use the {typeKindName} keyword|typeKindName=const")
            .expect("valid");
        assert_eq!(d.ctx.get("typeKindName").map(String::as_str), Some("const"));
        assert_eq!(d.span, None);
    }

    #[test]
    fn diag_without_detail() {
        let d = parse_diag("CE10099|msg").expect("valid");
        assert_eq!(d.span, None);
    }

    #[test]
    fn diag_rejects_malformed() {
        assert!(parse_diag("CE10099").is_err(), "no message");
        assert!(parse_diag("|msg").is_err(), "no code");
        assert!(parse_diag("RE10044|m|bar=x").is_err(), "non-numeric bar");
        assert!(parse_diag("RE10044|m|bar=1,bar=2").is_err(), "two bars");
        assert!(
            parse_diag("CE1|m|junk").is_err(),
            "detail neither span nor pair"
        );
    }

    #[test]
    fn spans_are_one_based_and_ordered() {
        assert!(is_span("9:1-10:5"));
        assert!(is_span("13:14-13:15"));
        assert!(!is_span("0:0-0:0"), "zero positions");
        assert!(!is_span("10:5-9:1"), "reversed range");
        assert!(!is_span("13:14-15"), "abbreviated end");
    }

    #[test]
    fn candidate_spec() {
        let c = parse_candidate("gross|open term is (C - E) * Q").expect("valid");
        assert_eq!(c.name, "gross");
        assert_eq!(c.model.as_deref(), Some("open term is (C - E) * Q"));
        assert_eq!(c.state, CandidateState::Undecided);
        assert_eq!(parse_candidate("net").expect("valid").model, None);
        assert!(parse_candidate("|model").is_err());
    }
}
