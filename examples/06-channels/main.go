package main

import "fmt"

func main() {
    var ch = make(chan int)

    var x = 7;
    var y = 3;
    // glowy::label::{high}
    var z = 2;

    go foo(x, ch)
    go foo(y, ch)
    go foo(z, ch)

    // glowy::sink::{}
    fmt.Println(<- ch); // should fail
    // glowy::sink::{}
    fmt.Println(<- ch); // should fail
    // glowy::sink::{}
    fmt.Println(<- ch); // should fail
}

func foo(a int, ch chan int) {
    var b = a * a
    ch <- b
}
