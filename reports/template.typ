#let kthblue = rgb("#2258a5")
#let stylize_link(it) = if type(it.dest) == "string" {
  underline(stroke: 1pt + kthblue, text(fill: kthblue, it))
} else {
  underline(text(it))
}

#let cover_page(title: none, date: none) = {
  set page("a4")
  set text(12pt)
  show link: stylize_link
  show heading: set block(above: 1.4em, below: 1em)

  align(center + horizon)[
    #image("assets/KTH_logo_RGB_bla.svg", height: 100pt)

    #v(20pt)

    #heading(outlined: false)[#title]

    #heading(outlined: false, level: 2)[DD2525 Language-Based Security]

    #v(20pt)

    *Group 1*

    #grid(columns: (50%, 50%), align(center)[
      Diogo Correia\
      #link("mailto:diogotc@kth.se")
    ], align(center)[
      Rafael Oliveira\
      #link("mailto:rmfseo@kth.se")
    ])

    #v(20pt)

    #smallcaps[#date]

    KTH Royal Institute of Technology
  ]
}

#let header(title: none, authors: []) = {
  set text(10pt)
  smallcaps(title)
  h(1fr)
  smallcaps[DD2525 LangSec VT24]
  line(length: 100%, stroke: 0.5pt + rgb("#888"))
}

#let footer = {
  set text(10pt)
  line(length: 100%, stroke: 0.5pt + rgb("#888"))
  smallcaps[Group 1: Diogo Correia & Rafael Oliveira]
  h(1fr)
  [Page ]
  counter(page).display("1 of 1", both: true)
}

#let setup_page(content) = {
  show link: stylize_link
  set par(justify: true)
  set heading(numbering: "1.1.")

  content
}

#let lined_code(caption: none, starting_from: 0, code) = {
  figure({
    show raw.line: it => {
      text(fill: gray, [#(it.number + starting_from)])
      h(1em)
      it.body
    }

    code
  }, caption: caption)
}
