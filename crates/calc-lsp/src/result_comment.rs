use calc_core::LineEvaluation;

pub(crate) const RESULT_MARKER: &str = "=";

pub(crate) fn expected_result_comment(line_text: &str, line: &LineEvaluation) -> Option<String> {
    if is_numeric_literal_assignment(line_text) {
        return None;
    }

    let value = line.result.as_ref().ok().and_then(Option::as_ref)?;
    Some(format!("{RESULT_MARKER} {}", value.display_text()))
}

pub(crate) fn split_comment(line_text: &str) -> Option<(&str, &str)> {
    let mut in_text = false;

    for (index, ch) in line_text.char_indices() {
        match ch {
            '"' => in_text = !in_text,
            '#' if !in_text => return Some((&line_text[..index], &line_text[index + 1..])),
            _ => {}
        }
    }

    None
}

pub(crate) fn result_comment_span(line_text: &str) -> Option<(usize, usize, &str)> {
    let (code, comment) = split_comment(line_text)?;
    if !is_result_comment(comment) {
        return None;
    }

    let comment_start = code.len() + 1;
    Some((comment_start, line_text.len(), comment))
}

pub(crate) fn is_result_comment(comment: &str) -> bool {
    comment.trim_start().starts_with(RESULT_MARKER)
}

fn is_numeric_literal_assignment(line_text: &str) -> bool {
    let code = split_comment(line_text)
        .map_or(line_text, |(before, _)| before)
        .trim();
    let Some((name, value)) = code.split_once('=') else {
        return false;
    };

    is_identifier(name.trim()) && is_number_literal_or_unary_literal(value.trim())
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    matches!(first, 'A'..='Z' | 'a'..='z' | '_')
        && chars.all(|ch| matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_'))
}

fn is_number_literal_or_unary_literal(text: &str) -> bool {
    is_number_literal(text)
        || text
            .strip_prefix('-')
            .is_some_and(|value| is_number_literal(value.trim_start()))
}

fn is_number_literal(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    let mut seen_digit = false;
    let mut seen_dot = false;

    for ch in text.chars() {
        match ch {
            '0'..='9' => seen_digit = true,
            '.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }

    seen_digit
}
