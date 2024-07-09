module.exports = grammar({
  name: "ignorefile",

  extras: ($) => [/\s+/, $.comment],

  rules: {
    source_file: ($) => repeat(seq($._instruction, "\n")),

    _identifier: ($) => /[a-zA-Z0-9-_]+/,

    _instruction: ($) =>
      choice(
        $.from_instruction,
        $.ignore_instruction,
        $.export_instruction,
      ),

    from_instruction: ($) =>
      seq(
        alias(/[fF][rR][oO][mM]/, "FROM"),
        $._identifier,
        optional(seq(alias(/[aA][sS]/, "AS"), field("as", $._identifier)))
      ),

    export_instruction: ($) =>
      seq(
        alias(/[eE][xX][pP][oO][rR][tT]/, "EXPORT"),
        $.path,
      ),

    ignore_instruction: ($) =>
      seq(
        alias(/[iI][gG][nN][oO][rR][eE]/, "IGNORE"),
        choice(
          $.path,
          $.double_quoted_string,
          $.single_quoted_string,
          $.unquoted_string
        )
      ),

    path: ($) =>
      seq(
        choice(
          /[^-\s\$<]/, // cannot start with a '-' to avoid conflicts with params
          /<[^<]/, // cannot start with a '<<' to avoid conflicts with heredocs (a single < is fine, though)
        ),
        repeat(token.immediate(/[^\s\$]+/))
      ),

    double_quoted_string: ($) =>
      seq(
        '"',
        repeat(
          choice(
            token.immediate(/[^"\n\\\$]+/),
            alias($.double_quoted_escape_sequence, $.escape_sequence),
            "\\",
          )
        ),
        '"'
      ),

    single_quoted_string: ($) =>
      seq(
        "'",
        repeat(
          choice(
            token.immediate(/[^'\n\\]+/),
            alias($.single_quoted_escape_sequence, $.escape_sequence),
            "\\",
          )
        ),
        "'"
      ),

    unquoted_string: ($) =>
      repeat1(
        choice(
          token.immediate(/[^\s\n\"'\\\$]+/),
          token.immediate("\\ "),
        )
      ),

    double_quoted_escape_sequence: () => token.immediate(
      choice(
        "\\\\",
        "\\\""
      )
    ),

    single_quoted_escape_sequence: () => token.immediate(
      choice(
        "\\\\",
        "\\'"
      )
    ),

    _non_newline_whitespace: () => token.immediate(/[\t ]+/),

    comment: () => /#.*/,
  },
});