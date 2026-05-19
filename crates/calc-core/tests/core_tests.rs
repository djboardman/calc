use calc_core::{CalcErrorKind, Value, evaluate_edited_document, evaluate_new_document};

#[test]
fn new_document_evaluation_returns_per_line_results() {
    let document = evaluate_new_document("price = 10\ntax = price * 0.2\nprice + tax");

    assert_eq!(document.lines.len(), 3);
    assert_eq!(document.lines[0].line, 0);
    assert_eq!(
        document.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value { number: 10.0 })
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value { number: 2.0 })
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value { number: 12.0 })
    );
}

#[test]
fn document_evaluation_supports_precedence_parentheses_and_unary_minus() {
    let document = evaluate_new_document("1 + 2 * 3\n(1 + 2) * 3\n-5 + 2");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value { number: 7.0 })
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value { number: 9.0 })
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value { number: -3.0 })
    );
}

#[test]
fn blank_lines_are_included_and_evaluate_to_no_value() {
    let document = evaluate_new_document("1\n\n2\n");

    assert_eq!(document.lines.len(), 4);
    assert_eq!(document.lines[1].result, Ok(None));
    assert_eq!(document.lines[3].result, Ok(None));
}

#[test]
fn defines_returns_assigned_symbol_text() {
    let document = evaluate_new_document("total = 12");
    let symbol = document.lines[0].defines.expect("line defines total");

    assert_eq!(document.symbol_text(symbol), "total");
}

#[test]
fn edited_document_reevaluates_dependent_later_lines() {
    let original = evaluate_new_document("price = 10\ntax = price * 0.2\nprice + tax");
    let edited = evaluate_edited_document(original, "price = 20\ntax = price * 0.2\nprice + tax");

    assert_eq!(
        edited.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value { number: 20.0 })
    );
    assert_eq!(
        edited.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value { number: 4.0 })
    );
    assert_eq!(
        edited.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value { number: 24.0 })
    );
}

#[test]
fn edited_document_handles_inserted_lines() {
    let original = evaluate_new_document("price = 10\nprice + tax");
    let edited = evaluate_edited_document(original, "price = 10\ntax = 2\nprice + tax");

    assert_eq!(edited.lines[2].line, 2);
    assert_eq!(
        edited.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value { number: 12.0 })
    );
}

#[test]
fn document_evaluation_reports_line_relative_errors() {
    let document = evaluate_new_document("1\nmissing + 1\n1 / 0");

    let missing = document.lines[1]
        .result
        .as_ref()
        .expect_err("variable is undefined");
    assert_eq!(missing.kind, CalcErrorKind::UndefinedVariable);
    assert_eq!(missing.span.start, 0);
    assert_eq!(missing.span.end, 7);

    let division = document.lines[2]
        .result
        .as_ref()
        .expect_err("division by zero");
    assert_eq!(division.kind, CalcErrorKind::DivisionByZero);
    assert_eq!(division.span.start, 2);
    assert_eq!(division.span.end, 3);
}

#[test]
fn comments_are_ignored_during_evaluation() {
    let document = evaluate_new_document("# comment\n1 # comment\nvalue = 2 # comment\nvalue + 3");

    assert_eq!(document.lines[0].result, Ok(None));
    assert_eq!(
        document.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value { number: 1.0 })
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value { number: 2.0 })
    );
    assert_eq!(
        document.lines[3].result.as_ref().expect("line 3 succeeds"),
        &Some(Value { number: 5.0 })
    );
}
