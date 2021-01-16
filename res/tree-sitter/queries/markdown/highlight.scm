(atx_heading
 ((atx_heading_marker) @_markdown.h1.marker
  (#eq? @_markdown.h1.marker "#"))) @markdown.h1

(atx_heading
 ((atx_heading_marker) @_markdown.h2.marker
  (#eq? @_markdown.h2.marker "##"))) @markdown.h2

(atx_heading
 ((atx_heading_marker) @_markdown.h3.marker
  (#eq? @_markdown.h3.marker "###"))) @markdown.h3

(atx_heading
 ((atx_heading_marker) @_markdown.h4.marker
  (#eq? @_markdown.h4.marker "####"))) @markdown.h4

(atx_heading
 ((atx_heading_marker) @_markdown.h5.marker
  (#eq? @_markdown.h5.marker "#####"))) @markdown.h5

(atx_heading
 ((atx_heading_marker) @_markdown.h6.marker
  (#eq? @_markdown.h6.marker "######"))) @markdown.h6

(emphasis (text) @markdown.emphasis) @conceal
(strong_emphasis (text) @markdown.strong) @conceal
