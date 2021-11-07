
use hashbrown::HashMap;
// #[allow(unused_imports)]
// use std::f64::consts::E;
use std::cmp::Ordering;

use util;
use ga::GA;
use node::Node;
use link::Link;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Net {
    pub vset: usize,
    pub score: f32,
    pub success: f32,
    pub nodes: HashMap<String, Node>,
    pub actions: HashMap<u32, Vec<(f32, String)>>,
    pub time: u32,
    pub id: usize,
    // pub log: Vec<String>,
}

impl Ord for Net {
    fn cmp(&self, other: &Net) -> Ordering {
        if self.score > other.score { return Ordering::Less }
        if self.score < other.score { return Ordering::Greater }
        Ordering::Equal
    }
}

impl PartialOrd for Net {
    fn partial_cmp(&self, other: &Net) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Net {}

impl PartialEq for Net {
    fn eq(&self, other: &Net) -> bool {
        self.score == other.score
    }
}


impl Net {

    pub fn new () -> Net {
        Net {
            time: 0,
            score: 0.,
            success: 0.,
            nodes: HashMap::new(),
            actions: HashMap::new(),
            id: 0,
            vset: 0,
            // log: Vec::new(),
        }
    }

    pub fn from_ga (ga: &GA, input: &Vec<String>, output: &Vec<String>) -> Net {
        // if util::maybe(0.1) && !ga.leaderboard.is_empty() {
        //     let r = (util::rand_float(0., 1.).powi(4) * ga.leaderboard.len() as f32) as usize;
        //     return ga.leaderboard.iter().skip(r as usize).next().unwrap().clone()
        // }

        let mut n = Net::new();
        n.add_genes(ga, input, true, false);
        n.add_genes(ga, output, false, true);

        // if !ga.is_initialized() {
        //     // Not working with noise
        //     // for _ in 0..10 {
        //     //     n.mutate(ga, 0.1);
        //     // }
        //
        //     // Completely changes the results in better
        //     for _input in input.iter() {
        //         for _output in output.iter() {
        //             if util::maybe(0.8) { continue }
        //             let link = Link::new(_output.to_string());
        //             n.node(&_input.to_string()).links.push(link);
        //         }
        //     }
        //     // for _ in 0..10000 {
        //     //     n.mutate(0.2);
        //     // }
        // }

        n
    }

    fn add_genes (&mut self, ga: &GA, genes: &Vec<String>, is_input: bool, is_output: bool) {
        for gene in genes.iter() {
            if ga.has_gene(gene) {
                ga.get_with_dependencies(gene, &mut self.nodes);
            } else {
                self.nodes.insert(gene.to_string(), Node::new(is_input, is_output));
            }
        }
    }

    pub fn reset (&mut self) {
        self.time = 0;
        self.actions.clear();
        for (_gene, n) in &mut self.nodes {
            n.reset();
        }
    }
    pub fn reset_score (&mut self) {
        self.score = 0.0;
        self.success = 0.0;
        for (_gene, n) in &mut self.nodes {
            n.reset_score();
        }
    }

    pub fn input_map (&self) -> HashMap<String, Vec<(String, usize)>> {
        let mut map = HashMap::new();
        self.nodes.iter().for_each(|(name, node)| {
            node.links.iter().enumerate().for_each(|(link_i, link)| {
                if !map.contains_key(&link.out) {
                    map.insert(link.out.to_string(), vec![(name.to_string(), link_i)]);
                } else {
                    map.get_mut(&link.out).unwrap().push((name.to_string(), link_i));
                }
            });
        });
        map
    }

    pub fn feedback (&mut self, node: String, score: f32, input_map: &HashMap<String, Vec<(String, usize)>>) {
        let mut updated = std::collections::HashSet::new();
        let mut todo = std::collections::HashSet::new();
        todo.insert(node);
        while !todo.is_empty() {
            let node: String = util::pop(&mut todo);
            if updated.contains(&node) { continue; }
            updated.insert(node.to_string());

            self.nodes.get_mut(&node).unwrap().score+= score;
            input_map.get(&node).unwrap_or(&vec![]).iter().for_each(|input| {
                {
                    let mut link = &mut self.node(&input.0).links[input.1];
                    link.score+= score;
                }
                if !updated.contains(&input.0.to_string()) {
                    todo.insert(input.0.to_string());
                }
            });
        }
        // updated.iter().for_each(|name| {
        //     self.node(name).compute_score();
        // })
    }

    pub fn cycle (&mut self, ticks: u32, trigger: bool) -> HashMap<String, Node> {
        let mut active = HashMap::new();

        for _ in 0..ticks {
            active = self.tick();
            if trigger && !active.is_empty() {
                // eprintln!("active: {} {} {:?}", self.actions.is_empty(), trigger, &active);
                break
            }
        }
        active
    }

    pub fn signal(&mut self, time: u32, name: String, value: f32) {

        if !self.actions.contains_key(&time) {
            self.actions.insert(time, vec![]);
        }
        self.actions.get_mut(&time).unwrap().push((value, name));
    }

    fn tick (&mut self) -> HashMap<String, Node> {
        let mut touched: HashMap<String, (f32, f32)> = HashMap::new();
        let cur_time = self.time as usize;
        debug!("Tick [{}]", cur_time);

        // Calc new values
        if let Some(actions) = self.actions.remove(&self.time) {
            // if !actions.is_empty() {println!("Actions: {}", actions.len());}
            // println!("{}, {:?}", self.time, actions.clone());
            for action in actions.into_iter() {
                let node = self.node(&action.1);
                node.value+= action.0;
                touched.insert(action.1.to_string(), (node.value, node.threshold));

                debug!("- action {} + {} -> {}", action.1, action.0, node);
            }
        }
        debug!("touched: {}", touched.len());

        let mut ret = HashMap::new();

        for (name, (value, threshold)) in touched {
            // NOTE: adding noise
            // value = value
            // * util::rand_float(0.98, 1.03)
            // + util::rand_float(-0.01, 0.01);
            debug!("Activate {}? {} {}", name.to_string(), value, threshold);
            if value <= threshold { continue }
            debug!("Activated!! {} {}", name.to_string(), value);

            let cloned_node = self.node(&name).clone();
            let val = cloned_node.activated();
            // Fire to links
            for syn in cloned_node.links.iter() {
                assert!(syn.delay > 0);
                let t = cur_time as u32 + syn.delay;
                if !cloned_node.is_input {
                    debug!("+ action from {} -> {} at tick {}->{}: {} * {}", name.to_string(), syn.out, self.time, t, syn.weight, val);
                }
                // let w = 1.0/(1.0 + syn.weight.abs()).powi(2);
                self.signal(t, syn.out.to_string(), syn.weight * val);
                // self.signal(t+1, syn.out.to_string(), syn.weight * val);
                // self.signal(t+2, syn.out.to_string(), syn.weight * val);
            }

            // Return activated outputs
            if cloned_node.is_output {
                ret.insert(name.to_string(), cloned_node);
            }

            // Reset value
            let mut node = self.node(&name);
            node.value = node.reset;
            node.last_fired = cur_time as usize;
        }
        debug!("actions: {:?}", self.actions.clone());

        // Important!
        self.time+= 1;
        ret
    }

    pub fn node(&mut self, id: &String) -> &mut Node {
        self.nodes.get_mut(id).expect(&format!("cannot find node {}", id))
    }

    #[allow(dead_code)]
    pub fn value_of(&self, id: &String) -> f32 {
        self.nodes.get(id).expect(&format!("cannot find node {}", id)).value
    }


    pub fn mutate(&mut self, _ga: &GA, _mrate: f32) {

        let name = self.mutable_gene();
        let link_num = self.nodes[&name].links.len() as f32;
        loop {
            if util::maybe(1.0 / (2.0 + link_num).powi(2)) {
                let target = self.rand_gene_ni();
                let node = self.node(&name);
                node.mut_link_new(target);
                break;
            }

            if util::maybe(0.05) && link_num > 0.0 {
                let node = self.node(&name);
                node.mut_link_delay();
                break;
            }

            if util::maybe((link_num / 80.0).powi(2)) {
                let node = self.node(&name);
                node.mut_link_del();
                break;
            }

            if util::maybe(0.05) && link_num > 7.0 {
                let mut split = Node::new(false, false);
                let split_gene = util::rand_string(5);
                {
                    let node = self.node(&name);
                    split.links = node.links.clone();
                    node.links.push(Link::new(split_gene.clone()));
                }
                self.nodes.insert(split_gene, split);
                break;
            }


            let node = self.node(&name);
            node.mut_link_weight();
            break;
        }

        self.node(&name).allele = None;
    }

    // pub fn mutate(&mut self, ga: &GA, mrate: f32) {
    //     // panic!("cannot mutate");
    //     // use noise::{Perlin};
    //     let mut mutated = false;
    //     // let perlin = Perlin::new();
    //     while !mutated {
    //         // let now = util::now();
    //         while util::maybe(mrate * 0.1) { //*perlin.get([now as f64/500000.0, 37.7, 2.8]) as f32)) {
    //             mutated|= self.mutate_link_del();
    //         }
    //         while util::maybe(mrate * 0.1) { // *perlin.get([now as f64/500000.0, 36547.7, 2000.8]) as f32)) {
    //             mutated|= self.mutate_link_new(ga);
    //         }
    //         while util::maybe(mrate * 0.01) { // *perlin.get([now as f64/500000.0, 37.7, 200000.8]) as f32)) {
    //             mutated|= self.mutate_split();
    //         }
    //         // while util::maybe(mrate*(0.02)) { // *perlin.get([now as f64/500000.0, 1237.7, 1000002.8]) as f32)) {
    //         //     mutated|= self.mutate_discharge();
    //         // }
    //         // while util::maybe(mrate*(0.05)) { // *perlin.get([now as f64/500000.0, 432137.7, 57777.8]) as f32)) {
    //         //     mutated|= self.mutate_mrate();
    //         // }
    //         // while util::maybe(mrate*(0.01)) { // *perlin.get([now as f64/500000.0, 37.7, 2.8]) as f32)) {
    //         //     mutated|= self.mutate_reset();
    //         // }
    //         // while util::maybe(mrate) { // *perlin.get([now as f64/500000.0, 37.7, 2.8]) as f32)) {
    //         //     mutated|= self.mutate_threshold();
    //         // }
    //         while util::maybe(mrate) { // *perlin.get([now as f64/500000.0, 3654457.7, 2.8]) as f32)) {
    //             mutated|= self.mutate_link_weight();
    //         }
    //         while util::maybe(mrate * 0.01) { // *perlin.get([now as f64/500000.0, 33457.7, 2.8]) as f32)) {
    //             mutated|= self.mutate_link_delay();
    //         }
    //     }
    //     // self.clean();
    // }

    pub fn links_num(&self) -> usize {
        let mut n = 0;
        for node in self.nodes.values() {
            n+= node.links.len();
        }
        n
    }

    #[allow(dead_code)]
    fn mutate_split(&mut self) -> bool {
        let mut split = Node::new(false, false);
        let split_gene = util::rand_string(5);

        {

            let nt = self.mutable_node_nt();
            if nt.is_none() { return false; }
            let source = &mut nt.unwrap();

            {
                let (_, link) = source.rand_link();
                let split_delay = util::rand_int(link.delay+1);

                let mut split_link = Link::new(link.out.to_string());
                split_link.delay = split_delay.max(1);
                // split_link.weight = 1.;
                split.links.push(split_link);

                link.out = split_gene.to_string();
                link.delay = (link.delay - split_delay).max(1);
            }

            source.allele = None;
        }
        self.nodes.insert(split_gene, split);

        true
    }

    #[allow(dead_code)]
    fn mutate_reset(&mut self) -> bool {
        let n = self.mutable_node();
        n.reset = -Net::mut_weight(n.reset).abs();
        n.allele = None;
        // n.last_edit = Some("mutate_reset".to_string());
        true
    }

    #[allow(dead_code)]
    fn mutate_threshold(&mut self) -> bool {
        let n = self.mutable_node();
        if n.is_input { return false }
        n.threshold = Net::mut_weight(n.threshold).abs();
        n.allele = None;
        // n.last_edit = Some("mutate_threshold".to_string());
        true
    }

    #[allow(dead_code)]
    fn mutate_link_new(&mut self, _ga: &GA) -> bool {
        let source_id = self.mutable_gene();
        let target_id;
        if util::maybe(0.9) {
            let source = self.nodes[&source_id].clone();
            if source.links.is_empty() { return false }

            let inter_id = source.links[util::rand_usize(source.links.len())].out.clone();
            let inter = self.nodes[&inter_id].clone();
            if inter.links.is_empty() { return false }

            target_id = inter.links.get(util::rand_usize(inter.links.len())).expect("not found").out.clone();
        } else {
            target_id = self.mutable_gene_ni();
        }
        let n = self.node(&source_id);
        n.links.push(Link::new(target_id.to_string()));
        n.allele = None;
        true
    }

    #[allow(dead_code)]
    fn mutate_link_del(&mut self) -> bool {
        let n = self.mutable_node_nt();
        if n.is_none() { return false; }
        let n = n.unwrap();
        let i = util::rand_int(n.links.len() as u32) as usize;
        n.links.remove(i);
        // eprintln!("new link added!! from {} to {}\n", gene, target);
        n.allele = None;
        // n.last_edit = Some("mutate_link_del".to_string());
        true
    }

    #[allow(dead_code)]
    fn mutate_link_weight(&mut self) -> bool {
        let n = self.mutable_node_nt();
        if n.is_none() { return false; }
        let n = n.unwrap();
        {
            let (_, link) = n.rand_link();
            link.weight = Net::mut_weight(link.weight);
        }
        n.allele = None;
        // n.last_edit = Some("mutate_link_weight".to_string());
        true
    }

    #[allow(dead_code)]
    fn mutate_link_delay(&mut self) -> bool {
        let n = self.mutable_node_nt();
        if n.is_none() { return false; }
        let n = n.unwrap();
        let old: u32;
        let cur: u32;

        {
            let (_, link) = n.rand_link();
            old = link.delay;
            link.delay = (link.delay as i32 + util::rand_intr(-2, 3)).max(1) as u32;
            cur = link.delay;
        }

        if cur != old {
            n.allele = None;
            // n.last_edit = Some(format!("mutate_link_delay {} -> {}", old, cur).to_string());
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn rand_node (&mut self) -> &mut Node {
        let gene: String = self.rand_gene();
        self.node(&gene)
    }
    fn mutable_node (&mut self) -> &mut Node {
        let gene: String = self.mutable_gene();
        self.node(&gene)
    }

    fn mutable_node_nt (&mut self) -> Option<&mut Node> {
        let mut gene = "".to_string();

        for _ in 0..100 {
            let g = self.mutable_gene();
            if !self.node(&g).is_terminal() {
                gene = g;
                break;
            }
        }

        if gene == "" { None }
        else {
            Some(self.node(&gene))
        }
    }

    fn mutable_gene_ni (&mut self) -> String {
        let nodes = self.nodes.clone();
        self.nodes = nodes.iter().filter(|x| !x.1.is_input).map(|x| (x.0.to_string(), x.1.clone())).collect();
        let g = self.mutable_gene();
        self.nodes = nodes;
        return g;
    }

    fn rand_gene_ni (&mut self) -> String {
        let nodes = self.nodes.clone();
        self.nodes = nodes.iter().filter(|x| !x.1.is_input).map(|x| (x.0.to_string(), x.1.clone())).collect();
        let g = self.rand_gene();
        self.nodes = nodes;
        return g;
    }

    fn mutable_gene (&self) -> String {
        let mut gene: String = self.rand_gene();
        let mut val = 99999.0;

        for _ in 0..20 {
            let name = self.rand_gene();
            let n = self.nodes.get(&name).unwrap();
            if n.score < val {
                gene = name;
                val = n.score;
            }
        }
        // println!("worst found is {} with score {}", gene, val);
        return gene;
    }

    fn rand_gene (&self) -> String {
        return util::rand_hash_key(&self.nodes).unwrap().to_string();
    }

    fn mut_weight(w: f32) -> f32 {
        // REVIEW
        return w + util::rand_float(-0.1, 0.1);
        // if util::maybe(0.001) {
        //     return util::rand_float(-10., 10.);
        // } else if w == 0. || util::maybe(0.002) {
        //     return w + util::rand_float(-10., 10.);
        // } else {
        //     if util::maybe(0.5) {
        //         return w * util::rand_float(0.8, 1.3);
        //     } else {
        //         return w * util::rand_float(0.5, 5.);
        //     }
        // }
    }


    #[allow(dead_code)]
    pub fn clean(&mut self) {
        use std::collections::HashSet;
        let mut useless_fuckers = HashSet::new();
        let mut parsed = HashSet::new();
        let mut toparse = HashSet::new();

        for (name, node) in self.nodes.iter() {
            if !node.is_output && !node.is_input {
                useless_fuckers.insert(name.to_string());
            }
        }

        for name in self.nodes.iter().filter(|x| x.1.is_input) {
            toparse.insert(name.0.to_string());
        }

        while !toparse.is_empty() {
            let name = toparse.iter().next().unwrap().clone();
            if parsed.contains(&name) { continue }

            toparse.remove(&name);
            parsed.insert(name.to_string());
            useless_fuckers.remove(&name);

            let node = &self.nodes[&name];
            for link in node.links.iter() {
                if parsed.contains(&link.out) { continue }
                if toparse.contains(&link.out) { continue }
                toparse.insert(link.out.to_string());
            }
        }

        for name in useless_fuckers {
            println!("removing useless fucker [{}]", name);
            self.nodes.remove(&name);
        }
    }

}
