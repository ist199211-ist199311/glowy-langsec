package main

import "fmt"

func foo(a int) int {
  var b = a; // {<0>}
  // glowy::label::{lbl1, lbl2}
  var c = 3;
  var d = b + c; // {<0>, lbl1, lbl2}

  // glowy::label::{lbl2, lbl3}
  var e = 5;

  return bar(d, e);
} // foo returns {<0>, lbl1, lbl2, lbl3}

func bar(a int, b int) int {
  // glowy::label::{lbl1}
  var c = 1;

  return a + b + c; // {<0>, <1>, lbl1}
}

func main() {
  // glowy::label::{lbl4}
  var a = 1;

  var b = foo(a); // b has label {lbl1, lbl2, lbl3, lbl4}

  // glowy::sink::{}
  fmt.Println(b);
}

