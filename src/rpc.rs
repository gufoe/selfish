use std::net::TcpListener;
use std::thread;
use com::TcpWrapper;

pub type NetDescription = (
    String, // ga pool name
    Vec<String>, // input names
    Vec<String>, // output names
);


use std::sync::{Mutex, Arc, RwLock};
use yaml_rust::Yaml;
use getopts::Matches;
use hashbrown::HashMap;

use net::Net;
use util;
use dataset::Dataset;
use gapool::GaPool;


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Opts {
    pub mrate: f32,
    pub samples: u32,
    pub lead_size: usize,
    pub tset_slice: usize,
    pub vset_slice: usize,
}

#[derive(Clone)]
struct GaServer {
    pub opts: Yaml,
    ga_pools: Arc<RwLock<HashMap<String, GaPool>>>,
    id: usize,
    out_lock: Arc<Mutex<()>>,
    pop_size: usize,
    ds: Dataset,
}

impl GaServer {
    fn check_gap_existence(&self, name: &String) {
        let exists = self.ga_pools.read().unwrap().contains_key(name);

        if !exists {
            let gap = GaPool::new(&self.opts);
            self.ga_pools.write().unwrap().insert(name.to_string(), gap);
        }
    }

    fn gen_net(&self, genes: &NetDescription) -> Net {
        // let (genes) = self.ds.describe();
        self.check_gap_existence(&genes.0);
        let mut net: Net = self.ga_pools.read().unwrap()[&genes.0].gen_net(&genes.1, &genes.2);
        net.id = self.ga_pools.write().unwrap().get_mut(&genes.0).unwrap().gen_exp_id();
        net
    }

    fn get_nets(&self, genes: NetDescription) -> Vec<Net> {
        let mut vec = vec![];
        for _ in 0..self.pop_size {
            vec.push(self.gen_net(&genes));
        }
        return vec
    }




    fn record_and_get(self, nets: HashMap<String, Net>, genes: Vec<NetDescription>) -> (HashMap<String, Vec<Net>>, Opts) {
        self.clone().record(nets);
        self.get(genes)
    }

    fn get(self, genes: Vec<NetDescription>) -> (HashMap<String, Vec<Net>>, Opts) {
        // eprintln!("[s {}] get_net", self.id);
        let mut ret_nets: HashMap<String, Vec<Net>> = HashMap::new();
        for nd in genes {
            ret_nets.insert(nd.0.to_string(), self.get_nets(nd));
        }
        (ret_nets, Opts {
            mrate: util::opt_f32(&self.opts["worker"], "mrate", 0.2),
            samples: util::opt_i64(&self.opts["worker"], "samples", 20) as u32,
            lead_size: util::opt_i64(&self.opts["worker"], "lead_size", 2) as usize,
            tset_slice: util::opt_i64(&self.opts["worker"], "tset_slice", 100) as usize,
            vset_slice: util::opt_i64(&self.opts["worker"], "vset_slice", 1000) as usize,
        })
    }

    fn record(self, nets: HashMap<String, Net>) {
        // eprintln!("[s {}] recording {} score: {}", self.id, net.id, net.score);
        let guard = self.out_lock.lock().unwrap();
        for (name, mut net) in nets {
            self.check_gap_existence(&name);
            let mut gapsw = self.ga_pools.write().unwrap();
            let gapw = gapsw.get_mut(&name).unwrap();

            // let time = util::time();
            let count = gapw.record(&mut net);
            if count > 0 {
                // let diff = time.diff();
                let size = gapw.src_size();
                let size2 = gapw.dst_size();




                println!("{}\t{:.3}\t{:}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    count,
                    net.success,
                    net.score,
                    util::now(),
                    name,
                    size.0,
                    size.1,
                    size.2,
                    size2.0,
                    size2.1,
                    size2.2,
                    net.nodes.len(),
                    gapw.gas.len(),
                    gapw.cur_vset,
                    net.links_num()
                );
            } else {
                eprintln!("dup {}", net.score);
            }
        }
        drop(guard);
        // eprintln!("[s {}] Recording {} took {}", self.id, count, diff);

    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Cmd {
    Get ( Vec<NetDescription> ),
    RecordAndGet (HashMap<String, Net>, Vec<NetDescription>),
}

pub fn server(args: &Matches, opts: &Yaml, ga_pools: Arc<RwLock<HashMap<String, GaPool>>>, ds: Dataset) {
    let server = GaServer {
        id: 0,
        ga_pools,
        ds,
        opts: opts.clone(),
        pop_size: util::opt_i64(&opts["master"], "pop_size", 3) as usize,
        out_lock: Arc::new(Mutex::new(())),
    };


    let port = util::arg(&args, "serve", 54321);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    eprintln!("[s] listening for connections...");
    for stream in listener.incoming() {
        eprintln!("[s] new connection");
        let server = server.clone();
        thread::spawn(move || {
            let stream = stream.unwrap();
            let mut com = TcpWrapper{stream};

            loop {
                let server = server.clone();
                let cmd: Cmd = com.recv();
                // eprintln!("[s] Rec: {:?}", cmd);
                match cmd {
                    Cmd::Get(x) => {
                        com.send(&server.get(x));
                    },
                    Cmd::RecordAndGet(res, def) => {
                        com.send(&server.record_and_get(res, def));
                    }
                }
            }
        });
    }
}
