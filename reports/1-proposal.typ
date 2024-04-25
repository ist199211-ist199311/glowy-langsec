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

Short description of targeted domain
#lorem(50)

= Project Goals

#lorem(50)

= Relevance to Language-Based Security

#lorem(50)

= Planned Work Overview

#lorem(50)

= Schedule

#lorem(50)

= Conclusion

In summary, #lorem(50)

*target grade and why*
