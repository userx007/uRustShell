#![allow(non_snake_case)]

pub fn init() {
    println!("init | no-args");
}

pub fn read(descr: i8, nbytes: u32) {
    println!("read | descriptor: {}, bytes:{}", descr, nbytes);
}

pub fn write(filename: &str, nbytes: u64, val: u8) {
    println!(
        "write | filename: {}, bytes:{}, value:{:X}/{:o}/{:b}",
        filename, nbytes, val, val, val
    );
}

pub fn led(onoff: bool) {
    if onoff {
        println!("led | ON");
    } else {
        println!("led | OFF");
    }
}

pub fn greeting(s1: &str, s2: &str) {
    println!("greeting | [{}] : [{}]", s1, s2);
}

pub fn send(port: &str, baud: u32, data: &[u8]) {
    println!("send | port: {} baudrate: {}, data:{:?}", port, baud, data);
}

pub fn astring(s: &str) {
    println!("astring | {}", s);
}

pub fn bstring(s: &str) {
    println!("bstring | {}", s);
}

pub fn cstring(s: &str) {
    println!("cstring | {}", s);
}
