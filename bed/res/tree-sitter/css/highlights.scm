(comment) @comment

(tag_name) @tag
(nesting_selector) @tag
(universal_selector) @tag

"~" @operator
">" @operator
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"=" @operator
"^=" @operator
"|=" @operator
"~=" @operator
"$=" @operator
"*=" @operator

"and" @operator
"or" @operator
"not" @operator
"only" @operator

(attribute_selector (plain_value) @literal.string)
(pseudo_element_selector (tag_name) @decorator)
(pseudo_class_selector (class_name) @decorator)

(class_name) @decorator
(id_name) @decorator
(namespace_name) @decorator

(property_name) @variable.builtin
(feature_name) @variable.builtin

(attribute_name) @attribute

(function_name) @function

((property_name) @variable
 (#match? @variable "^--"))
((plain_value) @variable
 (#match? @variable "^--"))

"@media" @keyword
"@import" @keyword
"@charset" @keyword
"@namespace" @keyword
"@supports" @keyword
"@keyframes" @keyword
(at_keyword) @keyword
(to) @keyword
(from) @keyword
(important) @keyword

(string_value) @literal.string
(color_value) @literal.string

(integer_value) @literal.number
(float_value) @literal.number
(unit) @type

"#" @punctuation.delimiter
"," @punctuation.delimiter
