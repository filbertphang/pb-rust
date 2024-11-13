use crate::ffitest::lean_helpers::*;
use lean_sys::*;

mod simple {
    #[link(name = "Simple")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Simple"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn return_hello(s: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn print_hello(world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn back_and_forth(world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
    }
}

// from leni's RustCallLean
unsafe fn test_return_from_lean() {
    let msg = String::from("Rust!");

    let lean_str = rust_string_to_lean(msg);
    let result_obj = simple::return_hello(lean_str);
    let rust_str = lean_string_to_rust(result_obj);

    println!("{rust_str}");
}

// from george's RustCallLean
unsafe fn test_print_from_lean() {
    let res = simple::print_hello(lean_io_mk_world());
    cleanup_lean_io(res);
}

// note: may want to write a macro or something similar to facilitate
// rust-lean type conversions so we don't have to do this boilerplate
// marshalling every time.
//
// when you use an `extern` function in lean, the called function is expected
// to conform to lean's memory model.
// so we have to do the marshalling here on the rust-side, rather than on
// the lean side.
#[no_mangle]
pub unsafe extern "C" fn from_rust(s: *mut lean_object) -> *mut lean_object {
    let rust_str = lean_string_to_rust(s);

    // body code
    let res = format!("{rust_str} (from rust!!)");

    rust_string_to_lean(res)
}

unsafe fn test_back_and_forth_with_lean() {
    let res = simple::back_and_forth(lean_io_mk_world());
    cleanup_lean_io(res);
}

unsafe fn double_call() {
    // this testcase checks whether lean functions take ownership of their arguments,
    // and drop it from memory after a function call.
    //
    // verdict: they don't. you can re-use variables across multiple calls.
    let x = String::from("me");

    let lean_s = simple::return_hello(rust_string_to_lean(x));

    let lean_s1 = simple::return_hello(lean_s);
    let lean_s2 = simple::return_hello(lean_s);

    let lean_s11 = lean_string_to_rust(lean_s1);
    let lean_s21 = lean_string_to_rust(lean_s2);

    println!("{lean_s11}");
    println!("{lean_s21}");
}

pub fn main(module: &str) {
    unsafe {
        initialize_lean_environment(simple::initialize);

        match module {
            "ret" => test_return_from_lean(),
            "pr" => test_print_from_lean(),
            "baf" => test_back_and_forth_with_lean(),
            "dc" => double_call(),
            _ => panic!("invalid ffitest::simple test!"),
        }
    };
}
