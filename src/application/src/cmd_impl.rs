#![allow(non_snake_case)]

pub fn testfct(b: u8, u: u32, i: i32, B: bool, U: u64) {
    println!("testfct called with: {}, {}, {}, {}, {}", b, u, i, B, U);
}

pub fn testi(i1: i32, i2: i32, i3: i32, i4: i32, i5: i32) {
    println!("testi called with: {}, {}, {}, {}, {}", i1, i2, i3, i4, i5);
}

pub fn greet(s: &str) {
    println!("Hello, {}!", s);
}

pub fn greet2(s1: &str, s2: &str) {
    println!("{} - {}", s1, s2);
}

pub fn greet_again(s: &str) {
    println!("Welcome again, {}!", s);
}

pub fn parse_mix(w: u16, f: f64, s: &str) {
    println!("parse_mix: w={}, f={}, s={}", w, f, s);
}

pub fn vtest() {
    println!("vtest()");
}
