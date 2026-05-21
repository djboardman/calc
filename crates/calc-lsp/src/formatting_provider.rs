use calc_core::LineEvaluation;
use tower_lsp::lsp_types::{DocumentFormattingParams, Position, Range, TextEdit};

use crate::{
    document_store::DocumentStore,
    result_comment::{expected_result_comment, result_comment_span, split_comment},
};

pub(crate) fn formatting(
    documents: &DocumentStore,
    params: &DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    let document = documents.get(&params.text_document.uri)?;
    let new_text = source_with_result_comments(&document.source, &document.evaluation.lines);

    if new_text == document.source {
        return Some(Vec::new());
    }

    Some(vec![TextEdit {
        range: full_document_range(&document.source),
        new_text,
    }])
}

fn source_with_result_comments(source: &str, lines: &[LineEvaluation]) -> String {
    source
        .split('\n')
        .zip(lines)
        .map(|(line_text, line)| line_with_result_comment(line_text, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_with_result_comment(line_text: &str, line: &LineEvaluation) -> String {
    let expected = expected_result_comment(line_text, line);
    let Some((code, _)) = split_comment(line_text) else {
        return match expected {
            Some(expected) => format!("{} # {}", line_text.trim_end(), expected),
            None => line_text.to_string(),
        };
    };

    if result_comment_span(line_text).is_none() {
        return line_text.to_string();
    }

    match expected {
        Some(expected) => format!("{} # {}", code.trim_end(), expected),
        None => code.trim_end().to_string(),
    }
}

fn full_document_range(source: &str) -> Range {
    let mut lines = source.split('\n').enumerate();
    let Some((mut last_line, mut last_text)) = lines.next() else {
        return Range::default();
    };

    for (line, text) in lines {
        last_line = line;
        last_text = text;
    }

    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: last_line as u32,
            character: last_text.chars().count() as u32,
        },
    }
}

#[cfg(test)]
mod tests {
    use calc_core::evaluate_new_document;

    use super::source_with_result_comments;

    #[test]
    fn writes_result_comments_for_successful_nonblank_lines() {
        let source = "price = 10\ntax = price * 0.2\nprice + tax";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "price = 10\ntax = price * 0.2 # = 2\nprice + tax # = 12"
        );
    }

    #[test]
    fn replaces_existing_result_comments() {
        let source = "price = 10\ntax = price * 0.2 # = 9";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "price = 10\ntax = price * 0.2 # = 2"
        );
    }

    #[test]
    fn removes_result_comments_from_literal_assignments() {
        let source = "price = 10 # = 10\nrate = .5 # = 0.5\nending = 5. # = 5\nloss = -10 # = -10";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "price = 10\nrate = .5\nending = 5.\nloss = -10"
        );
    }

    #[test]
    fn does_not_replace_ordinary_comments() {
        let source = "price = 10 # ordinary";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            source
        );
    }

    #[test]
    fn removes_existing_result_comments_from_lines_without_values() {
        let source = "# = 10\nmissing # = 1";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "\nmissing"
        );
    }

    #[test]
    fn writes_result_comments_for_all_supported_value_types() {
        let source = "money = USD10 + USD2\ntext = \"a\" + \"b\"\nboolean = true\nlist = [1, 2]";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "money = USD10 + USD2 # = $12.00\ntext = \"a\" + \"b\" # = \"ab\"\nboolean = true # = true\nlist = [1, 2] # = [1, 2]"
        );
    }

    #[test]
    fn writes_currency_symbols_for_gbp_usd_and_eur_money_result_comments() {
        let source =
            "gbp = GBP10 + GBP2\nusd = USD10 + USD2\neur = EUR10 + EUR2\nother = CAD10 + CAD2";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "gbp = GBP10 + GBP2 # = £12.00\nusd = USD10 + USD2 # = $12.00\neur = EUR10 + EUR2 # = €12.00\nother = CAD10 + CAD2 # = CAD12.00"
        );
    }
}
