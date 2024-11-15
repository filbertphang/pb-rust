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

abbrev MyStr1 := String
@[export return_mystr1]
def return_mystr1 (_: Unit): MyStr1 :=
  s!"Hello, mystr1!"

@[reducible] def MyStr2 := String
@[export return_mystr2]
def return_mystr2 (_: Unit): MyStr2 :=
  s!"Hello, mystr2!"
