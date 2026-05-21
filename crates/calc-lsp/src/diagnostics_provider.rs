use calc_core::Span;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::{document_store::DocumentStore, result_adapter, result_comment};

pub(crate) fn diagnostics(documents: &DocumentStore, uri: &Url) -> Vec<Diagnostic> {
    let Some(document) = documents.get(uri) else {
        return Vec::new();
    };

    let calculation_diagnostics = document
        .evaluation
        .lines
        .iter()
        .filter_map(result_adapter::diagnostic)
        .collect::<Vec<_>>();

    let stale_result_diagnostics = document
        .source
        .split('\n')
        .zip(&document.evaluation.lines)
        .filter_map(|(line_text, line)| stale_result_comment_diagnostic(line_text, line))
        .collect::<Vec<_>>();

    calculation_diagnostics
        .into_iter()
        .chain(stale_result_diagnostics)
        .collect()
}

pub(crate) fn range(line: usize, span: &Span) -> Range {
    Range {
        start: Position {
            line: line as u32,
            character: span.start as u32,
        },
        end: Position {
            line: line as u32,
            character: span.end as u32,
        },
    }
}

fn stale_result_comment_diagnostic(
    line_text: &str,
    line: &calc_core::LineEvaluation,
) -> Option<Diagnostic> {
    let (start, end, comment) = result_comment::result_comment_span(line_text)?;
    let expected = result_comment::expected_result_comment(line_text, line);

    if expected.as_deref() == Some(comment.trim_start()) {
        return None;
    }

    Some(Diagnostic {
        range: Range {
            start: Position {
                line: line.line as u32,
                character: start as u32,
            },
            end: Position {
                line: line.line as u32,
                character: end as u32,
            },
        },
        severity: Some(DiagnosticSeverity::WARNING),
        source: Some("calc".to_string()),
        message: "Stale result comment".to_string(),
        ..Diagnostic::default()
    })
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::{
        diagnostics_provider::diagnostics, document_input_adapter, document_store::DocumentStore,
    };

    #[test]
    fn warns_for_stale_result_comments() {
        let uri = Url::parse("file:///test.calc").expect("valid uri");
        let mut documents = DocumentStore::default();
        document_input_adapter::open_document(
            &mut documents,
            uri.clone(),
            "price = 10\ntax = price * 0.2 # = 3".to_string(),
        );

        let diagnostics = diagnostics(&documents, &uri);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Stale result comment");
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn does_not_warn_for_current_or_missing_result_comments() {
        let uri = Url::parse("file:///test.calc").expect("valid uri");
        let mut documents = DocumentStore::default();
        document_input_adapter::open_document(
            &mut documents,
            uri.clone(),
            "price = 10\ntax = price * 0.2 # = 2\nprice + tax".to_string(),
        );

        assert!(diagnostics(&documents, &uri).is_empty());
    }

    #[test]
    fn warns_for_stale_non_number_result_comments() {
        let uri = Url::parse("file:///test.calc").expect("valid uri");
        let mut documents = DocumentStore::default();
        document_input_adapter::open_document(
            &mut documents,
            uri.clone(),
            "total = USD10 + USD2 # = USD9.00".to_string(),
        );

        let diagnostics = diagnostics(&documents, &uri);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Stale result comment");
    }
}
