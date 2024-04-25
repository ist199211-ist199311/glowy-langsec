#let kind = [Project Proposal]
#let title = [Static Information Flow Analysis of Go]
#let kthblue = rgb("#004791")

#set page("a4", header: {
  set text(10pt)
  smallcaps[#kind: #title]
  h(1fr)
  smallcaps[DD2525 Lang. Sec.]
  line(length: 100%, stroke: 0.5pt + rgb("#888"))
}, footer: {
  set text(10pt)
  line(length: 100%, stroke: 0.5pt + rgb("#888"))
  smallcaps[Diogo Correia & Rafael Serra e Oliveira]
  h(1fr)
  [Page ]
  counter(page).display("1 of 1", both: true)
})
#set par(justify: true)

#align(
  center,
)[
  = #kind: #underline(title)
  == DD2525 Language-Based Security
  ==== #smallcaps(text(fill: kthblue, [KTH Royal Institute of Technology, April 2024]))
  #v(7pt)
  #grid(columns: (20%, 30%, 30%, 20%), [], align(center)[
    Diogo Correia\
    #link("mailto:diogotc@kth.se")
  ], align(center)[
    Rafael Oliveira\
    #link("mailto:rmfseo@kth.se")
  ], [])
  #line(length: 60%, stroke: 1.5pt + black)
  #v(7pt)
]

#set heading(numbering: "1.1.")

= Background

Go is a high-level, compiled, memory-safe programming language created by Google
in 2007 and made public two years later. Nowadays, it has seen widespread
adoption in various fields, for the most part due to its C-like performance and
improved developer experience.

= Project Goals <goals>

Although Go is designed to avoid certain kinds of bugs (e.g., it does not
support pointer arithmetic), it lacks any mechanisms to enforce information flow
security through non-interference. This project's objective is to conceptualize
and implement a conservative *static analyzer* for (a subset of) the Go
language, allowing insecure data interference flows (both implicit and explicit)
to be detected and reported at compile time. Additionally, the tool should
prioritize ease of use and incur zero runtime costs.

= Relevance to Language-Based Security

The proposed static information flow control analyzer works at the language
level, even before Go's compiler is able to produce any executables, preventing
an entire class of bugs and unintended data flows. It is therefore an extension
to the Go environment with the goal of attaining improved security.

= Planned Work Overview

In order to achieve the goals described in @goals, we propose developing a
standalone Rust command-line application capable of parsing Go code and
analyzing both explicit and implicit data flows. Due to time constraints, the
initial prototype would only support a subset of the Go language, and only allow
tracking two security labels (#smallcaps[High] and #smallcaps[Low]), but in any
case special care would be taken during development to facilitate these later
extensions by promoting modularity and loose coupling.

= Preliminary Schedule

- *Week 18 (29#super[th] April - 5#super[th] May):* initial implementation of Go
  parser and AST nodes for variable declaration, assignment, and conditional
  statements;
- *Week 19 (6#super[th] - 12#super[th] May):* initial implementation of the IFC
  analyzer for the aforementioned nodes;
- *Week 20 (13#super[th] - 19#super[th] May):* parser expansion to support more Go
  features, e.g. loops, function calls;
- *Week 21 (20#super[th] - 26#super[th] May):* IFC analyzer expansion to support
  said newly-parsable Go features;
- *Week 22 (27#super[th] May - 2#super[nd] June):* formalization into draft
  project report plus final cleanup; and
- *3#super[rd] June:* project presentation.

= Conclusion

In summary, this project aims to produce a working proof-of-concept for a static
information flow analyzer for the Go language, since such a program does not
seem to exist at the moment within the ecosystem. With this project, we aim to
achieve the grade A, due to the high conceptualization and implementation
complexity it entails and the relevance of the topic for the course.

