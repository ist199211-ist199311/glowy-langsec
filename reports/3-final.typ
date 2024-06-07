#import "@preview/ansi-render:0.6.1": ansi-render, terminal-themes
#import "template.typ": cover_page, header, footer, setup_page, lined_code

#cover_page(
  title: "Glowy: Information Flow Control of Go Programs", date: "June 2024",
)

#pagebreak()

#show: setup_page

#set page(
  "a4", header: header(title: "Project Report: Glowy (IFC of Go programs)"), footer: footer,
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
achievements, @discussion presents our reflections on the topic, and finally
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
  therefore allowing for focused information flow analysis at said bottlenecks;
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

We hold that such a verification tool would be invaluable for developers and
auditors alike, allowing its users to reliably find potentially insecure flows
and pinpoint specific high-sensitivity snippets of code that should be manually
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

Concretely, in this implementation, a label is a set of _tags_ which uniquely
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
  particular piece of data when it is judged safe to deliberately leak some amount
  of information about it; e.g.,

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
complex programs, as some policies may allow for minimal information leakage.
Moreover, this has the advantage of identifying and placing special emphasis on
the exact points that should be carefully analyzed by human reviewers designing
the intended security policy.

#pagebreak()

== Taint Analysis

Between sources and sinks, Glowy makes use of *taint analysis* to continuously
update labels according to how bindings and expressions are used. Several kinds
of constructs impose requirements of labels that must be met, and the overall
label is the result of the union of all such requirements. For example, as
already shown,

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
based on the value of `c`. This is taken into account even in branching with
multiple alternative paths, where the final label is taken to be the union of
the ones resulting from all alternatives. For instance,

```go
// glowy::label::{one}
x := 1
// glowy::label::{two}
y := 2

// glowy::label::{three}
z := 3 // <-- this starts with label {three}
if check() {
  z += x // non-simple assignment: in this path, label becomes {one, three}
} else {
  z = y // simple assignment: in this path, label is overwritten to {two}
}

// glowy::sink::{two, three}
return z // <-- REJECTED: z has label {one, two, three}
```

Although ordinarily a simple assignment would override the binding's label
entirely, in this case this cannot be taken as true just in order of node
visitation, since otherwise the above example's `else`-clause would erase all
traces of ${"one", "three"}$. Instead, Glowy remembers that several alternative
paths of execution are possible and thus `z`'s final label is the union of the
ones originating from each path.

Glowy tracks all of these cases, tainting symbols and expressions even if they
are only implicitly affected by others; finally, when they are passed to a sink,
a judgement is made to assess whether the accumulated label is compatible with
the security policy defined by the code annotations.
#pagebreak()

If any insecure flow is present, Glowy will immediately take notice and flag it
for human review, so that the risk may be examined and a course of action
determined (either by refactoring the code to eradicate the data flow, or by
explicitly accepting the danger through a `declassify` annotation instructing
Glowy to ignore it). For instance, given the Go program:

```go
package main

import "fmt"

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

#ansi-render(
  read("assets/opaque.ansi"), font: none, size: 0.95em, inset: 10pt, radius: 5pt, theme: terminal-themes.one-half-dark,
)

This output can be read bottom-up to clearly identify the problem and then trace
the exact reasoning causing Glowy to detect this as an insecure flow. This means
that any user reading the error can easily understand what violation is being
reported and, starting from the end of the trace, read upwards as much as
necessary until they understand the underlying problem.

As stressed in @goals, making the analysis output as intuitive and concise as
possible, while still sufficiently descriptive, was a significant priority,
since in the end Glowy's usefulness is proportional to how user-friendly and
easy to control it presents itself, especially to people without a security
background. We believe that this approach highlights the key points on which
attention should be focused on, accomplishing our user-experience objectives.

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
+ Maintain a mapping of reverse dependencies $D$, initially empty.
+ Initialize an empty symbol queue $Q$ of global declarations to process, starting
  with $Q = emptyset$.
+ Visit each top-level declaration, registering it in $s_0$ with an initial label
  of $bot$, and adding it to $Q$.
  - All errors found during this stage are ignored, as they may stem from yet
    incorrect information.
+ While there is a global symbol $sigma$ in the queue (i.e., while $Q eq.not emptyset$),
  visit $sigma$'s declaration as follows:
  - Hierarchically and sequentially visit each node in the declaration's tree,
    ignoring too during this stage any errors found.
  - Store the current branch's label $ell_lt.curly$ and union it with the label of
    any expression on whose value the execution flow branches.
  - Whenever any operation takes place, such as a declaration, assignment, or
    increment, union the target binding's label with $ell_lt.curly$.
  - Whenever an operand name expression is found (i.e., when another symbol $sigma'$ is
    used), remember in $D$ that $sigma$ depends on $sigma'$.
  - If $sigma$ is a function:
    + Push a new scope $s'$ onto $T$'s stack to hold local bindings.
    + Add symbols $"arg"_i$ to $s'$ to represent arguments and map them to labels
      composed of _synthetic tags_ such that $s'("arg"_i) = {"<i>"}$.
    + Visit the statements in the function's body as normal, using $s'$ as the current
      scope.
    + Discard the $s'$ scope from $T$'s stack, since it is no longer necessary.
    + If a return statement was found, map $sigma$ to its label $ell_R$
      in $T$'s current scope $s (!= s')$.
    + Otherwise, map $sigma$ to $bot$ (i.e., ${}$).
  - Whenever a function call to a symbol $sigma_f$ is found, replace $ell_f = T(sigma_f)$'s
    synthetic tags with the passed argument's concrete tags to obtain a concrete
    label corresponding to the call expression.
  - If $sigma$'s label has changed while its declaration was visited (in this
    iteration), enqueue all symbols $sigma'$ that depend on it (per $D$) in $Q$ to
    be processed again.
+ After $Q = emptyset$ and so all labels have stabilized, visit the entire tree
  one last time to collect errors.
  - Specifically, if a sink is found, report an insecure flow if $ell_E lt.eq.not ell_S$,
    where $ell_E$ is the label calculated for the expression (or the union of all
    expressions', if multiple, in addition to the current branch label $ell_lt.curly$)
    being passed to a sink annotated with $ell_S$.
+ If no errors are found, report that the program appears secure.

This mechanism supports complex inter-dependent constructs, including even
mutually recursive functions: they will be visited multiple times until their
labels stabilize. Additionally, the use of synthetic labels for functions is
simple yet ingenious and proves a powerful abstraction to deal with unknown
values that will eventually be mapped to concrete labels.

#pagebreak()

== Concurrency and Message-Passing

As stressed in @bg-go, Go includes native techniques for concurrency and
message-passing, implemented using _goroutines_ and _channels_. This makes it
relatively simple to extend our solution to support tracking information flow
across channels, since all data transfers are centralized in a combination of
send statements (`ch <- exp`) and receive expressions (`<-ch`).

Given that all messages between _goroutines_ are sent through channels, Glowy
simply has to keep track of each of the latter and taint it when a piece of
information with a certain label is sent through it. Subsequently, all the data
read from that channel will have the same security label as the channel itself.
However, since usage of the channel can be spread throughout the Go code, it
might be necessary to visit functions multiple times until the labels stabilize,
treating channel arguments as mutable.

For example, in the following program `ch` is, in the end, associated with
${"medium", "high"}$:

```go
package main

import "fmt"

// glowy::label::{high}
const secret = 42

func main() {
    var ch = make(chan int)

    var x = 7
    var y = 3
    // glowy::label::{medium}
    var z = 2

    go runner(x, ch)
    go runner(y, ch)
    go runner(z, ch)

    // glowy::sink::{}
    fmt.Println(<- ch) // <-- REJECTED
    // glowy::sink::{medium}
    fmt.Println(<- ch) // <-- REJECTED
    // glowy::sink::{medium, high}
    fmt.Println(<- ch) // <-- ACCEPTED
}

func runner(a int, ch chan int) {
    if expensiveComputation(a) == 16 {
      ch <- secret // (will also depend on `a`'s label, since we branched)
    } else {
      ch <- 0
    }
}

func expensiveComputation(a int) int {
  return a * a
}
```

It should be noted that we cannot keep track of the order under which labelled
data is sent into the channel and tie it with the order in which data is
received, since _goroutines_ can execute concurrently and so in no particular
order.

#pagebreak()

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
much better than manual review since humans only need to validate code
annotations (which there should be relatively few of due to the tainting and
label inference rules) and the tool's output if any risks are detected and
reported.

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
or expressed, this can render the entire analysis moot. Concretely, this means
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

In terms of scalability, we are confident that if used from the beginning Glowy
can easily accompany a project's growth without significant development overhead
(and the tool itself would likely cope well with the size too, given its
remarkable performance). However, it may not be as simple to suddenly adapt an
existing large code base to make full use of Glowy's features: sources and sinks
must be identified and annotated with labels, which can be easy in more
straightforward code with few secrets but harder for otherwise more complex
flows of many kinds of private information. In any case, the very act of
labelling sources and sinks (in addition to, perhaps, declassification
operations) is a useful exercise insofar as it forces one to consciously
consider the secrets at hand and how they may be processed; in this sense, Glowy
can be said to bring a positive effect even before starting its analysis.

Finally, as repeatedly pointed out across this report, Glowy focuses exclusively
on confidentiality: a program deemed to be secure by the analyzer to run in a
trusted context may still be perfectly capable of exploiting other properties
such as integrity and availability, e.g., it may execute a shell command
`rm -rf /data` without the analyzer reporting it.

#pagebreak()

= Future Work <future>

Despite being just a prototype, Glowy is already a very promising audit tool, as
highlighted in @results. Nevertheless, it is unquestionably still not ready for
production-grade usage, and there are various features that could be explored in
the future to make the tool more useful and effective, as well as more suitable
for broader purposes.

Firstly and most importantly, it is paramount to work towards full language
support: Glowy does not presently support all Go constructs, such as arrays or
nested function declarations, nor does it support more complex types or methods.
This is the primary obstacle preventing real-world testing on true code bases,
but it can be surpassed somewhat simply, given a great deal of implementation
work dedicated.

Secondly, and tied to the first one, cross-dependency handling comprises
essential functionality that is to be expected when analyzing larger programs
with, undoubtedly, several dependencies that should too be subject to analysis.
Glowy has limited support for this, as all symbols are scoped with a package
name, but it does not allow for real multi-package information flow control.

Moreover, it would be ideal to further extend Glowy's test suite so that there
is a continuous baseline of what constructs are supported and how the expected
behavior is characterized. Although many low-level components already count with
unit tests (e.g., for lexing, parsing, and label set operations), the taint
analysis itself is not currently covered by automatic tests. This addition would
prevent regressions and promote the tool's overall robustness over time, thus
proving a valuable contribution.

Finally, another way to ensure Glowy's robustness would be to perform a
mathematical study with the goal of faithfully modelling its implementation and
proving that it works correctly in accordance with its claims, in the face of
any (supported) scenario. This kind of formal verification, or another with
similar consequences, would greatly increase users' trust in the tool and
strengthen auditors' confidence in that it is suitable for real-world usage.

We believe that these developments, if accomplished, would make Glowy into a
very powerful analyzer. Unfortunately, however, we were unable to make further
progress on them, primarily due to lack of time, given this project's short time
span. It is possible that we may continue to work on this in the future, but
otherwise (as stated above), the license under which the source code is released
permits anyone to extend our work as they see fit, with few limitations.

= Conclusion

In summary, we have conceptualized and implemented a Go information flow control
static analyzer and enforcer, written in Rust, which can prove to be a powerful
tool for various kinds of users to verify that certain programs work as intended
and do not leak information except as permitted by the defined security policy.

These results have been further summarized in @results and discussed in
@discussion, but overall we are confident that this project has been greatly
successful and has accomplished our goals, even though it is far from a
finished, production-grade product, with some potential improvements and next
steps being outlined in @future.

As previously mentioned, @usage of this report explains in detail how to run and
operate the tool with arbitrary Go source files, while @contribs gives a
high-level overview of each group member's contributions to the project.

#pagebreak()

#set heading(numbering: "A.", supplement: "Appendix")
#counter(heading).update(0)

= Glowy: Tool Usage Instructions <usage>

In order to analyze a Go source file using the Glowy binary, one need only:

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

Alternatively, Glowy can be compiled and run directly using ```bash cargo run --release path/to/file.go```.

#v(1em)

_Note: Glowy's behavior is undefined for invalid Go programs, but a best-effort
attempt is made to report useful information for simple mistakes such as tokens
failing parsing expectations._

The GitHub repository includes a directory `examples/` which contains several Go
source files illustrating how to provide annotations and what kinds of features
are supported by the analyzer. These examples may be fed directly as input to
the tool.

#pagebreak()

= Group Member Contributions <contribs>

Rafael implemented the parser, while the analyzer itself was implemented by both
parties; more concretely, Diogo developed the initial structure and function
analysis, while Rafael explored other constructs such as channels and implicit
flows.

In any case, both members worked in close proximity, and all work was subjected
to the other's code review. Ideas were often discussed by both and solutions
brainstormed together.

The present report was written by Rafael, with Diogo's guidance for some parts
that he was more familiar with.

