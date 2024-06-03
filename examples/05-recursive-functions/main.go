package main

import "fmt"

// glowy::label::{secret}
const secret = 123;

func main() {
    // glowy::label::{high}
    var x = 7;

    var b = even(x);

    // glowy::sink::{}
    fmt.Println(b); // should fail
}

func even(n int) bool {
    if (n == 0) {
        return true;
    }
    return odd(n - 1);
}

func odd(n int) bool {
    if (n == 0 && secret > 0) {
        return false;
    }
    return even(n - 1);
}
