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

pub unsafe fn rust_string_to_lean(s: String) -> *mut lean_object {
    let c_str_s = std::ffi::CString::new(s).unwrap();
    // need to cast to *const u8, since that's the type accepted by `lean_mk_string`.
    let c_str_ptr = c_str_s.as_ptr();
    let c_str = lean_mk_string(c_str_ptr as *const u8);

    c_str
}

pub unsafe fn lean_string_to_rust(s: *mut lean_object) -> String {
    let result_c_str_ptr = lean_string_cstr(s);
    let result_c_str = std::ffi::CStr::from_ptr(result_c_str_ptr as *const i8);
    let result_str = result_c_str.to_str().unwrap().to_string();

    // free c-str on lean side
    lean_dec(s);

    result_str
}

// https://lean-lang.org/lean4/doc/dev/ffi.html#initialization
// https://git.leni.sh/aniva/RustCallLean/src/branch/main/src/main.rs#L30
pub fn initialize_lean_environment() {
    unsafe {
        lean_initialize_runtime_module();
        lean_initialize(); // necessary if you (indirectly) access the `Lean` package
        let builtin: u8 = 1;
        let res = simple::initialize(builtin, lean_io_mk_world());
        if lean_io_result_is_ok(res) {
            lean_dec_ref(res);
        } else {
            lean_io_result_show_error(res);
            lean_dec(res);
            panic!("Failed to load callee!");
        }
        //lean_init_task_manager(); // necessary if you (indirectly) use `Task`
        lean_io_mark_end_initialization();
    }
}

// from leni's RustCallLean
fn test_return_from_lean() {
    let msg = String::from("Rust!");

    let result = unsafe {
        let lean_str = rust_string_to_lean(msg);
        let result_obj = simple::return_hello(lean_str);
        let rust_str = lean_string_to_rust(result_obj);

        rust_str
    };

    println!("{result}");
}

unsafe fn cleanup_lean_io(o: *mut lean_object) {
    if lean_io_result_is_ok(o) {
        lean_dec_ref(o);
    } else {
        lean_io_result_show_error(o);
        lean_dec(o);
        panic!("IO Monad execution failed");
    }
}

// from george's RustCallLean
fn test_print_from_lean() {
    unsafe {
        let res = simple::print_hello(lean_io_mk_world());
        cleanup_lean_io(res);
    }
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

fn test_back_and_forth_with_lean() {
    unsafe {
        let res = simple::back_and_forth(lean_io_mk_world());
        cleanup_lean_io(res);
    }
}

pub fn main(module: &str) {
    initialize_lean_environment();

    match module {
        "ret" => test_return_from_lean(),
        "pr" => test_print_from_lean(),
        "baf" => test_back_and_forth_with_lean(),
        _ => panic!("invalid ffitest::simple test!"),
    }
}
