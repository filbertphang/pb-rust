use lean_sys::*;

mod simple {
    #[link(name = "Simple")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Simple"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn print_hello(world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn return_hello(s: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
    }
}

// https://lean-lang.org/lean4/doc/dev/ffi.html#initialization
// https://git.leni.sh/aniva/RustCallLean/src/branch/main/src/main.rs#L30
fn initialize_lean_environment() {
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
    let c_str_s = std::ffi::CString::new("Rust!").unwrap();
    let result = unsafe {
        // need to cast to *const u8, since that's the type accepted by `lean_mk_string`.
        let c_str_ptr = c_str_s.as_ptr();
        let c_str = lean_mk_string(c_str_ptr as *const u8);
        let result_obj = simple::return_hello(c_str);
        let result_c_str_ptr = lean_string_cstr(result_obj);
        let result_c_str = std::ffi::CStr::from_ptr(result_c_str_ptr as *const i8);
        let result_str = result_c_str.to_str().unwrap().to_string();
        // free c-str on lean side
        lean_dec(result_obj);
        result_str
    };

    println!("{result}");
}

// from george's RustCallLean
fn test_print_from_lean() {
    unsafe {
        let res = simple::print_hello(lean_io_mk_world());
        if lean_io_result_is_ok(res) {
            lean_dec_ref(res);
        } else {
            lean_io_result_show_error(res);
            lean_dec(res);
            panic!("IO Monad execution failed");
        }
    }
}

pub fn main(module: &str) {
    initialize_lean_environment();

    match module {
        "ret" => test_return_from_lean(),
        "pr" => test_print_from_lean(),
        _ => panic!("invalid ffitest module!"),
    }
}
