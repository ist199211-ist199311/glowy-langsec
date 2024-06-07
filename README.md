# Glowy

Glowy is a tool written in Rust to analyze Go code in order to enforce information flow control and detect potentially insecure interference.

## Usage Instructions

In order to analyze a Go source file using the Glowy binary, one need only:

-   Obtain the tool's source code from the present repository:

    `$ git clone git@github.com:ist199211-ist199311/glowy`

-   Compile Glowy:

    `$ cargo build --release`

-   Optionally create a convenience link to the binary in the root directory or
    somewhere on the `$PATH`:

    `$ ln -s target/release/glowy ./glowy`

-   Annotate the target `.go` files with line comments specifying what source and
    sink label constraints should be enforced, e.g.:

    ```go
    // (...)

    // glowy::label::{high}
    const secret = "hunter12"

    // (...)

    // glowy::sink::{}
    fmt.Println(result)
    ```

-   Analyze the annotated source file:

    `$ ./glowy path/to/file.go`

Alternatively, Glowy can be compiled and run directly using `$ cargo run --release path/to/file.go`.

---

_Note: Glowy's behavior is undefined for invalid Go programs, but a best-effort
attempt is made to report useful information for simple mistakes such as tokens
failing parsing expectations._

This repository includes a directory [`examples/`](/examples) which contains several Go
source files illustrating how to provide annotations and what kinds of features
are supported by the analyzer. These examples may be fed directly as input to
the tool.
