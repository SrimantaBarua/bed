; Special identifiers
;--------------------

((identifier) @constant
 (#match? @constant "^[A-Z_][A-Z\\d_]+$"))

((shorthand_property_identifier) @constant
 (#match? @constant "^[A-Z_][A-Z\\d_]+$"))

((identifier) @constructor
 (#match? @constructor "^[A-Z]"))

((identifier) @variable.builtin
 (#match? @variable.builtin "^(arguments|module|console|window|document)$")
 (#is-not? local))

((identifier) @funccall.builtin
 (#eq? @funccall.builtin "require")
 (#is-not? local))

; Function and method definitions
;--------------------------------

(function
  name: (identifier) @funcdefn)
(function_declaration
  name: (identifier) @funcdefn)
(method_definition
  name: (property_identifier) @funcdefn)

(pair
  key: (property_identifier) @funcdefn
  value: (function))
(pair
  key: (property_identifier) @funcdefn
  value: (arrow_function))

(assignment_expression
  left: (member_expression
    property: (property_identifier) @funcdefn)
  right: (arrow_function))
(assignment_expression
  left: (member_expression
    property: (property_identifier) @funcdefn)
  right: (function))

(variable_declarator
  name: (identifier) @funcdefn
  value: (arrow_function))
(variable_declarator
  name: (identifier) @funcdefn
  value: (function))

(assignment_expression
  left: (identifier) @funcdefn
  right: (arrow_function))
(assignment_expression
  left: (identifier) @funcdefn
  right: (function))

; Function and method calls
;--------------------------

(call_expression
  function: (identifier) @funccall)

(call_expression
  function: (member_expression
    property: (property_identifier) @funccall.method))

; Variables
;----------

(formal_parameters (identifier) @variable.parameter)

(identifier) @variable

; Properties
;-----------

(property_identifier) @property

; Literals
;---------

(this) @variable.builtin
(super) @variable.builtin

(true) @literal.booleana
(false) @cliteral.booleana
(comment) @comment
(string) @literal.string
(regex) @literal.escape
(template_string) @literal.string
(number) @literal.numeric

; Punctuation
;------------

(template_substitution
  "${" @literal.string
  "}" @literal.string) @literal.escape

";" @punctuation.delimiter
"." @punctuation.delimiter
"," @punctuation.delimiter

"--" @operator
"-" @operator
"-=" @operator
"&&" @operator
"+" @operator
"++" @operator
"+=" @operator
"<" @operator
"<<" @operator
"=" @operator
"==" @operator
"===" @operator
"=>" @operator
">" @operator
">>" @operator
"||" @operator

"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket

; Keywords
;----------

"as" @keyword
"async" @keyword
"await" @keyword
"break" @keyword
"case" @keyword
"catch" @keyword
"class" @keyword
"const" @keyword
"continue" @keyword
"debugger" @keyword
"default" @keyword
"delete" @keyword
"do" @keyword
"else" @keyword
"export" @keyword
"extends" @keyword
"finally" @keyword
"for" @keyword
"from" @keyword
"function" @keyword
"get" @keyword
"if" @keyword
"import" @keyword
"in" @keyword
"instanceof" @keyword
"let" @keyword
"new" @keyword
"of" @keyword
"return" @keyword
"set" @keyword
"static" @keyword
"switch" @keyword
"target" @keyword
"throw" @keyword
"try" @keyword
"typeof" @keyword
"var" @keyword
"void" @keyword
"while" @keyword
"with" @keyword
"yield" @keyword
