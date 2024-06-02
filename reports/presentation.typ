#import "@preview/polylux:0.3.1": *
#import themes.clean: *

// compile .pdfpc wth `polylux2pdfpc {fname}.typ`
// then present with, e.g., `pdfpc --windowed both {fname}.pdf`

// uncomment to get a "submittable" PDF
// #enable-handout-mode(true)

#let kthblue = rgb("#000060")
#show: clean-theme.with(
  short-title: [*DD2525 LangSec (Group 1) |* Glowy: IFC of Go Programs], color: kthblue, logo: image("assets/KTH_logo_RGB_bla.svg"),
)

#pdfpc.config(duration-minutes: 8)

// consistent formatting + make typstfmt play nice
#let notes(speaker: "???", ..bullets) = pdfpc.speaker-note("## " + speaker + "\n\n" + bullets.pos().map(x => "- " + x).join("\n"))

#show link: it => underline(stroke: 1pt + kthblue, text(fill: kthblue, it))

#let focus = it => text(kthblue, strong(it))

#let cover = title-slide(
  title: [Glowy: Information Flow Control of Go Programs], subtitle: [
    *DD2525 Language-Based Security*

    #smallcaps[KTH Royal Institute of Technology]

    Monday, June 3#super[rd], 2024

    #notes(
      speaker: "Raf", "Introduce group members", "Present our LangSec project, starting with our established goals",
    )
  ], authors: ([Diogo Correia\
    #link("mailto:diogotc@kth.se")], [*Group 1*], [Rafael Oliveira\
    #link("mailto:rmfseo@kth.se")]),
)

#cover

#new-section-slide("Project Goals")

#slide(
  title: "Background: Information Flow Control",
)[
  - Ensures data transfers don't violate some security policy

  - Focus on *confidentiality*

  - *Noninterference:* public outputs only depend on public inputs

  - Static analysis >> dynamic monitoring

  #notes(
    speaker: "Diogo", "THIS PROJECT focuses EXCLUSIVELY on upholding **confidentiality**: purpose is to detect and prevent secrets from being exfiltrated into untrusted contexts", "Public here means anything observable by an attacker", "Static incurs no costs on runtime performance",
  )
]

#slide(
  title: "Background: The Go Programming Language",
)[
  #side-by-side([
    - High-level, compiled, statically typed, & memory-safe

    - Widespread adoption across different domains
      - e.g. Docker, caddy

    - Some characteristics suitable for static analysis
  ], align(center + horizon, image("assets/go.png")))

  #notes(
    speaker: "Diogo", "Google 2007, released 2009", "Besides prevalence and rising popularity, characteristics:\n\t- No pointer arithmetic\n\t- Native concurrency / message-passing",
  )
]

#slide(
  title: "Project Goals",
)[
  - *Static analyzer* for (a subset) of the Go language

  - Desired properties:
    - #text(
        fill: kthblue,
      )[*Capable of detecting & reporting both explicit and implicit flows*]
    - Reasonably fast performance (e.g., CI/CD pipelines)
    - Flexibility to easily shape a custom security policy
    - Streamlined UX with simple controls and output

  - Threat model: attacker controls programs & public info (but no side-chan.s)

  #notes(
    speaker: "Raf", "Objective: conceptualize + implement a proof-of-concept for a static analyzer", "Parse valid Go programs at compile-time and draw conclusions on their apparent security wrt confidentiality", "----", "Attacker can provide arbitrary public inputs and read any public outputs",
  )
]

#new-section-slide("Developed Solution")

#slide(
  title: "Glowy",
)[
  - Static analyzer employing IFC techniques to assess Go programs

  - Written in Rust from scratch, including (spec-compliant) Go lexer/parser

  - Information categorized with *labels*
    - e.g., ${"high"}$ or ${"nuclear", "pentagon", "navy"}$
    - Accumulate over time

  #notes(speaker: "Diogo")
]

#slide(
  title: "Tracking Lifetime & Code Annotations",
)[
  #side-by-side[
    - *Sources:* artificial points of origin

      #raw(lang: "go", "// glowy::label::{secret}\nconst hmacSecret = 0x4D757342")

      #pause
    - *Sinks:* end-state for information

      #raw(lang: "go", "// glowy::sink::{}\nfmt.Println(output)")

      *Insecure flow if $ell_E lt.eq.not ell_S$*

      #pause
  ][
    - *Declassification:* always explicit

      #{
        set block(stroke: ("left": 4pt + red), inset: ("left": 1em, "y": 5pt))
        raw(
          lang: "go", "// glowy::label::{alice}\nvar a int = 3\n// glowy::label::{bob}\nb := 4 * a // {alice, bob}", block: true,
        )
      }

      #{
        set block(stroke: ("left": 4pt + blue), inset: ("left": 1em, "y": 5pt))
        raw(
          lang: "go", "// glowy::declassify::{}\nhash := sha256(secret)", block: true,
        )
      }
  ]

  #notes(
    speaker: "Raf", "Explicit declassification creates very identifiable points in the code that need to be looked at very carefully",
  )
]

#slide(
  title: "Taint Analysis",
)[
  #side-by-side[
    - *Explicit Flows*

      #raw(
        lang: "go", "// glowy::label::{alice}\na := 3\n// glowy::label::{bob}\nb := 7\n\na += 9 + 2 * b\n// ^ final label {alice, bob}",
      )

    #pause
  ][
    - *Implicit Flows*

      #raw(
        lang: "go", "// glowy::label::{charlie}\nc := 98\n\nout := true // label {}\nif c % 10 == 0 {\n\tout = false\n\t// ^ new label {charlie}\n}",
      )
  ]

  #notes(speaker: "Raf")
]

#slide(title: "Example Output")[
  #grid(columns: (auto, auto), column-gutter: 1em, [
    #set text(size: 15pt)
    #raw(lang: "go", read("assets/opaque.go"))
  ], image("assets/opaque.png", height: 85%))

  #notes(speaker: "Diogo", "Read output bottom-up")
]

#slide(
  title: "Functions: Synthetic Tags",
)[
  #side-by-side(
    raw(lang: "go", read("assets/synthetic-foo.go")), raw(lang: "go", read("assets/synthetic-main.go")),
  )

  #notes(
    speaker: "Diogo", "If parameter `a` was never used by `foo`, b would not be tainted by `{lbl2}`",
  )
]

#new-section-slide("Results")

#slide(
  title: "Results",
)[
  - Goals achieved

  - Powerful auditing tool despite PoC/prototype

  - Human review: just annotations + output

  - Future work:
    - Full language support
    - Better multi-package support

  #notes(
    speaker: "Raf", "We believe Glowy accomplishes the objectives we established",
  )
]

#cover
