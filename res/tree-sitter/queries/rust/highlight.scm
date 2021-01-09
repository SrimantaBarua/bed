[
 "as"
 "break"
 "const"
 "continue"
 (crate)
 "else"
 "enum"
 "extern"
 "false"
 "fn"
 "for"
 "if"
 "impl"
 "in"
 "let"
 "loop"
 "macro_rules!"
 "match"
 "mod"
 "move"
 (mutable_specifier)
 "pub"
 "ref"
 "return"
 (self)
 "static"
 "struct"
 (super)
 "trait"
 "true"
 "type"
 "union"
 "unsafe"
 "use"
 "where"
 "while"
] @keyword

[
 "("
 ")"
 "{"
 "}"
 "["
 "]"
 ":"
 "::"
 ";"
 ","
 "->"
 ".."
 "..."
 "..="
 "."
] @punctuation

"." @punctuation.accessor

[
 "$"
 "?"
 "+"
 "-"
 "*"
 "/"
 "%"
 "&&"
 "||"
 "^"
 "=="
 "!="
 "<<"
 ">>"
 "="
 "+="
 "-="
 "*="
 "/="
 "%="
 "&="
 "|="
 "^="
 "<<="
 ">>="
] @operator

(macro_rule
  "=>" @operator)

(fragment_specifier) @type

(attribute_item) @keyword
(inner_attribute_item) @keyword

(mod_item (identifier) @module-def)

(enum_variant (identifier) @enum)

(dynamic_type "dyn" @keyword)

(extern_crate_declaration
  name: (identifier) @import
  alias: (identifier)? @import)

; Assume all-caps names are constants
((identifier) @constant
 (#match? @constant "^[A-Z][A-Z\\d_]+$'"))

(const_item
  name: (identifier) @constant)

(function_item
  name: (_) @funcdefn)

(function_modifiers) @keyword

(trait_item
  name: (type_identifier) @interface)

(associated_type
  name: (type_identifier) @type)

(type_parameters
  "<" @punctuation
  ">" @punctuation)

(use_as_clause
  alias: (identifier) @type)

(parameters
   "_" @punctuation)

(variadic_parameter) @punctuation

(scoped_type_identifier
  name: (type_identifier) @type)

(visibility_modifier) @keyword
(extern_modifier) @keyword

(type_identifier) @type
(primitive_type) @type.builtin

(bracketed_type
  "<" @punctuation
  ">" @punctuation)

(lifetime) @label

(generic_function
  function: [
    (identifier) @funccall
    (scoped_identifier
      name: (identifier) @funccall)
  ])

(pointer_type
  "*" @keyword)

(empty_type) @type

(macro_invocation
  macro: [
    (identifier) @funccall.macro
    (scoped_identifier
      name: (identifier) @funccall.macro)
  ]
  "!" @funccall.macro)

(binary_expression
  operator: [
    "|"
    "<"
    ">"
  ] @operator)

(call_expression
  function: [
    (identifier) @funccall
    (field_expression
      field: (_) @funccall)
  ])

(loop_label) @label

(closure_parameters
  "|" @punctuation)

(await_expression) @keyword
(async_block
  "async" @keyword)

(tuple_struct_pattern
  type: [
    (identifier) @type
    (scoped_identifier
      name: (identifier) @type)
  ])

(captured_pattern
  "@" @operator)

(integer_literal) @literal.number
(string_literal) @literal.string
(char_literal) @literal.char
(escape_sequence) @literal.escape
(boolean_literal) @literal.bool

(line_comment) @comment
(block_comment) @comment
