use circuit::circuit;
use wasm_bindgen::prelude::*;

extern "C" {
    fn wasm_input(_: u32) -> u64;
}

#[wasm_bindgen]
pub fn zkwasm() -> u64 {
    let a = unsafe { wasm_input(0) };
    let b = unsafe { wasm_input(1) };
    // let source_list = read_addr_from_input(0);
    // let block_list = read_addr_from_input(0);

    // circuit(source_list, block_list) as u64
    a - b
}

// fn read_addr_from_input(typ: u32) -> Vec<String> {
//     let len = unsafe { wasm_input(typ as u32) };
//     let mut result = vec![];
//     let mut str = String::from("");

//     while len > 0 {
//         loop {
//             let ch = unsafe { wasm_input(typ as u32) };
//             let ch = char::from_u32(ch as u32).unwrap();
//         }
//     }
//     for _ in 0..len {
//         let ch = unsafe { wasm_input(typ as u32) };
//         let ch = char::from_u32(ch as u32).unwrap();
//         str.push(ch);
//     }

//     result
// }
