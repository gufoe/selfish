use yaml_rust::Yaml;
use ga::GA;
use net;
use util;
use dataset::{Dataset,DsMode};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GaPool {
    pub gas: Vec<GA>,
    pub debug: bool,
    pub mrate: f32,
    pub samples: u32,
    pub lead_size: usize,
    pub tset_slice: usize,
    pub vset_slice: usize,
    pub cur_vset: usize,
    pub old_vset: usize,
}
impl GaPool {
    pub fn is_switching(&self) -> bool {
        self.gas.len() > 1
    }
    pub fn fix_overfitting(&mut self, opts: &Yaml, ds: &Dataset) {
        if self.debug {
            eprintln!("[gp] Checking overfitting, is_switching: {}", self.is_switching());
            for (i, g) in self.gas.iter().enumerate()  { eprintln!("[gp] before: {}: {}/{} {:?}", i, g.rec_count, g.exp_count, g.size()); }
        }
        if self.is_switch_complete(opts) {
            if self.debug { eprintln!("[gp] switch completed"); }
            // assert_eq!(self.gas.len(), 2);
            while self.gas.len() > 1 {
                self.gas.remove(0);
            }
        }
        if self.is_overfitting(opts, ds.clone()) {
            if self.debug { eprintln!("[gp] overfitting detected"); }
            assert_eq!(self.gas.len(), 1);
            self.incr_vset(opts, ds);
        }

        if self.debug {
            for (i, g) in self.gas.iter().enumerate()  { eprintln!("[gp] after: {}: {}/{} {:?}", i, g.rec_count, g.exp_count, g.size()); }
        }
    }

    pub fn incr_vset(&mut self, opts: &Yaml, ds: &Dataset) -> usize {
        self.old_vset = self.cur_vset;
        self.cur_vset = self.next_vset(opts, ds.clone());
        self.gas.push(GaPool::gen_ga(opts));
        self.cur_vset
    }
    pub fn is_switch_complete(&self, opts: &Yaml) -> bool {
        if !self.is_switching() { return false }

        let min_transfer = util::opt_i64(&opts["master"]["overfitting"], "min_transfer", 200) as usize;
        let count = self.dst().rec_count;
        if self.debug { eprintln!("[gp] finalizing? {}/{}", count, min_transfer); }
        count > min_transfer
    }
    pub fn is_overfitting(&self, opts: &Yaml, mut ds: Dataset) -> bool {
        if self.is_switching() { return false }
        if !self.src().has_leader() { return false }
        let min_grow = util::opt_i64(&opts["master"]["overfitting"], "min_grow", 100) as usize;
        let min_transfer = util::opt_i64(&opts["master"]["overfitting"], "min_transfer", 100) as usize;
        if self.src().rec_count < min_transfer+min_grow {
            if self.debug { eprintln!("[gp] OF detection not ready: {}/{}", self.src().rec_count, min_transfer+min_grow); }
            return false;
        }

        // Take the leader and test it on the next validation set;
        // if the score on the next vset is way lower, then there is overfitting
        // and a new generation should be started
        ds.mode = DsMode::Val;
        //
        // eval_net(&mut leader, &val_dset, &(slice_size*rand_vset..slice_size*(rand_vset+1)).collect(), 0, false);
        // let next_score = leader.score;
        //
        // if self.debug { eprintln!("[gp] Current vs next: {} {}", cur_score, next_score); }
        // return cur_score*of_threshold > next_score
        true
    }
    pub fn src(&self) -> &GA {
        self.gas.iter().next().unwrap()
    }
    pub fn dst(&self) -> &GA {
        self.gas.iter().last().unwrap()
    }

    pub fn record(&mut self, net: &mut net::Net) -> usize {
        let ga;
        if net.vset == self.cur_vset {
            let len = self.gas.len();
            ga = &mut self.gas[len-1];
        } else if net.vset == self.old_vset && self.gas.len() == 2 {
            ga = &mut self.gas[0];
        } else {
            if self.debug { eprintln!("[gp] Rejecting old old_vset"); }
            return 0;
        }
        ga.record(net)
    }

    pub fn next_vset(&self, opts: &Yaml, mut ds: Dataset) -> usize {
        ds.mode = DsMode::Val;
        let slice_size = opts["worker"]["vset_slice"].as_i64().unwrap() as usize;
        let slices_available = ds.len()/slice_size;
        (self.cur_vset + 1) % slices_available
    }
    #[allow(dead_code)]
    pub fn rand_vset(&self, opts: &Yaml, mut ds: Dataset) -> usize {
        ds.mode = DsMode::Val;
        let slice_size = opts["worker"]["vset_slice"].as_i64().unwrap() as usize;
        let slices_available = ds.len()/slice_size;
        util::rand_int(slices_available as u32) as usize
    }

    pub fn gen_ga(opts: &Yaml) -> GA {
        let mut ga = GA::new();
        GaPool::reconfig_ga(&mut ga, opts);
        ga
    }

    pub fn reconfig_ga(ga: &mut GA, opts: &Yaml) {
        ga.max_pool_size = util::opt_i64(&opts["master"], "max_pool_size", 10) as u32;
        ga.max_exp_count = util::opt_f32(&opts["master"], "max_exp_count", 10.0);
        ga.lead_size = util::opt_i64(&opts["master"], "lead_size", 10) as usize;
    }

    pub fn new(opts: &Yaml) -> GaPool {
        let mut ga = GA::new();
        GaPool::reconfig_ga(&mut ga, opts);

        GaPool {
            gas: vec![ga],
            debug: false,

            mrate: util::opt_f32(&opts["worker"], "mrate", 0.1),
            samples: util::opt_i64(&opts["worker"], "samples", 10) as u32,
            lead_size: util::opt_i64(&opts["worker"], "lead_size", 10) as usize,
            tset_slice: util::opt_i64(&opts["worker"], "tset_slice", 10) as usize,
            vset_slice: util::opt_i64(&opts["worker"], "vset_slice", 10) as usize,
            cur_vset: 0,
            old_vset: 0,
        }
    }

    pub fn reconfigure(&mut self, opts: &Yaml) {
        self.mrate = util::opt_f32(&opts["worker"], "mrate", 0.1);
        self.samples = util::opt_i64(&opts["worker"], "samples", 10) as u32;
        self.lead_size = util::opt_i64(&opts["worker"], "lead_size", 10) as usize;
        self.tset_slice = util::opt_i64(&opts["worker"], "tset_slice", 10) as usize;
        self.vset_slice = util::opt_i64(&opts["worker"], "vset_slice", 10) as usize;
    }

    pub fn dst_size(&self) -> (usize, usize, f32) {
        self.dst().size()
    }
    pub fn src_size(&self) -> (usize, usize, f32) {
        self.src().size()
    }

    pub fn gen_net(&self, input: &Vec<String>, output: &Vec<String>) -> net::Net {
        let mut n = net::Net::from_ga(self.src(), input, output);
        n.vset = self.cur_vset;
        n
    }

    #[allow(dead_code)]
    pub fn cloned_src(&self) -> GA {
        self.gas[0].clone()
    }

    pub fn gen_exp_id(&mut self) -> usize {
        self.gas[0].next_id()
    }
}
