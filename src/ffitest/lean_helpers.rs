use lean_sys::*;

pub const VOID_PTR_SIZE: usize = size_of::<*mut std::ffi::c_void>();

pub enum Mode {
    Borrow,
    Owned,
}

// strings
// TODO: is there a better way that transfers ownership from rust to lean
// without de-allocating/re-allocating?

/// Copies a Rust string into Lean.
/// The Rust string will be deallocated, and re-allocated on the Lean side.
pub unsafe fn rust_string_to_lean(s: String) -> *mut lean_object {
    let c_str_s = std::ffi::CString::new(s).unwrap();
    // need to cast to *const u8, since that's the type accepted by `lean_mk_string`.
    let c_str_ptr = c_str_s.as_ptr();
    // reallocation in lean occurs here
    let c_str = lean_mk_string(c_str_ptr as *const u8);

    c_str

    // rust string `s` is freed after this block ends
}

/// Copies a Lean string into Rust.
/// The Lean string will be deallocated, and re-allocated on the Rust side.
pub unsafe fn lean_string_to_rust(s: *mut lean_object, mode: Mode) -> String {
    let result_c_str_ptr = lean_string_cstr(s);
    let result_c_str = std::ffi::CStr::from_ptr(result_c_str_ptr as *const i8);
    let result_str = result_c_str.to_str().unwrap().to_string();

    // free c-str on lean side
    match mode {
        Mode::Borrow => (),
        Mode::Owned => lean_dec(s),
    }

    result_str
}

// arrays
pub unsafe fn index_lean_array(arr: *mut lean_object, idx: usize) -> usize {
    assert!(lean_is_array(arr));
    let boxed_elem = lean_array_uget(arr, idx);
    // unbox as usize, and leave it to the user to cast it to their desired type.
    lean_unbox(boxed_elem)
}

pub unsafe fn rust_usize_vec_to_lean_array(vec: Vec<usize>) -> *mut lean_object {
    // this is for creating lean arrays of primitives (USize, UInt_32, etc).
    // for lean arrays of non-primitives, see impl in `rust_string_vec_to_lean_array` below.

    // this is fairly inefficient, because we do an O(n) loop to copy each array element
    // to lean array, only to do another O(n) conversion from lean Array to lean List.
    //
    // we can probably do better by creating the lean array struct then just copying over
    // the pointer for the underlying C-array into the `data` field of the struct,
    // but lets worry about performance later.

    let vec_len = vec.len();
    let arr = lean_mk_empty_array_with_capacity(lean_box(vec_len));
    for elem in vec {
        lean_array_push(arr, lean_box(elem));
    }
    arr
}

pub unsafe fn rust_string_vec_to_lean_array(vec: Vec<String>) -> *mut lean_object {
    // specialized for strings.
    // but this approach should work for any lean Array T type where T is
    // represented as a lean_object instead of a primitive.
    let vec_len = vec.len();
    let arr = lean_mk_empty_array_with_capacity(lean_box(vec_len));
    for elem in vec {
        let lean_str = rust_string_to_lean(elem);
        lean_array_push(arr, lean_str);
    }
    arr
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

/// Figure out what this Lean object is supposed to be.
#[allow(dead_code)]
pub unsafe fn what_is_this(s: &str, o: *mut lean_object) {
    println!(
            "\n === what is {s}? === \n {s} is string? {}.\n {s} is ref? {}.\n {s} is ctor? {}.\n {s} is scalar? {}.\n {s} is thunk? {}.\n ====== \n",
            lean_is_string(o),
            lean_is_ref(o),
            lean_is_ctor(o),
            lean_is_scalar(o),
            lean_is_thunk(o),
        );

    if lean_is_ctor(o) {
        println!(
            "since {s} is a constructor:\n
            ctor tag: {},
            num objs: {}
            ",
            lean_ptr_tag(o),
            lean_ctor_num_objs(o),
        )
    }
}
