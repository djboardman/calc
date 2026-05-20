use std::collections::HashSet;

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse, Position, Url};

use crate::{document_store::DocumentStore, result_adapter};

pub(crate) fn completions(
    documents: &DocumentStore,
    uri: &Url,
    position: Position,
) -> Option<CompletionResponse> {
    let document = documents.get(uri)?;
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

        if seen.insert(insert_text.clone()) {
            items.push(completion_item(insert_text.clone(), insert_text));
        }
        if seen.insert(full_name.clone()) {
            items.push(completion_item(full_name.clone(), full_name));
        }
    }

    Some(CompletionResponse::Array(items))
}

fn completion_item(label: String, insert_text: String) -> CompletionItem {
    CompletionItem {
        label,
        kind: Some(CompletionItemKind::VARIABLE),
        insert_text: Some(insert_text),
        ..CompletionItem::default()
    }
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
