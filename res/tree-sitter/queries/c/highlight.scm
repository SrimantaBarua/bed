[
 "break"
 "case"
 "const"
 "continue"
 "default"
 "do"
 "else"
 "enum"
 "extern"
 "for"
 "goto"
 "if"
 "inline"
 "long"
 "register"
 "restrict"
 "return"
 "short"
 "signed"
 "sizeof"
 "static"
 "struct"
 "switch"
 "typedef"
 "union"
 "unsigned"
 "volatile"
 "while"

 "#include"
 "#define"
 "#if"
 "#ifdef"
 "#ifndef"
 "#else"
 "#elif"
] @keyword

[
 "+"
 "-"
 "*"
 "/"
 "%"
 "+="
 "-="
 "*="
 "/="
 "%="
 "|"
 "&"
 "^"
 "<<"
 ">>"
 "|="
 "&="
 "^="
 "<<="
 ">>="
 "++"
 "--"
 "~"
 "="
 "=="
 "!="
 "<"
 "<="
 ">"
 ">="
 "&&"
 "||"
 "!"
 ":"
 "?"
] @operator

[
 "{"
 "}"
 "["
 "]"
 "("
 ")"
 ","
 ";"
 "->"
 "."
 "..."
] @punctuation

[
 (string_literal)
 (system_lib_string)
] @literal.string

(null) @literal.numeric
(number_literal) @literal.numeric
(char_literal) @literal.string

(call_expression
  function: (identifier) @funccall)
(call_expression
  function: (field_expression
    field: (field_identifier) @funccall))

; (field_identifier) @property

(function_declarator
  declarator: (identifier) @funcdefn)

(statement_identifier) @label

; (preproc_ifdef
  ; name: (identifier) @macrodefn)
; (preproc_def
  ; name: (identifier) @macrodefn)
; (preproc_function_def
  ; name: (identifier) @macrodefn)

[
 (type_identifier)
 (primitive_type)
 (sized_type_specifier)
] @type

((identifier) @constant
 (#match? @constant "^[A-Z][A-Z\\d_]*$"))

(identifier) @variable

(comment) @comment

; (ms_call_modifier) @call-convention
; (storage_class_specifier) @type-qualifier
; (type_qualifier)          @type-qualifier
; (attribute_specifier)     @type-qualifier
; (ms_declspec_modifier)    @type-qualifier
