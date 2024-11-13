use std::ffi::{c_uint, c_void};

use crate::ffitest::lean_helpers::*;
use lean_sys::*;

mod structs {
    #[link(name = "Structs")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Structs"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;

        pub fn return_structured_msg(
            o: lean_sys::lean_obj_arg,
            r: lean_sys::lean_obj_arg,
            v: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        pub fn return_inductive_msg(
            o: lean_sys::lean_obj_arg,
            r: lean_sys::lean_obj_arg,
            v: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        pub fn return_compound_msg(
            o: lean_sys::lean_obj_arg,
            r: lean_sys::lean_obj_arg,
            v: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;

        pub fn get_struct_with_function() -> lean_sys::lean_obj_res;
        pub fn call_struct_with_function(
            wf: lean_sys::lean_obj_arg,
            world: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
    }
}

unsafe fn test_structures() {
    let o = rust_string_to_lean(String::from("0xDEADBEEF"));
    let r = lean_usize_to_nat(420);
    let v = rust_string_to_lean(String::from("hello from rust!"));
    let structured_msg = structs::return_structured_msg(o, r, v);

    let oo = lean_string_to_rust(lean_ctor_get(structured_msg, 0));
    let rr = lean_usize_of_nat(lean_ctor_get(structured_msg, 1));
    let vv = lean_string_to_rust(lean_ctor_get(structured_msg, 2));

    println!("Address: {oo}, Round: {rr}, Value: {vv}");
}

// helpful source:
// https://leni.sh/post/240304-rust-call-lean/
unsafe fn test_inductives() {
    let o = rust_string_to_lean(String::from("0xDEADBEEF"));
    let r = lean_usize_to_nat(420);
    let v = rust_string_to_lean(String::from("hello from rust!"));
    let inductive_msg = structs::return_inductive_msg(o, r, v);

    let tag = lean_ptr_tag(inductive_msg);
    assert!(tag == 1);
    let fields = lean_ctor_num_objs(inductive_msg);
    assert!(fields == 3);
    println!("Constructor Tag: {tag}, Fields: {fields}");

    let oo = lean_string_to_rust(lean_ctor_get(inductive_msg, 0));
    let rr = lean_usize_of_nat(lean_ctor_get(inductive_msg, 1));
    let vv = lean_string_to_rust(lean_ctor_get(inductive_msg, 2));

    println!("Address: {oo}, Round: {rr}, Value: {vv}");
}

unsafe fn test_compounds() {
    let o = rust_string_to_lean(String::from("0xDEADBEEF"));
    let r = lean_usize_to_nat(420);
    let v = rust_string_to_lean(String::from("hello from rust!"));
    let compound_msg = structs::return_compound_msg(o, r, v);

    let tag = lean_ptr_tag(compound_msg);
    assert!(tag == 1);
    // amusingly, scalar values don't seem to count as fields for lean constructors.
    // we only have 1 field corresponding to msg (i think),
    // instead of 2 for msg + num
    let fields = lean_ctor_num_objs(compound_msg);
    println!("Constructor Tag: {tag}, Fields: {fields}");

    let msg = lean_ctor_get(compound_msg, 0);
    let is_msg_ctor = lean_is_ctor(msg);

    let void_ptr_size = size_of::<*mut c_void>();
    // refer to `Structs.lean` for explanation of the offset
    let num_offset: c_uint = (1 * void_ptr_size).try_into().unwrap();
    let num = lean_ctor_get_uint8(compound_msg, num_offset);
    println!("Is msg ctor: {is_msg_ctor}, num: {num}");

    // side note: all this boxing/unboxing may be possible to automate with a macro,
    // but i'm leaving that as a stretch goal.
    let oo = lean_string_to_rust(lean_ctor_get(msg, 0));
    let rr = lean_usize_of_nat(lean_ctor_get(msg, 1));
    let vv = lean_string_to_rust(lean_ctor_get(msg, 2));

    println!("Address: {oo}, Round: {rr}, Value: {vv}");
}

unsafe fn test_functions() {
    // this case tests whether we can pass around structs with functions.
    //
    // verdict: yes they can.
    let s = structs::get_struct_with_function();
    let res = structs::call_struct_with_function(s, lean_io_mk_world());
    cleanup_lean_io(res);
}

pub fn main(module: &str) {
    unsafe {
        initialize_lean_environment(structs::initialize);

        match module {
            "strs" => test_structures(),
            "inds" => test_inductives(),
            "cpds" => test_compounds(),
            "fns" => test_functions(),
            _ => panic!("invalid ffitest::simple test!"),
        }
    };
}
