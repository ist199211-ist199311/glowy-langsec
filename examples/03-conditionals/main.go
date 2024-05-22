package main

import "fmt"

func main() {
    var x = 0;
    // glowy::label::{high}
    var y = 1 + x;
    var z = 0;
    var a = 0;

    if (y == 1) {
        z = 1;
    } else {
        z = 2;
    }

    if (x == 0) {
        a = z;
    } else {
        a = x;
    }

    // glowy::sink::{}
    fmt.Println(x); // should succeed
    // glowy::sink::{high}
    fmt.Println(y); // should succeed
    // glowy::sink::{}
    fmt.Println(z); // should fail
    // glowy::sink::{}
    fmt.Println(a); // should fail
}
