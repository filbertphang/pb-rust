@[extern "query_hashtbl_with_res"]
opaque query_hashtbl_with_res : UInt8 → String

@[export query]
def query (k: UInt8): IO Unit :=
  IO.println s!"(lean) queried hashtbl for {k}, got {query_hashtbl_with_res k}"
