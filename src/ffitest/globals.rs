use crate::ffitest::simple::{
    initialize_lean_environment, lean_string_to_rust, rust_string_to_lean,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static GLOBAL_HASHTBL: Lazy<Mutex<HashMap<u8, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn insert_to_hashtbl(k: u8, v: String) {
    let mut ht = GLOBAL_HASHTBL.lock().unwrap();
    ht.insert(k, v.clone());
    println!("inserting {k}:{v} into the hashtbl");
}

fn query_hashtbl(k: u8) {
    let ht = GLOBAL_HASHTBL.lock().unwrap();
    let res = ht.get(&k);
    match res {
        Some(x) => println!("query {k} in hashtbl: found {x}"),
        None => println!("query {k} NOT found in hashtbl"),
    }
}

fn test_basic() {
    query_hashtbl(3);
    insert_to_hashtbl(3, String::from("Hello, World!"));
    query_hashtbl(3);
    query_hashtbl(5);
}

fn test_with_lean() {
    panic!("nyi")
}

pub fn main(module: &str) {
    // initialize_lean_environment();

    match module {
        "basic" => test_basic(),
        "with_lean" => test_with_lean(),
        _ => panic!("invalid ffitest::globals test!"),
    }
}
