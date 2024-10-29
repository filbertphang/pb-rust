@[export create_array]
def create_array (x: UInt32) (y: UInt32) : Array UInt32 :=
  #[x, y]

@[export print_array]
def print_array (xs : Array UInt32) : IO Unit :=
  IO.println s!"(lean) here's your array: {xs}"
