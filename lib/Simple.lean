@[export return_hello]
def return_hello (s: String): String :=
  s!"Hello, {s}!"

@[export print_hello]
def print_hello : IO Unit :=
  IO.println s!"Hello from Lean!"

@[extern "from_rust"]
opaque from_rust : String → String

@[export back_and_forth]
def back_and_forth : IO Unit :=
  IO.println s!"Hello from lean: {from_rust "world"}"
