use crate::ffitest::helpers::*;
use lean_sys::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

mod globals {
    #[link(name = "Globals")]
    extern "C" {
        #[link_name = "initialize_Globals"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        pub fn query(k: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
    }
}

static GLOBAL_HASHTBL: Lazy<Mutex<HashMap<u8, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn insert_to_hashtbl(k: u8, v: String) {
    let mut ht = GLOBAL_HASHTBL.lock().unwrap();
    ht.insert(k, v.clone());
    println!("(rust) inserting {k}:{v} into the hashtbl");
}

fn query_hashtbl(k: u8) {
    let ht = GLOBAL_HASHTBL.lock().unwrap();
    let res = ht.get(&k);
    match res {
        Some(x) => println!("(rust) query {k} in hashtbl: found {x}"),
        None => println!("(rust) query {k} NOT found in hashtbl"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn query_hashtbl_with_res(k: u8) -> *mut lean_object {
    let ht = GLOBAL_HASHTBL.lock().unwrap();
    let res = ht.get(&k).unwrap().clone();
    unsafe { rust_string_to_lean(res) }
}

fn test_basic() {
    query_hashtbl(3);
    insert_to_hashtbl(3, String::from("Hello, World!"));
    query_hashtbl(3);
    query_hashtbl(5);
}

fn test_with_lean() {
    // we want to be able to access the global hashtbl state from lean.
    // this is a stepping stone to implementing the [inputValue] function.
    unsafe {
        initialize_lean_environment(globals::initialize);

        insert_to_hashtbl(3, String::from("Hello, World!"));
        let res = globals::query(3, lean_io_mk_world());
        cleanup_lean_io(res);

        insert_to_hashtbl(5, String::from("Goodbye, World!"));
        let res2 = globals::query(5, lean_io_mk_world());
        cleanup_lean_io(res2);
    }
}

pub fn main(module: &str) {
    match module {
        "basic" => test_basic(),
        "with_lean" => test_with_lean(),
        _ => panic!("invalid ffitest::globals test!"),
    }
}
