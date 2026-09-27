// Shared BM25 query hygiene for every tantivy search (`find`, `manual`,
// `suggest`, `verdict`). Users type Pine and prose, not tantivy query
// syntax: `NASDAQ:AAPL` would otherwise parse as a search of a field named
// `NASDAQ`, and `strategy.exit(` as an unclosed group. Query-syntax
// characters become spaces, leaving plain terms for the parser to OR together.

/// `q` with tantivy query-syntax characters replaced by spaces.
pub(crate) fn plain_terms(q: &str) -> String {
    q.chars()
        .map(|c| match c {
            ':' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '~' | '"' | '\'' | '+' | '-' | '!'
            | '*' | '\\' | '/' | '<' | '>' | '=' | '?' | '&' | '|' => ' ',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_characters_become_spaces() {
        assert_eq!(plain_terms("NASDAQ:AAPL"), "NASDAQ AAPL");
        assert_eq!(plain_terms("strategy.exit("), "strategy.exit ");
        assert_eq!(plain_terms("a ?: b"), "a    b");
    }
}
