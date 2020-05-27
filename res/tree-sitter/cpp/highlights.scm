; Functions

(call_expression
  function: (scoped_identifier
    name: (identifier) @funccall))

(template_function
  name: (identifier) @funccall)

(template_method
  name: (field_identifier) @funccall)

(template_function
  name: (scoped_identifier
    name: (identifier) @funccall))

(function_declarator
  declarator: (scoped_identifier
    name: (identifier) @funccall))

(function_declarator
  declarator: (scoped_identifier
    name: (identifier) @funccall))

(function_declarator
  declarator: (field_identifier) @funccall)

; Types

((namespace_identifier) @type
 (#match? @type "^[A-Z]"))

(auto) @type

; Constants

(this) @variable.builtin
(nullptr) @literal.numeric

; Keywords

"catch" @keyword
"class" @keyword
"constexpr" @keyword
"delete" @keyword
"explicit" @keyword
"final" @keyword
"friend" @keyword
"mutable" @keyword
"namespace" @keyword
"noexcept" @keyword
"new" @keyword
"override" @keyword
"private" @keyword
"protected" @keyword
"public" @keyword
"template" @keyword
"throw" @keyword
"try" @keyword
"typename" @keyword
"using" @keyword
"virtual" @keyword
