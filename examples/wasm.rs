use std::str;

use wasmer::{
    imports, wat2wasm, Function, FunctionType, Instance, Module, NativeFunc, Store, Type,
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Rect {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("wasm helloworld");

    let store = Store::new(&Universal::new(Cranelift::default()).engine());
    let module = Module::from_file(
        &store,
        "./target/wasm32-unknown-unknown/release/wasm_demo_32.wasm",
    )?;

    let signature = FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32]);
    // let import_object = imports! {};
    let import_object = imports! {
        // We use the default namespace "env".
        "env" => {
            // And call our function "say_hello".
            "say_hello" => Function::new_native(&store, say_hello_world),
            // "print_str" => Function::new_native(&store, print_str), //print_str<[u32, u32] -> []>,
            // "print_str" => Function::new(&store, signature, print_str),
        }
    };

    println!("Instantiating module...");
    let instance = Instance::new(&module, &import_object)?;

    let add = instance
        .exports
        .get_function("add")?
        .native::<(i32, i32), i32>()?;

    let result = add.call(1, 2)?;

    println!("{}", result);

    let point1 = Point { x: 2, y: 3 };
    let point2 = Point { x: 8, y: 9 };

    let value = (point1, point2);

    let (arg_buffer_ptr, arg_buffer_len) = reserve_wasm_memory_buffer(&value, &instance)?;
    fill_buffer(&value, &instance, arg_buffer_ptr, arg_buffer_len)?;

    // Ok(split_i64_to_i32(result))

    let wasm_func = instance
        .exports
        .get_function("do_compute")?
        .native::<(i32, i32), i64>()?;

    let result_in_i64 = wasm_func.call(arg_buffer_ptr, arg_buffer_len)?;
    let (result_buffer_ptr, result_buffer_len) = split_i64_to_i32(result_in_i64);

    let mem = instance.exports.get_memory("memory")?;
    let mem_array_ref = unsafe { mem.data_unchecked() };
    let start_ptr = mem_array_ref
        .as_ptr()
        .wrapping_offset(result_buffer_ptr as isize);
    let a = unsafe { std::slice::from_raw_parts(start_ptr, result_buffer_len as usize) };

    let decoded: Rect = bincode::deserialize(a).expect("deseralize error");

    println!("-- {:?}", decoded);

    Ok(())
}

fn fill_buffer<T>(
    value: &T,
    instance: &Instance,
    ptr: i32,
    len: i32,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    // let mem = instance.get_export("memory").expect("Cannot get export memory from instance").memory().expect("cannot get memory");
    let mem = instance.exports.get_memory("memory")?;
    let mem_array: &mut [u8];
    let serialized_array = bincode::serialize(value).expect("Error inside bincode::serialize");
    unsafe {
        mem_array = mem.data_unchecked_mut();
        for i in 0..len {
            mem_array[ptr as usize + i as usize] = serialized_array[i as usize];
        }
    }
    Ok(())
}

fn reserve_wasm_memory_buffer<'a, T>(
    value: &T,
    instance: &Instance,
) -> Result<(i32, i32), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    let buffer_size = bincode::serialized_size(value)
        .expect("Error when calculate the buffer size using bincode") as i32;
    let prepare_buffer_func = instance
        .exports
        .get_function("prepare_buffer")?
        .native::<i32, i64>()?;
    let result = prepare_buffer_func.call(buffer_size)?;
    Ok(split_i64_to_i32(result))
}

pub fn join_i32_to_i64(a: i32, b: i32) -> i64 {
    //((a as i64) << 32) | (b as i64)
    (((a as u64) << 32) | (b as u64) & 0x00000000ffffffff) as i64
}
/// Split one i64 into two i32
pub fn split_i64_to_i32(r: i64) -> (i32, i32) {
    (
        (((r as u64) & 0xffffffff00000000) >> 32) as i32,
        ((r as u64) & 0x00000000ffffffff) as i32,
    )
}

fn say_hello_world() {
    println!("Hello, world!")
}

extern "C" fn print_str(ptr: u32, len: u32) {
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    // let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    // let str_slice = &memory[ptr as usize..(ptr + len) as usize];

    // Convert the subslice to a `&str`.
    // let string = str::from_utf8(str_slice).unwrap();

    // Print it!
    println!("{}", "string");
}
