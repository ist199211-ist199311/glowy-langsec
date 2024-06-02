func main() {
  // glowy::label::{lbl2}
  var a = 1;

  var b = foo(a);
  // ^ label {lbl1, lbl2}
}
