use yaml_rust::Yaml;
use std::fs::File;
use std::io::prelude::*;
use hashbrown::HashMap;
use serde::{Serialize};
use ga;
use gapool;
use net;
use serde_json;



pub fn load_ga(session_path: &String) -> ga::GA {
    match File::open(&session_path) {
        Ok(mut f) => {
            eprintln!("[storage] Loading GA from {}...", session_path);
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).expect("[storage] Session file is broken")
        }
        Err(_) => {
            eprintln!("[storage] Generating a new GA ({})", session_path);
            ga::GA::new()
        }
    }
}

pub fn store_ga(session_path: &String, ga: &ga::GA) {

    // thread::spawn(move || {
        let json = serde_json::to_string(&ga).expect("[storage] Couldn't serialize GA");
        let mut file = File::create(session_path).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        eprintln!("[storage] done saving ga");
    // });
}

pub fn load_ga_pool(session_path: &String, opts: &Yaml) -> Option<HashMap<String, gapool::GaPool>> {
    match File::open(&session_path) {
        Ok(mut f) => {
            eprintln!("[storage] GaPool from {}...", session_path);
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            let mut gas: HashMap<String, gapool::GaPool> = serde_json::from_str(&contents).expect("[storage] Session file is broken");
            for ga in gas.values_mut() {
                ga.reconfigure(opts);
            }
            Some(gas)
        }
        Err(_) => {
            eprintln!("[storage] Generating a new GaPool ({})", session_path);
            None
        }
    }
}

pub fn store_ga_pools(session_path: &String, gaps: &HashMap<String, gapool::GaPool>) {

    // thread::spawn(move || {
        let json = serde_json::to_string(&gaps).expect("[storage] Couldn't serialize GA");
        let mut file = File::create(session_path).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        eprintln!("[storage] done saving ga");
    // });
}

#[allow(dead_code)]
pub fn load_net(session_path: &String) -> net::Net {
    eprintln!("[storage] Loading net from {}...", session_path);
    let mut f = File::open(&session_path).expect("[storage] File cannot be opened");
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).expect("[storage] File file is broken")
}


pub fn store<T: Serialize>(session_path: &String, x: &T) {
    let json = serde_json::to_string(&x).expect("[storage] Couldn't serialize T");
    let mut file = File::create(session_path).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    eprintln!("[storage] done saving net");
}
pub fn load_leaders(session_path: &String) -> HashMap<String, net::Net> {
    eprintln!("[storage] Loading net from {}...", session_path);
    let mut f = File::open(&session_path).expect("[storage] File cannot be opened");
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).expect("[storage] File is broken")
}
