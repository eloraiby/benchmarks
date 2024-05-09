//
// Copyright (C) 2024 Wael El Oraiby.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
// OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
////////////////////////////////////////////////////////////////////////////////
//
// NoHasher code:
//
// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
// OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::{
    collections::HashMap,
    fs::File,
    hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher, RandomState},
    io::Write,
    time::Instant,
};

use rand::{thread_rng, RngCore};

#[derive(PartialEq, Eq, Clone, Copy)]
struct Index(usize);
impl Hash for Index {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0)
    }
}

#[derive(Default)]
pub struct NoHasher(u64);

///
/// NoHasher: code from https://github.com/paritytech/nohash-hasher
///
impl Hasher for NoHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("Invalid use of NoHashHasher")
    }

    fn write_u8(&mut self, n: u8) {
        self.0 = u64::from(n)
    }
    fn write_u16(&mut self, n: u16) {
        self.0 = u64::from(n)
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = u64::from(n)
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = n as u64
    }

    fn write_i8(&mut self, n: i8) {
        self.0 = n as u64
    }
    fn write_i16(&mut self, n: i16) {
        self.0 = n as u64
    }
    fn write_i32(&mut self, n: i32) {
        self.0 = n as u64
    }
    fn write_i64(&mut self, n: i64) {
        self.0 = n as u64
    }
    fn write_isize(&mut self, n: isize) {
        self.0 = n as u64
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn test_hashmap_with_size<Hasher: BuildHasher + Default>(
    entry_count: usize,
    access_count: usize,
    tmp: &mut usize,
) -> (usize, u128) {
    let mut rng = thread_rng();
    let mut hm = HashMap::<Index, usize, Hasher>::default();
    let mut access = Vec::new();
    let mut ids = Vec::new();
    for _ in 0..entry_count {
        let id = rng.next_u64() as usize;
        ids.push(Index(id));
        hm.insert(Index(id), rng.next_u64() as usize);
    }
    for _ in 0..entry_count * 4 {
        access.push(rng.next_u64() as usize)
    }

    let start = Instant::now();

    for a in 0..access_count {
        let id = ids[access[a % access.len()] % ids.len()];
        *tmp = tmp.overflowing_add(hm[&id]).0;
    }
    let duration = start.elapsed();

    (hm.capacity(), duration.as_nanos())
}

fn test_vec_with_size(entry_count: usize, access_count: usize, tmp: &mut usize) -> u128 {
    let mut rng = thread_rng();
    let mut v = Vec::new();
    let mut access = Vec::new();

    for _ in 0..entry_count {
        v.push(rng.next_u64() as usize);
    }
    for _ in 0..entry_count * 4 {
        access.push(rng.next_u64() as usize)
    }

    let start = Instant::now();

    for a in 0..access_count {
        let id = access[a % access.len()] % entry_count;
        *tmp = tmp.overflowing_add(v[id]).0;
    }
    let duration = start.elapsed();

    duration.as_nanos()
}

fn test_vec_with_size_dbl_ind(entry_count: usize, access_count: usize, tmp: &mut usize) -> u128 {
    let mut rng = thread_rng();
    let mut hm = Vec::new();
    let mut access = Vec::new();
    let mut ids = Vec::new();
    for _ in 0..entry_count {
        ids.push((rng.next_u64() as usize) % entry_count);
        hm.push(rng.next_u64() as usize);
    }
    for _ in 0..entry_count * 4 {
        access.push(rng.next_u64() as usize)
    }

    let start = Instant::now();

    for a in 0..access_count {
        let id = ids[access[a % access.len()] % ids.len()];
        *tmp = tmp.overflowing_add(hm[id]).0;
    }
    let duration = start.elapsed();

    duration.as_nanos()
}

fn main() {
    let sizes = [
        1024,
        2048,
        4096,
        8192,
        16384,
        32768,
        65536,
        128 * 1024,
        256 * 1024,
        512 * 1024,
        1024 * 1024,
    ];

    let mut tmp = 0usize;

    println!("Element Count; Default HashMap(ms); NoHasher HashMap(ms); HashMap capacity; Vector(No Indirection)(ms); Vector(Indirection)(ms)");
    for s in sizes {
        let duration_hm_default_hasher = test_hashmap_with_size::<BuildHasherDefault<DefaultHasher>>(
            s,
            1024 * 1024 * 1024,
            &mut tmp,
        );
        let duration_hm_no_hasher =
            test_hashmap_with_size::<BuildHasherDefault<NoHasher>>(s, 1024 * 1024 * 1024, &mut tmp);
        let duration_v = test_vec_with_size(s, 1024 * 1024 * 1024, &mut tmp);
        let duration_v_dbl = test_vec_with_size_dbl_ind(s, 1024 * 1024 * 1024, &mut tmp);
        println!(
            "{}; {}; {}; {}; {}; {}",
            s,
            duration_hm_default_hasher.1 / (1000 * 1000),
            duration_hm_no_hasher.1 / (1000 * 1000),
            duration_hm_default_hasher.0,
            duration_v / (1000 * 1000),
            duration_v_dbl / (1000 * 1000)
        );
    }

    let mut f = File::create("/tmp/garbage.txt").unwrap();
    f.write_fmt(format_args!("{}", tmp)).unwrap();
}
