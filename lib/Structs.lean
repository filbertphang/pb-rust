/-
following the lean ffi manual, the memory layout of this struct should be identical as
its definition, since all 3 fields are represented as `lean_object *`s.

so we should be able to access them in the correct order.
-/
structure StructuredMessage :=
  originator : String
  round : Nat
  value: String

@[export return_structured_msg]
def return_structured_msg (o: String) (r: Nat) (v: String) : StructuredMessage :=
  {originator := o, round := r, value := v}

-- this implementation mimics the actual Message type for bythos,
-- with concrete type parameters.
inductive InductiveMessage
  | InitialMsg (r : Nat) (v : String)
  | EchoMsg (originator : String) (r : Nat) (v : String)
  | VoteMsg (originator : String) (r : Nat) (v : String) (dummy_field_for_testing: String)

@[export return_inductive_msg]
def return_inductive_msg (o: String) (r: Nat) (v: String) : InductiveMessage :=
  InductiveMessage.EchoMsg o r v

/-
note: because Bool is represented as a UInt8, this struct's memory layout would actually
be re-ordered to the following:

inductive CompoundMessage
  | None
  | ActualMessage
    (msg: StructuredMessage) -- lean_ctor_get(val, 0)
    (num: UInt8) -- lean_ctor_get_uint8(val, sizeof(void*))

-/
inductive CompoundMessage
  | None
  | ActualMessage (num: UInt8) (msg: StructuredMessage)

@[export return_compound_msg]
def return_compound_msg (o: String) (r: Nat) (v: String) : CompoundMessage :=
  CompoundMessage.ActualMessage 17 {originator := o, round := r, value := v}

structure WithFunction :=
  f: String → String

def my_f (s: String) : String := s!"i have added to {s}"

@[export get_struct_with_function]
def get_struct_with_function (_: Unit) : WithFunction :=
  {f := my_f}

@[export call_struct_with_function]
def call_struct_with_function (wf : WithFunction) : IO Unit :=
  IO.println s!"Hello from lean: {wf.f "oops"}"
