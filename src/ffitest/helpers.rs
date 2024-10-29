use lean_sys::*;

// conversion functions
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

// array helpers
pub unsafe fn index_lean_array(arr: *mut lean_object, idx: usize) -> usize {
    assert!(lean_is_array(arr));
    let boxed_elem = lean_array_uget(arr, idx);
    // unbox as usize, and leave it to the user to cast it to their desired type.
    lean_unbox(boxed_elem)
}

pub unsafe fn rust_usize_vec_to_lean_array(vec: Vec<usize>) -> *mut lean_object {
    panic!("nyi")
}

// io helpers
pub unsafe fn cleanup_lean_io(o: *mut lean_object) {
    if lean_io_result_is_ok(o) {
        lean_dec_ref(o);
    } else {
        lean_io_result_show_error(o);
        lean_dec(o);
        panic!("IO Monad execution failed");
    }
}

// https://lean-lang.org/lean4/doc/dev/ffi.html#initialization
// https://git.leni.sh/aniva/RustCallLean/src/branch/main/src/main.rs#L30
pub unsafe fn initialize_lean_environment(
    initialize_callee: unsafe extern "C" fn(u8, lean_obj_arg) -> lean_obj_res,
) {
    lean_initialize_runtime_module();
    lean_initialize(); // necessary if you (indirectly) access the `Lean` package

    let builtin: u8 = 1;
    let res = initialize_callee(builtin, lean_io_mk_world());

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
