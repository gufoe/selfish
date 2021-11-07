#![allow(deprecated)]
#![allow(dead_code)]

use rand::random;

use hashbrown::HashMap;
use std::hash::Hash;
use std::{thread, time};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::fmt::Debug;

use rand::thread_rng;
use rand::Rng;
use time::PreciseTime;
use time::Duration;

use yaml_rust::Yaml;
use getopts::Matches;


pub fn rand_hash<K: Eq + Hash, V>(hash: &HashMap<K, V>) -> Option<&V> {
    if hash.is_empty() {
        return None;
    }
    let index = thread_rng().gen_range(0, hash.len());
    hash.values().skip(index).next()
}
pub fn rand_hash_key<K: Eq + Hash, V>(hash: &HashMap<K, V>) -> Option<&K> {
    if hash.is_empty() {
        return None;
    }
    let index = thread_rng().gen_range(0, hash.len());
    hash.keys().skip(index).next()
}

pub fn rand_hash_mut<K: Eq + Hash, V>(hash: &mut HashMap<K, V>) -> Option<&mut V> {
    if hash.is_empty() {
        return None;
    }
    let index = thread_rng().gen_range(0, hash.len());
    hash.values_mut().skip(index).next()
}

pub fn rand_int(max: u32) -> u32 {
    thread_rng().gen_range(0, max)
}
pub fn rand_usize(max: usize) -> usize {
    thread_rng().gen_range(0, max)
}
pub fn rand_intr(min: i32, max: i32) -> i32 {
    thread_rng().gen_range(min, max)
}
pub fn rand_float(min: f32, max: f32) -> f32 {
    thread_rng().gen_range(min, max)
}
pub fn rand_string(size: u32) -> String {
    (0..size).map(|_| (65u8 + (random::<f32>() * 26.) as u8) as char).collect()
}
pub fn maybe(pty: f32) -> bool{
    rand_float(0., 1.) < pty
}

pub fn shuffle<T>(vec: &mut Vec<T>) {
    thread_rng().shuffle(vec);
}

pub fn sleep(millis: u64) {
    let ten_millis = time::Duration::from_millis(millis);
    thread::sleep(ten_millis);
}

pub fn time() -> Time {
    Time::start()
}

pub fn now() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ste = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    ste.as_secs() as usize * 1000 + ste.subsec_nanos() as usize / 1_000_000
}

pub struct Time {
    start: PreciseTime,
}

impl Time {
    pub fn start() -> Time {
        Time {
            start: PreciseTime::now(),
        }
    }

    pub fn diff(&self) -> Duration {
        self.start.to(PreciseTime::now())
    }
    pub fn diff_milli(&self) -> f32 {
        return self.diff().num_microseconds().unwrap() as f32 / 1000.0;
    }
}

pub fn mma(v: &[f32]) -> (f32, f32, f32) {
    let mut max = -9999999999.0;
    let mut min = 9999999999.0;
    let mut tot = 0.0;
    for i in v.iter() {
        if *i > max {
            max = *i;
        }
        if *i < min {
            min = *i;
        }
        tot+= *i;
    }
    (min, max, tot / v.len() as f32)
}

#[allow(dead_code)]
pub fn max_i(v: &[impl PartialOrd]) -> usize {
    let mut max = 0;
    for i in 1..v.len() {
        if v[i] > v[max] {
            max = i;
        }
    }
    max
}
#[allow(dead_code)]
pub fn min_i(v: &[impl PartialOrd]) -> usize {
    let mut min = 0;
    for i in 1..v.len() {
        if v[i] < v[min] {
            min = i;
        }
    }
    min
}
pub fn softmax(v: &mut [f32]) {
    norm(v);
    let sum = v.iter().fold(0.0, |sum, &val| sum+val);
    if sum > 0. {
        for n in v { *n /= sum }
    }
}
pub fn norm(v: &mut [f32]) {
    let mut max = -9999999999.0;
    let mut min = 9999999999.0;
    for i in v.iter() {
        if *i > max {
            max = *i;
        }
        if *i < min {
            min = *i;
        }
    }
    let mut n = max-min;
    if n == 0. {
        n = 1.;
    }
    for i in v.iter_mut() {
        *i-= min;
        *i/= n;
    }
}


pub fn read_file(file: &String) -> String {
    let mut f = File::open(file).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(&format!("something went wrong reading the file {}", file));
    contents
}


pub fn opt_i64(opts: &Yaml, field: &str, default: i64) -> i64 {
    if opts[field].is_badvalue() {
        eprintln!("Could not find opt {} -> {}", field, default);
        default
    } else {
        opts[field].as_i64().unwrap()
    }
}
pub fn opt_str(opts: &Yaml, field: &str, default: String) -> String {
    if opts[field].is_badvalue() {
        eprintln!("Could not find opt {} -> {}", field, default);
        default
    } else {
        opts[field].as_str().unwrap().to_string()
    }
}
pub fn opt_f32(opts: &Yaml, field: &str, default: f32) -> f32 {
    if opts[field].is_badvalue() {
        eprintln!("Could not find opt {} -> {}", field, default);
        default
    } else {
        opts[field].as_f64().unwrap() as f32
    }
}
pub fn arg<T>(args: &Matches, field: &str, default: T) -> T
where T: FromStr, <T as FromStr>::Err: Debug {
    if args.opt_present(field) {
        args.opt_get::<T>(field).unwrap().unwrap()
    } else {
        eprintln!("Could not find arg {}", field);
        default
    }
}

pub fn gmut<'a, T>(hm: &'a mut HashMap<String, T>, key: &str) -> &'a mut T
{
    hm.get_mut(key).unwrap()
}


pub fn pop<T>(set: &mut std::collections::HashSet<T>) -> T
where
    T: Eq + Clone + std::hash::Hash,
{
    let elt = set.iter().next().cloned().unwrap();
    set.remove(&elt);
    elt
}
