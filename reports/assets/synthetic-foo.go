func foo(a int) int {
  var b = a; // {<0>}

  // glowy::label::{lbl1}
  var c = 3;
  var d = b + c; // {<0>, lbl1}

  return a;
} // foo returns {<0>, lbl1}
