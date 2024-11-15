@[export create_array]
def create_array (x: UInt32) (y: UInt32) : Array UInt32 :=
  #[x, y]

@[export print_array]
def print_array (xs : Array UInt32) : IO Unit :=
  let my_arr := #[9, 15, 26]
  let another_arr := #[100, 200, 300, 400, 500]
  do
    IO.println s!"(lean) here's a sample array: {my_arr}"
    IO.println s!"(lean) here's another sample array: {another_arr}"
    IO.println s!"(lean) here's your array: {xs}"


@[export return_string_array]
def return_string_array (_ : Unit) : Array String :=
  #["hello", "world"]

@[export print_string_array]
def print_string_array (xs : Array String) : IO Unit :=
  do
    IO.println s!"(lean) here's your string array: {xs}"
