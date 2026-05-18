module.exports = grammar({
  name: "calc",

  extras: $ => [
    /[ \t\r]/,
  ],

  rules: {
    source_file: $ => seq(
      repeat($._line),
      optional($.statement),
    ),

    _line: $ => seq(
      optional($.statement),
      "\n",
    ),

    statement: $ => choice(
      $.assignment_statement,
      $.expression_statement,
    ),

    assignment_statement: $ => seq(
      field("name", $.identifier),
      "=",
      field("value", $.expression),
    ),

    expression_statement: $ => $.expression,

    expression: $ => choice(
      $.number,
      $.identifier,
      $.parenthesized_expression,
      $.unary_expression,
      $.binary_expression,
    ),

    parenthesized_expression: $ => seq(
      "(",
      $.expression,
      ")",
    ),

    unary_expression: $ => prec(3, seq(
      field("operator", "-"),
      field("argument", $.expression),
    )),

    binary_expression: $ => choice(
      prec.left(1, seq(
        field("left", $.expression),
        field("operator", choice("+", "-")),
        field("right", $.expression),
      )),
      prec.left(2, seq(
        field("left", $.expression),
        field("operator", choice("*", "/")),
        field("right", $.expression),
      )),
    ),

    number: $ => token(choice(
      /\d+(\.\d*)?/,
      /\.\d+/,
    )),

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,
  },
});
