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
] @keyword

[
 "#include"
 "#define"
 "#if"
 "#ifdef"
 "#ifndef"
 "#else"
 "#elif"
] @preprocessor-keyword

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

(preproc_include
  path: (_) @include-path)

(function_declarator
  declarator: (identifier) @function-definition-name
  )

(ms_call_modifier) @call-convention

(storage_class_specifier) @type-qualifier
(type_qualifier)          @type-qualifier
(attribute_specifier)     @type-qualifier
(ms_declspec_modifier)    @type-qualifier
