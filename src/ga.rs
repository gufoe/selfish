
use hashbrown::HashMap;
use std::collections::BTreeSet;
use std::fmt;
use util;
use node::Node;
use net::Net;
use gene::Gene;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GA {
    pub leaderboard: BTreeSet<Net>,
    pub genes: HashMap<String, Gene>,
    pub max_pool_size: u32,
    pub max_exp_count: f32,
    pub exp_count: usize,
    pub rec_count: usize,
    pub lead_size: usize,
    pub last_100: Vec<f32>,
    pub last_100_i: usize,
}


impl fmt::Display for GA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(GA (max_pool_size: {}):", self.max_pool_size).unwrap();
        for (name, gene) in &self.genes {
            writeln!(f, "- {} ({})", name, gene.alleles.len()).unwrap();
            for (id, allele) in &gene.alleles {
                writeln!(f, "  allele {} (exp: {}, avg: {})", id, allele.exp_count, allele.avg_score).unwrap();
                write!(f, "{}", allele.node).unwrap();
            }
        }
        Ok(())
    }
}

impl GA {
    pub fn new () -> GA {
        GA {
            leaderboard: BTreeSet::new(),
            genes: HashMap::new(),
            lead_size: 100,
            max_pool_size: 10,
            max_exp_count: 10.0,
            exp_count: 0,
            rec_count: 0,
            last_100: Vec::new(),
            last_100_i: 0,
        }
    }

    pub fn has_leader(&self) -> bool {
        self.leaderboard.len() > 0
    }

    pub fn size(&self) -> (usize, usize, f32) {
        let mut tot = 0;
        for (_, g) in &self.genes {
            tot+= g.size();
        }
        (self.genes.len(), tot, tot as f32 / self.genes.len() as f32)
    }

    pub fn is_initialized(&self) -> bool {
        !self.genes.is_empty()
    }

    pub fn has_gene(&self, name: &str) -> bool {
        self.genes.contains_key(name)
    }

    pub fn add_gene(&mut self, name: &str) {
        let g = Gene {
            received_alleles: 0,
            alleles: HashMap::new(),
        };
        self.genes.insert(name.to_string(), g);
    }

    pub fn eval(&mut self, name: &str, node: &mut Node) {
        if !self.has_gene(name) {
            self.add_gene(name);
            debug!("len {}", self.genes.len())
        }
        let gene = self.genes.get_mut(name).unwrap();
        gene.eval(self.max_exp_count, node, name, self.max_pool_size);
    }

    // Algorithm: select the best between two random alleles
    pub fn get_with_dependencies(&self, name: &str, vec: &mut HashMap<String, Node>) {
        if vec.contains_key(name) { return }
        let gene = self.genes.get(name).expect(&format!("Required unknown gene {}", name));
        let mut node;
        if false && util::maybe(0.01) {
            // crossover
            let a = &gene.select().node;
            let b = &gene.select().node;
            node = Node::new(a.is_input, a.is_output);
            node.reset = if util::maybe(0.5) { a.reset } else { b.reset };
            node.threshold = if util::maybe(0.5) { a.threshold } else { b.threshold };
            for link in &a.links {
                node.links.push(link.clone());
            }
            for link in &b.links {
                let l = node.links.iter().position(|x| x.out == link.out);

                if l.is_none() {
                    node.links.push(link.clone());
                } else {
                    if util::maybe(0.5) {
                        node.links.remove(l.unwrap());
                        node.links.push(link.clone());
                    }
                }
            }
        } else {
            let allele = gene.select().clone();
            // let scores: Vec<f32> = gene.alleles.iter().map(|a| a.1.avg_score).collect();
            // eprintln!("selecting node {} with score {:.3} best: {:?}", name, allele.avg_score, &util::mma(&scores));
            node = allele.node;
            assert!(!node.allele.is_none());
        }

        vec.insert(name.to_string(), node.clone());
        for link in node.links.iter_mut() {
            self.get_with_dependencies(&link.out, vec);
        }
    }

    pub fn record(&mut self, net: &mut Net) -> usize {
        // if self.leaderboard.contains(net) {
        //     // return 0;
        // }

        // Discard those worse than the worst leader
        // if util::maybe(0.0001) && self.leaderboard.len() >= self.lead_size {
        //     let best = self.leaderboard.iter().next().unwrap().score;
        //     let worst = self.leaderboard.iter().next_back().unwrap().score;
        //     if net.score < worst {
        //         // eprintln!("net {} did worse than {}", net.score, worst);
        //         return 0;
        //     }
        // }


        if self.last_100.len() == self.last_100_i {
            self.last_100.push(net.score)
        } else {
            self.last_100[self.last_100_i] = net.score;
        }
        self.last_100_i+= 1;
        self.last_100_i%= 500;

        // let rel_score = 1;
        // let rel_score;
        // let taw = self.get_taw();
        // if (net.score - taw.1).abs() < 0.0001 { rel_score = 0.5; }
        // else if net.score > taw.1 { rel_score = 0.5 + ((net.score - taw.1) / (taw.0 - taw.1)) * 0.5; }
        // else { rel_score = ((net.score - taw.2) / (taw.1 - taw.2)) * 0.5; }

        // if rel_score < 0.1 {
        //     return 0;
        // }



        let max = net.nodes.values().map(|n|n.score).fold(0./0., f32::max);
        let min = net.nodes.values().map(|n|n.score).fold(0./0., f32::min);

        if self.leaderboard.is_empty() || self.leaderboard.iter().next_back().unwrap().score < net.score {
            eprintln!("saving {:.2}   max={:.2} min={:.2}", net.score, max, min);
            self.leaderboard.insert(net.clone());
        }

        // eprintln!("corrected to {} given real score {}, taw: {:?}     {} / {}", rel_score, net.score, taw, (net.score - taw.1), (taw.0 - taw.1));
        for (name, mut node) in &mut net.nodes {
            self.eval(name, &mut node)
        }

        while self.leaderboard.len() > self.lead_size {
            let last = self.leaderboard.iter().next_back().unwrap().clone();
            self.leaderboard.remove(&last);
        }

        self.rec_count+= 1;
        self.rec_count
    }

    #[allow(dead_code)]
    pub fn get_taw(&self) -> (f32, f32, f32) {
        let mut taw = (self.last_100[0], self.last_100[0], self.last_100[0]);
        for i in 1..self.last_100.len() {
            let v = self.last_100[i];
            taw.1+= v;
            if v > taw.0 { taw.0 = v; }
            if v < taw.2 { taw.2 = v; }
        }
        taw.1/= self.last_100.len() as f32;
        taw
    }

    pub fn next_id(&mut self) -> usize {
        self.exp_count+= 1;
        self.exp_count-1
    }
}
