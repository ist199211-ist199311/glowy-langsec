package main

import "fmt"

func main() {
    var x = 0;
    // glowy::label::{high}
    var y = 1 + x;
    var z = y + 3;

    // glowy::sink::{}
    fmt.Println(x); // should succeed
    // glowy::sink::{high}
    fmt.Println(y); // should succeed
    // glowy::sink::{}
    fmt.Println(z); // should fail
}
