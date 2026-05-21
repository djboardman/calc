use std::collections::HashSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, Position, Range,
    TextEdit, Url,
};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn completions(
    documents: &DocumentStore,
    uri: &Url,
    position: Position,
) -> Option<CompletionResponse> {
    let document = documents.get(uri)?;
    let context = completion_context(&document.source, position)?;
    let scope = current_scope(&document.source, position.line as usize);
    let mut seen = HashSet::new();
    let mut items = Vec::new();

    for line in document
        .evaluation
        .lines
        .iter()
        .take(position.line as usize)
    {
        let Some(name) = line.defines.as_ref() else {
            continue;
        };
        if line.result.as_ref().is_err() {
            continue;
        }
        let full_name = result_adapter::qualified_name_text(&document.evaluation, name);
        let insert_text = shortest_visible_name(
            &name
                .parts
                .iter()
                .map(|symbol| document.evaluation.symbol_text(*symbol).to_string())
                .collect::<Vec<_>>(),
            &scope,
        );

        if candidate_matches_prefix(&insert_text, &context.prefix)
            && seen.insert(insert_text.clone())
        {
            items.push(completion_item(insert_text.clone(), insert_text, &context));
        }
        if candidate_matches_prefix(&full_name, &context.prefix) && seen.insert(full_name.clone()) {
            items.push(completion_item(full_name.clone(), full_name, &context));
        }
    }

    Some(CompletionResponse::Array(items))
}

fn completion_item(label: String, new_text: String, context: &CompletionContext) -> CompletionItem {
    CompletionItem {
        label,
        kind: Some(CompletionItemKind::VARIABLE),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: context.replacement_range,
            new_text,
        })),
        ..CompletionItem::default()
    }
}

fn candidate_matches_prefix(candidate: &str, prefix: &str) -> bool {
    prefix.is_empty() || candidate.starts_with(prefix)
}

struct CompletionContext {
    prefix: String,
    replacement_range: Range,
}

fn completion_context(source: &str, position: Position) -> Option<CompletionContext> {
    let line_text = source.split('\n').nth(position.line as usize)?;
    let cursor = position.character as usize;
    if cursor > line_text.chars().count() {
        return None;
    }

    let before_cursor = take_chars(line_text, cursor);
    if before_cursor.contains('#') {
        return None;
    }

    let equals_index = before_cursor.rfind('=')?;
    if before_cursor[equals_index + 1..].trim().is_empty() {
        return Some(CompletionContext {
            prefix: String::new(),
            replacement_range: Range {
                start: position,
                end: position,
            },
        });
    }

    let prefix = variable_path_prefix(&before_cursor);
    let start = cursor.saturating_sub(prefix.chars().count());

    Some(CompletionContext {
        prefix,
        replacement_range: Range {
            start: Position {
                line: position.line,
                character: start as u32,
            },
            end: position,
        },
    })
}

fn take_chars(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

fn variable_path_prefix(text: &str) -> String {
    text.chars()
        .rev()
        .take_while(|ch| matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.'))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn shortest_visible_name(name: &[String], scope: &[Section]) -> String {
    for len in (0..=scope.len()).rev() {
        if name.len() == len + 1
            && name
                .iter()
                .take(len)
                .zip(scope.iter().take(len))
                .all(|(left, right)| left == &right.name)
        {
            return name[len].clone();
        }
    }

    name.join(".")
}

#[derive(Debug)]
struct Section {
    indent: usize,
    name: String,
}

fn current_scope(source: &str, line: usize) -> Vec<Section> {
    let mut sections = Vec::new();

    for text in source.split('\n').take(line) {
        let code = text.split_once('#').map_or(text, |(before, _)| before);
        if code.trim().is_empty() {
            continue;
        }
        let indent = code.chars().take_while(|ch| *ch == ' ').count();
        while sections
            .last()
            .is_some_and(|section: &Section| indent <= section.indent)
        {
            sections.pop();
        }
        let trimmed = code.trim();
        if let Some(name) = trimmed.strip_suffix(':')
            && is_identifier(name.trim())
        {
            sections.push(Section {
                indent,
                name: name.trim().to_string(),
            });
        }
    }

    sections
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    matches!(first, 'A'..='Z' | 'a'..='z' | '_')
        && chars.all(|ch| matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_'))
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{CompletionResponse, CompletionTextEdit, Position, TextEdit, Url};

    use crate::{
        completion_provider::completions, document_input_adapter, document_store::DocumentStore,
    };

    #[test]
    fn returns_no_completions_in_comments() {
        let source = "value = 10\nresult = # val";
        let items = completion_items(source, 1, 14);

        assert!(items.is_empty());
    }

    #[test]
    fn returns_no_completions_outside_assignment_rhs() {
        let source = "value = 10\nval";
        let items = completion_items(source, 1, 3);

        assert!(items.is_empty());
    }

    #[test]
    fn completion_replaces_partially_typed_qualified_path() {
        let source = "level1:\n  level2:\n    var = 0\n\nresult = level1.";
        let items = completion_items(source, 4, 16);
        let item = items
            .iter()
            .find(|item| item.label == "level1.level2.var")
            .expect("qualified completion exists");
        let edit = item_text_edit(item);

        assert_eq!(edit.range.start.line, 4);
        assert_eq!(edit.range.start.character, 9);
        assert_eq!(edit.range.end.character, 16);
        assert_eq!(edit.new_text, "level1.level2.var");
    }

    #[test]
    fn completion_inserts_visible_short_name_on_rhs() {
        let source = "level1:\n  level2:\n    var = 0\n    result = ";
        let items = completion_items(source, 3, 13);

        assert!(items.iter().any(|item| item.label == "var"));
    }

    fn completion_items(
        source: &str,
        line: u32,
        character: u32,
    ) -> Vec<tower_lsp::lsp_types::CompletionItem> {
        let uri = Url::parse("file:///test.calc").expect("valid uri");
        let mut documents = DocumentStore::default();
        document_input_adapter::open_document(&mut documents, uri.clone(), source.to_string());
        let response = completions(&documents, &uri, Position { line, character });

        match response {
            Some(CompletionResponse::Array(items)) => items,
            Some(CompletionResponse::List(list)) => list.items,
            None => Vec::new(),
        }
    }

    fn item_text_edit(item: &tower_lsp::lsp_types::CompletionItem) -> &TextEdit {
        match item.text_edit.as_ref().expect("text edit exists") {
            CompletionTextEdit::Edit(edit) => edit,
            CompletionTextEdit::InsertAndReplace(_) => panic!("expected text edit"),
        }
    }
}
