; Identifier conventions

; Assume all-caps names are constants
((identifier) @constant
 (#match? @constant "^[A-Z][A-Z\\d_]+$'"))

; Assume that uppercase names in paths are types
((scoped_identifier
  path: (identifier) @type)
 (#match? @type "^[A-Z]"))
((scoped_identifier
  path: (scoped_identifier
    name: (identifier) @type))
 (#match? @type "^[A-Z]"))

; Assume other uppercase names are enum constructors
((identifier) @constructor
 (#match? @constructor "^[A-Z]"))

; Function calls

(call_expression
  function: (identifier) @funccall)
(call_expression
  function: (field_expression
    field: (field_identifier) @funccall.method))
(call_expression
  function: (scoped_identifier
    "::"
    name: (identifier) @funccall))

(generic_function
  function: (identifier) @funccall)
(generic_function
  function: (scoped_identifier
    name: (identifier) @funccall))
(generic_function
  function: (field_expression
    field: (field_identifier) @funccall.method))

(macro_invocation
  macro: (identifier) @funccall.macro
  "!" @function.macro)

; Function definitions

(function_item (identifier) @funcdefn)
(function_signature_item (identifier) @funcdefn)

; Other identifiers

(type_identifier) @type
(primitive_type) @type.builtin
(field_identifier) @property

(line_comment) @comment
(block_comment) @comment

"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

(type_arguments
  "<" @punctuation.bracket
  ">" @punctuation.bracket)
(type_parameters
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

"::" @punctuation.delimiter
"." @punctuation.delimiter
";" @punctuation.delimiter

(parameter (identifier) @variable.parameter)

(lifetime (identifier) @label)

"break" @keyword
"const" @keyword
"continue" @keyword
"default" @keyword
"dyn" @keyword
"else" @keyword
"enum" @keyword
"extern" @keyword
"fn" @keyword
"for" @keyword
"if" @keyword
"impl" @keyword
"in" @keyword
"let" @keyword
"let" @keyword
"loop" @keyword
"macro_rules!" @keyword
"match" @keyword
"mod" @keyword
"move" @keyword
"pub" @keyword
"ref" @keyword
"return" @keyword
"static" @keyword
"struct" @keyword
"trait" @keyword
"type" @keyword
"union" @keyword
"unsafe" @keyword
"use" @keyword
"where" @keyword
"while" @keyword
(mutable_specifier) @keyword
(use_list (self) @keyword)
(scoped_use_list (self) @keyword)
(scoped_identifier (self) @keyword)
(super) @keyword

(self) @variable.builtin

(char_literal) @literal.string
(string_literal) @literal.string
(raw_string_literal) @literal.string

(boolean_literal) @literal.boolean
(integer_literal) @literal.numeric
(float_literal) @literal.numeric

(escape_sequence) @literal.escape

(attribute_item) @decorator
(inner_attribute_item) @decorator

"as" @operator
"*" @operator
"&" @operator
"'" @operator
