use std::fmt;
use link::Link;
use util;
use std::cmp::PartialEq;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub allele: Option<u32>,
    pub reset: f32,
    pub value: f32,
    pub is_input: bool,
    pub is_output: bool,
    pub threshold: f32,
    pub links: Vec<Link>,
    pub score: f32,
    pub last_fired: usize,
}

impl PartialEq for Node {
    fn eq(&self, n: &Node) -> bool {
        self.allele == n.allele
    }
}

impl Node {
    pub fn new (is_input: bool, is_output: bool) -> Node {
        Node {
            allele: None,
            value: 0.,
            is_input,
            is_output,
            reset: 0.0,
            threshold: if is_input { 0. } else { 1. },
            links: vec![],
            last_fired: 0,
            score: 0.0,
        }
    }

    pub fn activated(&self) -> f32 {
        let diff = self.value - self.threshold;
        if diff <= 0.0 {
            return 0.0;
        }
        // 1.0
        // self.value
        // self.value - self.threshold
        let v = (self.value - self.threshold).powi(2).min(1.0);
        // (1.0 + (self.value - self.threshold).powi(2))
        // let v = 1.0 / (1.0+2_f32.powf(- 10.0 * diff));
        // let v = 1.0;
        assert!(v >= 0.0);
        assert!(v <= 1.0);
        return v;
    }

    pub fn is_terminal(&self) -> bool {
        self.links.is_empty()
    }

    pub fn rand_link(&mut self) -> (usize, &mut Link) {
        let i = util::rand_int(self.links.len() as u32) as usize;
        (i, self.links.get_mut(i).unwrap())
    }

    pub fn reset (&mut self) {
        self.last_fired = 0;
        self.value = self.reset
    }

    pub fn reset_score(&mut self) {
        self.score = 0.0;
        self.links.iter_mut().for_each(|l| {l.score = 0.0;});
    }

    // pub fn compute_score(&mut self) {
    //     self.score = self.links.iter().map(|l| l.score).sum();
    //     self.score/= self.links.len().max(1) as f32
    // }

    pub fn is_equivalent_to(&self, node: &Node) -> bool {
        if self.reset != node.reset { return false }
        if self.threshold != node.threshold { return false }
        if self.links.len() != node.links.len() { return false }
        for (i, link) in self.links.iter().enumerate() {
            match node.links.get(i) {
                None => return false,
                Some(l) => {
                    if link.weight != l.weight { return false }
                    if link.delay != l.delay { return false }
                    if link.out != l.out { return false }
                }
            }
        }
        return true
    }

    #[allow(dead_code)]
    pub fn mut_link_weight (&mut self) -> bool {
        if self.links.is_empty() { return false }
        {
            let mut link: &mut Link = self.links.iter_mut().fold(None, |min, x| match min {
                None => Some(x),
                Some(y) => Some(if x.score < y.score { x } else { y }),
            }).expect("no min");
            link.weight+= util::rand_float(-0.1, 0.1);
            link.weight*= util::rand_float(0.8, 1.2);
        }
        self.allele = None;
        // n.last_edit = Some("mutate_link_weight".to_string());
        true
    }

    #[allow(dead_code)]
    pub fn mut_link_delay (&mut self) -> bool {
        if self.links.is_empty() { return false }
        {
            let (_, link) = self.rand_link();
            link.delay+= util::rand_int(6)-3;
            link.delay = link.delay.max(1);
        }
        self.allele = None;
        // n.last_edit = Some("mutate_link_weight".to_string());
        true
    }

    #[allow(dead_code)]
    pub fn mut_link_new (&mut self, target: String) -> bool {
        self.links.push(Link::new(target));
        true
    }

    #[allow(dead_code)]
    pub fn mut_link_del (&mut self) -> bool {
        if self.links.is_empty() {
            return false;
        }
        let i = util::rand_int(self.links.len() as u32) as usize;
        self.links.remove(i);
        self.allele = None;
        true
    }

    // pub fn is_too_old(&self) -> bool {
    //     let max_age = 1000.0;
    //     return util::maybe(self.avg_score.max(max_age)/max_age);
    // }
}


impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "            node: (allele: {:?}, links: {}, reset: {}, threshold: {})", self.allele, self.links.len(), self.reset, self.threshold).unwrap();
        for link in &self.links {
            writeln!(f, "            -> (out: {}, delay: {}, weight: {})", link.out, link.delay, link.weight).unwrap();
        }
        Ok(())
    }
}
