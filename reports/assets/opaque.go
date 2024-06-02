// glowy::label::{sensitive}
const secret = 123

func opaque(seed int) int {
  if (seed + secret) % 10 == 0 {
    return 5
  } else {
    return 7
  }
}

func main() {
  // glowy::sink::{}
  fmt.Println(4 * opaque(2))
}
