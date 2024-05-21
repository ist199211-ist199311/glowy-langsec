package main

import "fmt"

func main() {
    // glowy::label::{high}
    const name = "John Doe";
    // glowy::sink::{}
    fmt.Println("Hello, " + name); // should fail
}
