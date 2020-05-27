; Identifier naming conventions

((identifier) @constructor
 (#match? @constructor "^[A-Z]"))

((identifier) @constant
 (#match? @constant "^[A-Z][A-Z_]*$"))

; Function calls

(decorator) @funccall

(call
  function: (attribute attribute: (identifier) @funccall.method))
(call
  function: (identifier) @funccall)

; Builtin functions

((call
  function: (identifier) @funccall.builtin)
 (#match?
   @funccall.builtin
   "^(abs|all|any|ascii|bin|bool|breakpoint|bytearray|bytes|callable|chr|classmethod|compile|complex|delattr|dict|dir|divmod|enumerate|eval|exec|filter|float|format|frozenset|getattr|globals|hasattr|hash|help|hex|id|input|int|isinstance|issubclass|iter|len|list|locals|map|max|memoryview|min|next|object|oct|open|ord|pow|print|property|range|repr|reversed|round|set|setattr|slice|sorted|staticmethod|str|sum|super|tuple|type|vars|zip|__import__)$"))

; Function definitions

(function_definition
  name: (identifier) @funcdefn)

(identifier) @variable
(attribute attribute: (identifier) @property)
(type (identifier) @type)

; Literals

(none) @literal.numeric
(true) @literal.boolean
(false) @literal.boolean

(integer) @literal.numeric
(float) @literal.numeric

(comment) @comment
(string) @literal.string
(escape_sequence) @literal.escape

(interpolation
  "{" @punctuation.bracket
  "}" @punctuation.bracket) @literal.escape

; Tokens

"-" @operator
"-=" @operator
"!=" @operator
"*" @operator
"**" @operator
"**=" @operator
"*=" @operator
"/" @operator
"//" @operator
"//=" @operator
"/=" @operator
"&" @operator
"%" @operator
"%=" @operator
"^" @operator
"+" @operator
"+=" @operator
"<" @operator
"<<" @operator
"<=" @operator
"<>" @operator
"=" @operator
"==" @operator
">" @operator
">=" @operator
">>" @operator
"|" @operator
"~" @operator
"and" @operator
"in" @operator
"is" @operator
"not" @operator
"or" @operator

; Keywords

"as" @keyword
"assert" @keyword
"async" @keyword
"await" @keyword
"break" @keyword
"class" @keyword
"continue" @keyword
"def" @keyword
"del" @keyword
"elif" @keyword
"else" @keyword
"except" @keyword
"exec" @keyword
"finally" @keyword
"for" @keyword
"from" @keyword
"global" @keyword
"if" @keyword
"import" @keyword
"lambda" @keyword
"nonlocal" @keyword
"pass" @keyword
"print" @keyword
"raise" @keyword
"return" @keyword
"try" @keyword
"while" @keyword
"with" @keyword
"yield" @keyword
