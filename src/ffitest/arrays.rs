use crate::ffitest::helpers::*;
use lean_sys::*;

// testing involving array interfacing between lean and rust.
//
// this is needed when passing in the node list to lean for protocol initialization,
// which has type `List Address`.
//
// it's probably simpler to pass in a lean array then convert it to a list in the lean side
// than it is to construct a lean List here in rust.

mod arrays {
    #[link(name = "Arrays")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Arrays"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;

        pub fn create_array(x: u32, y: u32) -> lean_sys::lean_obj_res;
        pub fn print_array(
            xs: lean_sys::lean_obj_arg,
            world: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
    }
}

unsafe fn test_create_array() {
    let res = arrays::create_array(39, 251);
    let isarr = lean_is_array(res);
    println!("is array: {isarr}");

    let ax = index_lean_array(res, 0) as u32;
    println!("first elem: {ax}");

    let bx = index_lean_array(res, 1) as u32;
    println!("second elem: {bx}");

    lean_dec(res);
}

fn test_print_array() {
    panic!("nyi")
}

pub fn main(module: &str) {
    unsafe {
        initialize_lean_environment(arrays::initialize);

        match module {
            "cr" => test_create_array(),
            "prn" => test_print_array(),
            _ => panic!("invalid ffitest::simple test!"),
        }
    };
}
