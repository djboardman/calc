use calc_core::{CalcErrorKind, Value, evaluate_edited_document, evaluate_new_document};

#[test]
fn new_document_evaluation_returns_per_line_results() {
    let document = evaluate_new_document("price = 10\ntax = price * 0.2\nprice + tax");

    assert_eq!(document.lines.len(), 3);
    assert_eq!(document.lines[0].line, 0);
    assert_eq!(
        document.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value::Number(10.0))
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value::Number(2.0))
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(12.0))
    );
}

#[test]
fn document_evaluation_supports_precedence_parentheses_and_unary_minus() {
    let document = evaluate_new_document("1 + 2 * 3\n(1 + 2) * 3\n-5 + 2");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value::Number(7.0))
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value::Number(9.0))
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(-3.0))
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
    let name = document.lines[0]
        .defines
        .as_ref()
        .expect("line defines total");

    assert_eq!(name.parts.len(), 1);
    assert_eq!(document.symbol_text(name.parts[0]), "total");
}

#[test]
fn edited_document_reevaluates_dependent_later_lines() {
    let original = evaluate_new_document("price = 10\ntax = price * 0.2\nprice + tax");
    let edited = evaluate_edited_document(original, "price = 20\ntax = price * 0.2\nprice + tax");

    assert_eq!(
        edited.lines[0].result.as_ref().expect("line 0 succeeds"),
        &Some(Value::Number(20.0))
    );
    assert_eq!(
        edited.lines[1].result.as_ref().expect("line 1 succeeds"),
        &Some(Value::Number(4.0))
    );
    assert_eq!(
        edited.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(24.0))
    );
}

#[test]
fn edited_document_handles_inserted_lines() {
    let original = evaluate_new_document("price = 10\nprice + tax");
    let edited = evaluate_edited_document(original, "price = 10\ntax = 2\nprice + tax");

    assert_eq!(edited.lines[2].line, 2);
    assert_eq!(
        edited.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(12.0))
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
        &Some(Value::Number(1.0))
    );
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(2.0))
    );
    assert_eq!(
        document.lines[3].result.as_ref().expect("line 3 succeeds"),
        &Some(Value::Number(5.0))
    );
}

#[test]
fn sections_define_and_resolve_qualified_variables() {
    let document = evaluate_new_document(
        "house:\n  stairs:\n    total = 10\n    deposit = total * 0.5\n\ntotal:\n  deposit = house.stairs.deposit",
    );

    assert_eq!(document.lines[0].result, Ok(None));
    assert_eq!(document.lines[1].result, Ok(None));
    assert_eq!(
        document.lines[2].result.as_ref().expect("line 2 succeeds"),
        &Some(Value::Number(10.0))
    );
    assert_eq!(
        document.lines[3].result.as_ref().expect("line 3 succeeds"),
        &Some(Value::Number(5.0))
    );
    assert_eq!(
        document.lines[6].result.as_ref().expect("line 6 succeeds"),
        &Some(Value::Number(5.0))
    );

    let name = document.lines[3]
        .defines
        .as_ref()
        .expect("line defines deposit");
    let parts = name
        .parts
        .iter()
        .map(|symbol| document.symbol_text(*symbol))
        .collect::<Vec<_>>();
    assert_eq!(parts, ["house", "stairs", "deposit"]);
}

#[test]
fn leading_tabs_are_invalid_indentation() {
    let document = evaluate_new_document("\tvalue = 1");
    let error = document.lines[0]
        .result
        .as_ref()
        .expect_err("tab indentation is invalid");

    assert_eq!(error.kind, CalcErrorKind::InvalidIndentation);
}

#[test]
fn dotted_section_headers_are_invalid() {
    let document = evaluate_new_document("house.stairs:");
    let error = document.lines[0]
        .result
        .as_ref()
        .expect_err("dotted section header is invalid");

    assert_eq!(error.kind, CalcErrorKind::InvalidSectionHeader);
}

#[test]
fn core_infers_money_values_from_currency_symbol_literals() {
    let document = evaluate_new_document("price = £100");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "GBP".to_string(),
            minor_units: 10000
        })
    );
}

#[test]
fn core_infers_currency_values_from_iso_and_symbol_literals() {
    let document = evaluate_new_document("currency = USD\nsymbol = €");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Currency("USD".to_string()))
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line succeeds"),
        &Some(Value::Currency("EUR".to_string()))
    );
}

#[test]
fn core_infers_money_values_from_iso_currency_literals() {
    let document = evaluate_new_document("price = USD99.99");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 9999
        })
    );
}

#[test]
fn core_stores_money_values_with_two_decimal_places() {
    let document = evaluate_new_document("price = USD99.999");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 10000
        })
    );
}

#[test]
fn core_infers_text_values_from_double_quoted_literals() {
    let document = evaluate_new_document("message = \"Hello, world!\"");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Text("Hello, world!".to_string()))
    );
}

#[test]
fn core_infers_boolean_values_from_true_and_false_literals() {
    let document = evaluate_new_document("yes = true\nno = false");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Boolean(true))
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line succeeds"),
        &Some(Value::Boolean(false))
    );
}

#[test]
fn core_infers_list_values_from_homogeneous_list_literals() {
    let document = evaluate_new_document("values = [1, 2, 3]");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]))
    );
}

#[test]
fn core_accepts_list_literals_with_a_trailing_comma() {
    let document = evaluate_new_document("values = [1, 2, 3,]");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]))
    );
}

#[test]
fn core_rejects_mixed_type_list_literals() {
    let document = evaluate_new_document("values = [1, true]");
    let error = document.lines[0]
        .result
        .as_ref()
        .expect_err("mixed list is invalid");

    assert_eq!(error.kind, CalcErrorKind::MixedListTypes);
}

#[test]
fn core_evaluates_same_currency_money_addition_and_subtraction() {
    let document = evaluate_new_document("total = USD10 + USD2.50\nremaining = total - USD1");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 1250
        })
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 1150
        })
    );
}

#[test]
fn core_rejects_mixed_currency_money_addition_and_subtraction() {
    let document = evaluate_new_document("total = USD10 + GBP10");
    let error = document.lines[0]
        .result
        .as_ref()
        .expect_err("mixed currency arithmetic is invalid");

    assert_eq!(error.kind, CalcErrorKind::UnsupportedTypeOperation);
}

#[test]
fn core_evaluates_text_concatenation_with_plus() {
    let document = evaluate_new_document("message = \"Hello, \" + \"world!\"");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Text("Hello, world!".to_string()))
    );
}

#[test]
fn core_evaluates_money_multiplied_and_divided_by_number() {
    let document = evaluate_new_document("double = USD10 * 2\nhalf = USD10 / 2");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 2000
        })
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: 500
        })
    );
}

#[test]
fn core_rejects_unsupported_operator_type_combinations() {
    let document = evaluate_new_document("bad = true + false");
    let error = document.lines[0]
        .result
        .as_ref()
        .expect_err("boolean addition is invalid");

    assert_eq!(error.kind, CalcErrorKind::UnsupportedTypeOperation);
}

#[test]
fn core_evaluates_unary_minus_for_number_and_money() {
    let document = evaluate_new_document("-1\n-USD10");

    assert_eq!(
        document.lines[0].result.as_ref().expect("line succeeds"),
        &Some(Value::Number(-1.0))
    );
    assert_eq!(
        document.lines[1].result.as_ref().expect("line succeeds"),
        &Some(Value::Money {
            currency: "USD".to_string(),
            minor_units: -1000
        })
    );
}
