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

#[no_mangle]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

extern "C" {
    fn print_str(ptr: *const u8, len: usize);
}

static HELLO: &str = "Hello, World!";

// Export a function named "hello_wasm". This can be called
// from the embedder!
#[no_mangle]
pub extern "C" fn hello_wasm() {
    // Call the function we just imported and pass in
    // the offset of our string and its length as parameters.
    unsafe {
        print_str(HELLO.as_ptr(), HELLO.len());
    }
}

#[no_mangle]
fn prepare_buffer(buffer_size: i32) -> i64 {
    wasi_binio_wasm::wasm_prepare_buffer(buffer_size)
}

// #[no_mangle]
// fn prepare_buffer(buffer_size: i32) -> i64 {
//     wasm_prepare_buffer(buffer_size)
// }

#[no_mangle]
fn do_compute(ptr: i32, buffer_size: i32) -> i64 {
    //although we can only use i32 , i32 as function args, we can still call
    //binio_wasm::wasm_deserialize to get the Point struct from runtime.
    let point_tuple: (Point, Point) = wasi_binio_wasm::wasm_deserialize(ptr, buffer_size).unwrap();

    //print out points make sure we have got the correct args
    println!("Log from wasm -- point1 is {:?}", point_tuple.0);
    println!("Log from wasm -- point2 is {:?}", point_tuple.1);

    //the user logic, not related to binio
    let (left, right) = {
        if point_tuple.0.x > point_tuple.1.x {
            (point_tuple.1.x, point_tuple.0.x)
        } else {
            (point_tuple.0.x, point_tuple.1.x)
        }
    };

    let (top, bottom) = {
        if point_tuple.0.y > point_tuple.1.y {
            (point_tuple.1.y, point_tuple.0.y)
        } else {
            (point_tuple.0.y, point_tuple.1.y)
        }
    };
    let rect = Rect {
        left,
        right,
        top,
        bottom,
    };
    //Now we have the rect as function's result. we cannot just return a Rect struct
    //becuase wasm support i32 i64 f32 f64 only
    // so we need to use binio to transfer the rect into memory buffer, then send back the address/lenght
    wasi_binio_wasm::wasm_serialize(&rect).unwrap()
}

// static mut BUFFERS: Vec<Box<[u8]>> = Vec::new();
// fn wasm_prepare_buffer(size: i32) -> i64 {
//     //let buffer : Vec<u8> = Vec::with_capacity(size as usize);
//     let buffer = Vec::<u8>::with_capacity(size as usize).into_boxed_slice();
//     let ptr = buffer.as_ptr() as i32;
//     unsafe { BUFFERS.push(buffer) };
//     join_i32_to_i64(ptr, size)
// }

// fn wasm_deserialize<'a, T>(offset: i32, size: i32) -> bincode::Result<T>
// where
//     T: Deserialize<'a>,
// {
//     let slice = unsafe { std::slice::from_raw_parts(offset as *const _, size as usize) };
//     let _buffer_would_be_dropped = unsafe { BUFFERS.pop() };
//     bincode::deserialize(slice)
// }

// fn wasm_serialize<'a, T>(value: &T) -> bincode::Result<i64>
// where
//     T: Serialize,
// {
//     let buffer_size = bincode::serialized_size(value).unwrap() as i32;
//     let (result_ptr, result_len) = split_i64_to_i32(wasm_prepare_buffer(buffer_size));
//     let serialized_array = bincode::serialize(value).unwrap();
//     let slice =
//         unsafe { std::slice::from_raw_parts_mut(result_ptr as *mut _, result_len as usize) };
//     for i in 0..result_len {
//         slice[i as usize] = serialized_array[i as usize];
//     }
//     Ok(join_i32_to_i64(result_ptr, result_len))
// }

// fn join_i32_to_i64(a: i32, b: i32) -> i64 {
//     //((a as i64) << 32) | (b as i64)
//     (((a as u64) << 32) | (b as u64) & 0x00000000ffffffff) as i64
// }
// /// Split one i64 into two i32
// fn split_i64_to_i32(r: i64) -> (i32, i32) {
//     (
//         (((r as u64) & 0xffffffff00000000) >> 32) as i32,
//         ((r as u64) & 0x00000000ffffffff) as i32,
//     )
// }
