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

Go is a high-level, compiled memory-safe programming language
created by Google in 2007 and made public two years later.
Nowadays, is has seen widespread adoption in various applications,
such as backend web development, CLI applications and services and more,
due to its C-like performance and good developer experience.

= Project Goals

However, the Go language lacks any kind of information flow control
mechanisms.
This project's goal is to implement these mechanisms for the Go language
through a standalone application that can be ran over Go source code
at compile time, reporting any violations of the information flow
policies (?).

= Relevance to Language-Based Security

The proposed static information flow control analyser works at the
language level, even before Go's compiler is able to produce any
executables, preventing a whole class of bugs and unintended data flows.

= Planned Work Overview

To achieve the goal of this project, a Rust program will be created that
is able to parse Go code and then analyse it the information flow of
certain variables.
Due to time constrains, only a subset of the Go language will be supported.
For the same reason, the initial version of this program will only allow
low and high variables, but multiple labels support will be accounted
for during development.

= Schedule

- *Week 18 (29#super[th] April - 5#super[th] May):* Initial implementation
  of Go parser and AST nodes for variable declaration and conditional
  statements;
- *Week 19 (6#super[th] - 12#super[th] May):* Initial implementation of
  IFC analyser for supported AST nodes;
- *Week 20 (13#super[th] - 19#super[th] May):* Expansion of parser to
  more nodes, such as loops, function calls, etc.;
- *Week 21 (20#super[th] - 26#super[th] May):* Expansion of IFC analyser
  to support the AST nodes added in the previous week;
- *Week 22 (27#super[th] May - 2#super[nd] June):* Elaboration of draft
  project report and final cleanup;
- *3#super[rd] June:* Project presentation.

= Conclusion

In summary, this project aims to produce a working proof-of-concept
for a static information flow analyser for the Go language,
since such program seems to not exist to the best of our knowledge.
With this project, we aim to achieve the grade A, due to the high
implementation complexity it entails and the relevance of the topic
for the course.
