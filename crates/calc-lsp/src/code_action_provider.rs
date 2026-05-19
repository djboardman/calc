use std::collections::HashMap;

use calc_core::LineEvaluation;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    Position, Range, TextEdit, WorkspaceEdit,
};

use crate::document_store::DocumentStore;

const TITLE: &str = "Write Calc result comments";
const RESULT_MARKER: &str = "=>";

pub(crate) fn code_actions(
    documents: &DocumentStore,
    params: &CodeActionParams,
) -> Option<CodeActionResponse> {
    let uri = &params.text_document.uri;
    let document = documents.get(uri)?;
    let new_text = source_with_result_comments(&document.source, &document.evaluation.lines);

    if new_text == document.source {
        return Some(Vec::new());
    }

    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: full_document_range(&document.source),
            new_text,
        }],
    );

    Some(vec![CodeActionOrCommand::CodeAction(CodeAction {
        title: TITLE.to_string(),
        kind: Some(CodeActionKind::SOURCE),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }),
        ..CodeAction::default()
    })])
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
    let value = line.result.as_ref().ok().and_then(Option::as_ref);
    let Some((code, comment)) = split_comment(line_text) else {
        return match value {
            Some(value) => format!(
                "{} # {RESULT_MARKER} {}",
                line_text.trim_end(),
                value.number
            ),
            None => line_text.to_string(),
        };
    };

    if !is_result_comment(comment) {
        return line_text.to_string();
    }

    match value {
        Some(value) => format!("{} # {RESULT_MARKER} {}", code.trim_end(), value.number),
        None => code.trim_end().to_string(),
    }
}

fn split_comment(line_text: &str) -> Option<(&str, &str)> {
    line_text.split_once('#')
}

fn is_result_comment(comment: &str) -> bool {
    comment.trim_start().starts_with(RESULT_MARKER)
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
            "price = 10 # => 10\ntax = price * 0.2 # => 2\nprice + tax # => 12"
        );
    }

    #[test]
    fn replaces_existing_result_comments() {
        let source = "price = 10 # => 9";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "price = 10 # => 10"
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
        let source = "# => 10\nmissing # => 1";
        let evaluation = evaluate_new_document(source);

        assert_eq!(
            source_with_result_comments(source, &evaluation.lines),
            "\nmissing"
        );
    }
}
