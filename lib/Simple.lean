@[export return_hello]
def return_hello (s: String): String :=
  s!"Hello, {s}!"

@[export print_hello]
def print_hello : IO Unit :=
  IO.println s!"Hello from Lean!"
