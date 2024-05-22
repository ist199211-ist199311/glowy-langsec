package main

import "fmt"

func main() {
    var x = 0;
    var y = foo(x, 5);
    bar(y);

    // glowy::label::{high}
    var z = 4;
    var w = foo(z, y);
    bar(w);
}

func foo(a int, b int) int {
    return a + b
}

func bar(a int) {
    // glowy::sink::{}
    fmt.Println(a) // should succeed when called with y, but fail when called with w
}
