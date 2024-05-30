#import "template.typ": cover_page, header, footer, setup_page, lined_code

#cover_page(
  title: "Glowy: Information Flow Control of Go Programs", date: "June 2024",
)

#pagebreak()

#show: setup_page

#set page(
  "a4", header: header(title: "Project Draft Report: Glowy (IFC of Go programs)"), footer: footer,
)
#counter(page).update(1)

#let repo = link(
  "https://github.com/ist199211-ist199311/glowy-langsec", [GitHub repository],
)

= Introduction

This report describes the project we developed for the 2024 Spring Semester
edition of the DD2525 Language-Based Security course at KTH Royal Institute of
Technology, which focuses on static analysis of information flow in Go programs.

In terms of structure, @background summarizes the context from which the project
arose, @goals outlines the goals we set out to accomplish, and @overview
describes our solution in detail. Consequently, @results precises our
accomplishments, @discussion presents our reflections on the topic, and finally
@future lists potential improvements and approaches that may be explored in the
future. Additionally, @usage explains how to use the tool and @contribs states
the individual contributions of each group member to this project.

Supporting this report is a #repo containing the Rust source code for the tool,
some examples of annotated Go programs, and additional information relating to
the project.

= Background <background>

== Information Flow Control <bg-ifc>

Information Flow Control (IFC) is a procedure by which one ensures that data
transfers within an information system are not made in violation of a given
security policy. Its aim and implementation differ in accordance with the
intention behind its employment and the threat model under consideration, but in
this project our focus is exclusively on upholding *confidentiality*: the
purpose is to detect and prevent secrets from being exfiltrated into untrusted
contexts, rejecting programs with insecure flows before the latter take place.

In practice, this is measured through the notion of *noninterference*, which
asserts that the values of a (noninterfering) program's public outputs may only
depend on its public inputs, where public is here taken as anything observable
by the attacker under consideration. This means that a public output cannot
depend on any secret input, regardless of whether by means of explicit (e.g.,
`l := h;`) or implicit (e.g., `if (h) l++;`) flows.

IFC is usually realized by either means of static analysis (before the code is
run) or through the injection of a dynamic security monitor capable of mediating
all information transfers at runtime and assessing their legitimacy against a
defined security policy. Although the latter is often more flexible, we hold
that static analysis is for most use cases much more desirable than dynamic
monitoring, since (by definition) it incurs no costs on runtime performance and
can take place just once for each program, rather than every single time one
runs (and for the entire duration of its execution).

== The Go Programming Language <bg-go>

Go is a high-level, compiled, statically typed, memory-safe programming language
created by Google in 2007 and made public two years later. Nowadays, it has seen
widespread adoption in various applications, from backend web development to CLI
programs; for instance, #link("https://github.com/docker", [Docker]),
#link("https://github.com/rclone/rclone", [rclone]),
#link("https://github.com/grafana/grafana", [Grafana]) and
#link("https://github.com/hashicorp/terraform", [Terraform]) are all widely used
production-grade tools written in Go, demonstrating the relevance of the
language.

In addition to its prevalence and rising popularity, Go has several properties
which make it appropriate for static analysis:

- no support for pointer arithmetic, so arbitrary memory accesses are not
  possible, thus allowing for all contact with variables and constants to be
  tracked without evaluating terms;
- native concurrency and message-passing through _goroutines_ and _channels_,
  ensuring that a single unified mechanism is generally used by all programs,
  therefore allowing for focused information flow analysis at said bottleneck;
- explicit error handling, which reduces the possibility of silent failures and
  promotes easier traceability of program execution and data flow; and
- prohibition of circular dependencies in package imports, simplifying the
  dependency graph and making it easier to perform modular analysis.

As such, Go presents itself as a suitable candidate for static analysis of this
nature, in addition to little previous work having been conducted in relation to
the (fairly recent) programming language.

= Project Goals <goals>

In light of the benefits of IFC as explored in @bg-ifc and the Go language
characteristics detailed in @bg-go, we believe that it would be in the best
interest of security to combine these two concepts. Thus, this project's
objective is to conceptualize and implement a proof-of-concept for a
conservative *static analyzer* for (a subset of) the Go language. Our intention
is for this analyzer to parse valid Go programs at compile-time and draw
conclusions on their apparent security (with regard to confidentiality) by
making use of information flow control, specifically through the following
capabilities:

- detect and report explicit flows of information from high-sensitivity sources to
  low-trust sinks;
- likewise, detect and report implicit flows concealed by intermediary operations,
  or even lack thereof when branching on secret;
- provide reasonably fast performance so as to allow this analysis to be included
  as part of a regular build step in, e.g., continuous integration/development
  pipelines or git pre-commit hooks;
- allow some degree of flexibility and customization, exposing a (simple)
  interface for users to shape a security policy in a simple and straightforward
  manner; and
- offer a streamlined user experience by simplifying controls and especially the
  analysis output so that users can easily understand the risks being conveyed and
  how to mitigate them.

We hold that such an audit tool would be invaluable for developers and auditors
alike, allowing its users to reliably find potentially insecure flows and
pinpoint specific high-sensitivity snippets of code that should be manually
reviewed with increased care (i.e., those corresponding to declassification
operations).

As stated above, our focus is on confidentiality, with the analyzer's primary
purpose being the recognition and prevention of unwanted private data
exfiltration. Our *threat model* is as follows: the attacker is in full control
of the Go programs under analysis, which (if accepted) will be run in a trusted
context#footnote[Again, from a confidentiality standpoint; integrity and availability are out of
  scope in this project.]. The attacker can provide arbitrary values as (public)
inputs and may read any public outputs from untrusted sinks, but cannot perform
side-channel engagements such as timing attacks. Fine-grained tuning of what
inputs/sinks are considered trusted or public, and to what degree, is
accomplished through
_code annotations_, as further described in @overview.

Ideally, an interested party need only validate that the (few) annotations in
the source code match the user's expectations, execute the analyzer, and then
read its findings as reported by the tool; this should be sufficient to obtain
some degree of confidence regarding the risks associated with executing the
program, with respect to potentially insecure data transmission flows.

#pagebreak()

= Conceptual Overview <overview>

This project's main contribution is *_Glowy_*, a static analyzer which employs
Information Flow Control techniques to assess Go programs. It aims to fulfill
the goals set forth in @goals, and it is implemented entirely in Rust due to the
strong guarantees offered by the language, in addition to its great developer
experience.

This section describes our implementation at a high level, introducing relevant
concepts and exploring the tool's internal morphology, as well as its public
interface (i.e., how users may control it). More specific instructions on how to
execute it are included in @usage of this report.

== Base Concepts

Glowy categorizes and tracks information by assigning *_labels_* to Go constant
and variable declarations (hereinafter jointly referred to as _bindings_), as
well as to individual expressions. Labels are an abstraction that represent a
specific degree of trust, forming a wide, multidimensional spectrum of
possibilities for a flexible classification.

Concretely, in this implementation a label is a set of _tags_ which uniquely
identify a particular level of confidence within some specific context. For
example, $ell_1 = {"alice"}$ uses a single tag, while
$ell_2 = {"nuclear", "pentagon", "navy"}$ uses several and $ell_3 = {}$ uses
none; their flexibility makes labels suitable for a variety of different
contexts, and overall they are are nevertheless very simple to understand.

In more formal terms, we define a _security lattice_
$cal(L) = (LL, <=, union.sq, sect.sq, top, bot)$, where:

- $LL$ is the set of all possible labels, e.g. ${{}, {A}, {B}, {A, B}}$;
- $<=$ is the (partial) ordering relation between labels, where ${A} <= {A, B}$,
  but ${A} lt.eq.not {B}$;
- $union.sq$ is the operator used to obtain the union of two label, e.g.
  ${A} union.sq {A, B} = {A, B}$;
- $sect.sq$ is, likewise, used to obtain the intersection of two labels, e.g.
  ${A} sect.sq {A, B} = {A}$;
- $top$ is the label at the top of the hierarchy, above all others --- here,
  $top = {A, B}$;
- $bot$ is the label below all others in the ordering --- $bot = {}$.

This means that, essentially, Glowy implements a type checker wherein the types
are labels, i.e., elements of the lattice $cal(L)$. However, although Glowy
analyzes the entire code space to infer how labels evolve over time for
particular chunks of data, the tool has particular interest in two kinds of
instructions, each associated with one end of the flow tracking lifecycle:

- *Sources* are binding declarations that serve as artificial points of origin for
  tracking --- they are associated with an initial label, which is then implicitly
  propagated anywhere the declared binding is used (indicating that the labelled
  information is being transferred there); for example, this may be a constant
  declaration for an `hmacSecret`.
- *Sinks* are particular operations that represent an end-state for information,
  such as a function call to `fmt.Println`, a return statement, or an assignment
  to a newly-declared `output` variable. Sinks are of the upmost importance in
  information flow control, since it is here that certain assertions must take
  place: *an expression may only be passed to a sink if that sink is trusted
  enough to
  receive it*, otherwise it is an insecure flow.

In short, bindings are originally assigned a label $ell_B$, and when the symbols
they define are used in expressions then $ell_B$ influences that expression's
overall label $ell_E$. Over the rest of the program instructions, labels are
adjusted continuously based on what bindings affect them, and eventually a final
expression with label $ell_F$ is passed to a sink labelled $ell_S$. If
$ell_F <= ell_S$, the sink is trusted enough to receive that information, but
otherwise we are in the presence of an insecure program.

#pagebreak()

== Source Code Annotations

Although this is simple to understand in theory, there is still a piece missing:
it is necessary for Glowy to distinguish between what levels of confidentiality
should be assured for each piece of information; in other words, Glowy must know
what label to associate with each _source_ and, likewise, up to what label each
_sink_ is trusted to receive information. Within the intermediary operations of
the information tracking lifecycle it is possible to infer how labels evolve
over time, but those two crucial points at the beginning and at the end of the
process unavoidably require manual intervention to guarantee the desired
flexibility and accuracy, allowing humans to shape a security policy for Glowy
to enforce.

To achieve this, we introduce our only extension over the base Go language
(which is nevertheless compatible with any other Go tools): *_code
annotations_*. These simple, targeted controls take the form of standard Go line
comments following a particular shape:

```go // glowy::SCOPE::{TAG_1, TAG_2, ..., TAG_N}```

Such line comments always refer to the _next_ line#footnote[Technically, to the next suitable operation, if any, before the next semicolon
  (implicit or not).], with the `SCOPE` field indicating what effect the user
desires it to convey. In particular, we define the following `SCOPE`s:

- *`label`:* used to specify what label should be associated with a given _source_,
  e.g.:

  #raw(lang: "go", "// glowy::label::{alice}\nconst keyForAlice = 1234")

  It should be noted that the annotation merely ensures that the binding's label
  conforms to _at least_ what is specified: if the binding's initialization
  expression too requires an incompatible label, then the union of both will be
  used. For example, if

  #raw(lang: "go", "// glowy::label::{bob}\nvar keyForBob = 7 * keyForAlice")

  then the final label associated with `keyForBob` will be
  $ell = {"alice"} union.sq {"bob"} = {"alice", "bob"}$. This ensures that an
  accidental/covert user override is never possible and security is maintained.

- *`sink`:* used to simultaneously indicate (1) that the next operation should be
  considered an end-state, and (2) the extent to which such a _sink_ is trusted to
  receive information, as described above; e.g.,

  #raw(lang: "go", "// glowy::sink::{invoker}\nres.Respond(200, html)")

- *`declassify`:* used to explicitly override the label associated with a
  particular piece of information when it is judged safe to deliberately leak some
  amount of information; e.g.,

  #raw(lang: "go", "// glowy::declassify::{}\nhash := sha256(secret)")

This last `declassify` kind of annotations is potentially the most dangerous, as
it provides an escape hatch to completely disregard any labels that would
otherwise be required to be accommodated to enforce security, and if misused may
defeat the entire purpose of information flow control.

It is deliberately a separate annotation so as to highlight its nature, as
otherwise a `label` annotation might have been used for this behavior, but it
would not be obvious that such a hazardous operation would take place.
Nevertheless, we believe that offering this possibility outweighs the security
risks, since if used sparingly and correctly it offers a great deal of
flexibility that is essential to making Glowy understand and analyze more
complex programs. Moreover, this has the advantage of identifying and placing
special emphasis on the exact points that should be carefully analyzed by human
reviewers designing the intended security policy.

#pagebreak()

== Taint Analysis

Between sources and sinks, Glowy makes use of *taint analysis* to continuously
update labels according to how bindings and expressions are used. Several kinds
of constructs impose requirements of labels that must be met, and the overall
label is the result of union of all such requirements. For example, as already
shown,

```go
// glowy::label::{alice}
a := 3

// glowy::label::{bob}
b := 9 + 2 * a // <-- will have label {alice, bob}
```

Since `b`'s initialization expression uses `a` directly, Glowy requires that
`b` is _tainted_ with `a`'s label (${"alice"}$), and the source annotation
further requires ${"bob"}$, so the final label is their union. In other words,
if `b` is leaked, an attacker might be able to extract `a`'s value, so Glowy
must remember that `b` cannot be passed to any sink not trusted to know `a`.

Moreover, although the example above showed an explicit flow from `a` to `b`,
the following one is not as straightforward:

```go
// glowy::label::{charlie}
c := true

out := true // <-- label {} (bottom)
if !c {
  out = false // <-- label updated to {charlie}
}
```

Despite there not being a direct assignment of `c` to `out`, there is an
implicit flow of information between them, since the execution flow branches
based on the value of `c`. Glowy tracks all of these cases, tainting symbols and
expressions even if they are only implicitly affected by others; finally, when
they are passed to a sink, a judgement is made to assess whether the accumulated
label is compatible with the security policy defined by the code annotations.

If any insecure flow is present, Glowy will immediately take notice and flag it
for human review, so that the risk may be examined and a course of action
determined (either by refactoring the code to eradicate the data flow, or by
explicitly accepting the danger through a `declassify` annotation instructing
Glowy to ignore it). For instance, given the Go program:

```go
package main

// glowy::label::{sensitive}
const secret = 123

func opaque(seed int) int {
  if (seed + secret) % 10 == 0 {
    return 5
  } else {
    return 7
  }
}

func main() {
  // glowy::sink::{}
  fmt.Println(4 * opaque(2))
}
```

there is an evident leak of information present, which results in Glowy
outputting the error as follows:

#show raw.where(block: true): set par(justify: false)
```
error[E002]: insecure data flow to sink during function call
   ┌─ main.go:16:7
   │
 4 │ const secret = 123
   │       ------ symbol `secret` has been explicitly annotated with label {sensitive}
   ·
 7 │   if (seed + secret) % 10 == 0 {
   │              ------ symbol `secret` in branch has label {sensitive}
 8 │     return 5
   │     -------- function returns with label {sensitive}
   ·
16 │   fmt.Println(4 * opaque(2))
   │       ^^^^^^^     ------ function call to `opaque` has return value with label {sensitive}
   │       │
   │       sink `Println` has label {}, but the expression being assigned to it has label {sensitive}
```

#pagebreak()

== Algorithmic Outline

From a birdseye perspective, Glowy is implemented as follows:

+ Receive each file as a command-line argument, or read Go directly from standard
  input.
+ Parse each file using the custom standalone parser to obtain an Abstract Syntax
  Tree (AST):
  - A custom lexer reads the file source code and provides a peekable token stream
    that a hierarchy of parsing functions makes use of to identify Go constructs and
    translate them into more structured AST nodes, for the main Glowy analyzer to
    use.
  - Annotations (using the generic form above to allow future compatibility with new
    `SCOPE`s) are parsed too, and encoded in the relevant AST nodes.
+ Create an empty Symbol Table $T$ to record bindings and their respective labels
  as they evolve.
+ Push a new scope $s_0$ onto $T$'s stack to hold global bindings.
+ Visit each non-function top-level declaration, registering its label in $s_0$ --- see
  below.
+ Initialize a queue $Q$ of _abstract calls_ to process, starting with $Q = {"main()"}$:
  - Abstract calls are signatures holding a function identifier (i.e., its name) and
    a mapping between parameters and labels; for example, $f({"alice"}, {})$ would
    be different from $f({"bob"}, {"alice"})$ or from $g({"alice"}, {})$.
  - Using these structures allows us to reason about identical function calls only
    once, while still distinguishing different kinds of calls and supporting complex
    situations such as mutually recursive functions.
  - The `main` function (taking no arguments) is the primary entry-point in Go.
+ While there is an abstract call $f(ell_1, ell_2, dots.c, ell_k)$ in the queue
  (i.e., while $Q eq.not emptyset$):
  + Push a new scope $s'$ onto the $T$'s stack to hold local bindings.
  + Map in $s'$ each parameter name to its respective label $ell_i$ as passed.
  + Visit the statements in $f$'s body as described below, using $s'$ as the current
    scope.
  + Discard the $s'$ scope from $T$'s stack, since it is no longer necessary.
  + If a return statement was found, map $f$ to that expression's label $ell_R$
    in $T$'s current scope $s (!= s')$.
  + Otherwise, map $f$ to $bot$ (i.e., ${}$).
+ If no errors have been propagated back in the meantime, report that the program
  appears secure.

When visiting AST nodes (sequentially), whether as part of top-level
declarations or when processing abstract function calls from $Q$, the following
applies:
- Store the current branch's label $ell_lt.curly$ and union it with the label of
  any expression on whose value the execution flow branches.
- Whenever an operation such as a declaration, assignment, or increment takes
  place, union the target binding's label with $ell_lt.curly$.
- Whenever a function call is found, determine its arguments' labels and enqueue
  the abstract call in $Q$ to be processed (if not already present). Assume, for
  the present expression, that the label of the function's return value is the
  union of all the arguments' labels.
- If a sink is found, report an insecure flow if $ell_E lt.eq.not ell_S$, where
  $ell_E$ is the label calculated for the expression (or the union of all
  expressions', if multiple) being passed to a sink annotated with $ell_S$.

This mechanism even supports, as already stated, mutually recursive functions:
they will be visited multiple times until their labels stabilize.

== Concurrency and Message-Passing

As stressed in @bg-go, Go includes native techniques for concurrency and
message-passing, implemented using _goroutines_ and _channels_. This makes it
relatively simple to extend our solution to support tracking information flow
across channels, since all data transfers are centralized in send statements and
receive expressions.

Given that all messages between _goroutines_ are sent through channels, Glowy
simply has to keep track of each of the latter and taint it when a piece of
information with a certain label is sent through it. Then, all the data read
from that channel will have the same security label as the channel itself.
However, since usage of the channel can be spread throughout the Go code, it
might be necessary to visit functions multiple times until the labels stabilize.

= Results <results>

It is our understanding that Glowy meets our initial objectives and accomplishes
the goals presented in @goals, as it supports a significant fraction of Go
constructs and various kinds of flows, both implicit and explicit. Although it
is, at the present stage, intended as a proof-of-concept and a prototype, with
several possible improvements still evident (see @future), it is nevertheless
already a powerful tool.

Glowy's design is tailored to scale easily for larger code bases and especially
to accompany their growth over time: written in Rust and with few bottlenecks,
the analyzer's performance makes it suitable to be included as part of a build
step or test suite, ensuring that security does not degrade. Moreover, it scales
much better than manual review since humans only need to review code annotations
(which there should be relatively few of due to the tainting and label inference
rules) and the tool's output if any risks are detected and reported.

Furthermore, Glowy is extensible: although its small command-line application
binary is what is most often described in this report for simplicity, Glowy is
provided first and foremost as a library, which can be used by other Rust
programs very easily. The Go parser was also implemented as another standalone
library, and though focused for Glowy's IFC purposes, it is reasonably agnostic
and can be used for other kinds of analysis (or even to implement a compiler).

Glowy is free and open-source, with its source code being released under the MIT
license, so anyone may extend our work for more complex or more specific tasks
as they see fit, such as to implement a more generic tool that simply tracks
what is affected by what without a particular focus on security.

= Discussion <discussion>

Although Glowy is a very useful security analyzer, as already mentioned, its
weakest link is still evidently the instructions it is given: Glowy will blindly
trust its user, so if the security policy to be enforced is not adequately set
or expressed, it can render the entire analysis moot. Concretely, this means
that incorrect or unintended source code annotations can have very drastic
effects on the tool's efficacy, even if they appear to be small mistakes.

Moreover, despite Glowy being helpful for its efficiency and presumed accuracy,
its very nature may lead humans to become overly confident in its ability, and
especially in that it is working correctly. Incorrect annotations, or even
implementation bugs, can give its users a false sense of security that may have
effects worse than if it was not used at all. Nevertheless, we believe that this
is not likely to happen with sufficient frequency or severity that it justifies
avoiding the tool's usage, given the great benefits already discussed: it is
simply a matter of remaining vigilant, and especially ensuring that all code
annotations are correct.

Finally, as repeatedly pointed out across this report, Glowy focuses exclusively
on confidentiality: a program deemed to be secure by the analyzer to run in a
trusted context may still be perfectly capable of exploiting other properties
such as integrity and availability, e.g., it may execute a shell command
`rm -rf /data` without the analyzer reporting it.

#pagebreak()

= Future Work <future>

(draft)

- full language support
- formal verif
- better cross-dependency handling
- boolean short-circuiting (e.g., if second operation is a function)

= Conclusion

In summary, we have conceptualized and implemented a Go information flow control
static analyzer and enforced, written in Rust, which can prove to be a powerful
tool for various kinds of users to verify that certain programs work as intended
and do not leak information except as permitted by the defined security policy.
These results have been further summarized in @results and discussed in
@discussion.

#pagebreak()

#set heading(numbering: "A.", supplement: "Appendix")
#counter(heading).update(0)

= Glowy: Tool Usage Instructions <usage>

#lorem(100) readme?

In order to analyze a Go source file using Glowy, one need only:

+ Obtain the tool's source code from the #repo:

  ```bash git clone git@github.com:ist199211-ist199311/glowy```

+ Compile Glowy:

  ```bash cargo build --release```

+ Optionally create a convenience link to the binary in the root directory or
  somewhere on the `$PATH`:

  ```bash ln -s target/release/glowy ./glowy```

+ Annotate the target `.go` files with line comments specifying what source and
  sink label constraints should be enforced, e.g.:

  // I hate typstfmt
  #raw(
    lang: "go", "// (...)\n\n// glowy::label::{high}\nconst secret = \"hunter12\"\n\n// (...)\n\n// glowy::sink::{}\nfmt.Println(result)",
  )

+ Analyze the annotated source file:

  ```bash ./glowy path/to/file.go```

Alternatively, Glowy can be compiled and run directly using ```bash cargo run
--release path/to/file.go```.

#pagebreak()

= Group Member Contributions <contribs>

Rafael implemented the parser, while the analyzer itself was implemented by both
parties (with a slight majority of it being attributable to Diogo). The initial
conceptualization was started together, primarily explored and developed by
Diogo, and iterated on by Rafael.

The present report was mainly written by Rafael, with Diogo's guidance for some
parts that he was more familiar with.

