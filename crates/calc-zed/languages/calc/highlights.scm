(number) @number
(identifier) @variable
(qualified_identifier) @variable
(section_header
  name: (identifier) @namespace)
(comment) @comment
(result_comment) @hint

[
  "+"
  "-"
  "*"
  "/"
  "="
] @operator
