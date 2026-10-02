// Shared BM25 query building for every tantivy search (`find`, `manual`,
// `suggest`, `verdict`). Users type Pine and prose, not tantivy query syntax,
// so no query parser is involved: `NASDAQ:AAPL` would otherwise parse as a
// search of a field named `NASDAQ`, `strategy.exit(` as an unclosed group,
// and an uppercase `AND` / `OR` / `NOT` in plain words as an operator.
//
// Query-syntax characters become spaces and the rest splits on whitespace into
// literals. Each distinct literal (compared case-sensitively, before
// tokenization, as tantivy's parser deduplicates) is run through the field's
// own tokenizer: one token is a term, several are a phrase at the positions
// the tokenizer emitted, none is dropped. The literals are OR-ed. This is what
// the lenient parser built for sanitized input, minus its operators.

use std::collections::BTreeSet;

use anyhow::Result;
use tantivy::query::{BooleanQuery, EmptyQuery, PhraseQuery, Query, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::{Index, Term};

/// `q` with tantivy query-syntax characters replaced by spaces.
fn plain_terms(q: &str) -> String {
    q.chars()
        .map(|c| match c {
            ':' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '~' | '"' | '\'' | '+' | '-' | '!'
            | '*' | '\\' | '/' | '<' | '>' | '=' | '?' | '&' | '|' => ' ',
            _ => c,
        })
        .collect()
}

/// The plain-words query for `q` over `field` of `index`. Matches nothing
/// when no literal yields a token.
pub(crate) fn field_query(index: &Index, field: Field, q: &str) -> Result<Box<dyn Query>> {
    let mut tokenizer = index.tokenizer_for_field(field)?;
    let sanitized = plain_terms(q);
    let mut seen = BTreeSet::new();
    let mut clauses: Vec<Box<dyn Query>> = Vec::new();
    for literal in sanitized.split_whitespace() {
        if !seen.insert(literal) {
            continue;
        }
        let mut terms: Vec<(usize, Term)> = Vec::new();
        let mut stream = tokenizer.token_stream(literal);
        while let Some(token) = stream.next() {
            terms.push((token.position, Term::from_field_text(field, &token.text)));
        }
        match terms.len() {
            0 => {}
            1 => {
                let (_, term) = terms.remove(0);
                clauses.push(Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs)));
            }
            _ => clauses.push(Box::new(PhraseQuery::new_with_offset(terms))),
        }
    }
    Ok(if clauses.is_empty() {
        Box::new(EmptyQuery)
    } else {
        Box::new(BooleanQuery::union(clauses))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::schema::{Schema, TEXT};

    #[test]
    fn syntax_characters_become_spaces() {
        assert_eq!(plain_terms("NASDAQ:AAPL"), "NASDAQ AAPL");
        assert_eq!(plain_terms("strategy.exit("), "strategy.exit ");
        assert_eq!(plain_terms("a ?: b"), "a    b");
    }

    /// The query's shape, as tantivy debug-prints it.
    fn shape(q: &str) -> String {
        let mut builder = Schema::builder();
        let f = builder.add_text_field("f", TEXT);
        let index = Index::create_in_ram(builder.build());
        let query = field_query(&index, f, q).expect("query");
        format!("{query:?}")
    }

    #[test]
    fn operators_are_plain_words_and_dotted_names_are_phrases() {
        // `NOT` is a term like any other, never an exclusion.
        let not = shape("TV NOT did");
        assert!(not.contains("\"not\""), "{not}");
        assert!(!not.contains("MustNot"), "{not}");
        // One literal that tokenizes into several terms is a phrase.
        assert!(shape("ta.sma").contains("PhraseQuery"));
        // Repeating the same literal adds nothing; distinct literals that
        // normalize alike stay distinct, as the parser kept them.
        assert_eq!(shape("rsi rsi"), shape("rsi"));
        assert_ne!(shape("RSI rsi"), shape("rsi"));
        assert_eq!(shape("?: ()"), format!("{EmptyQuery:?}"));
    }
}
