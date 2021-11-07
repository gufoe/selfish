
use hashbrown::HashMap;
use util;
use node::Node;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gene {
    pub received_alleles: u32,
    pub alleles: HashMap<u32, Allele>,
}

impl Gene {
    pub fn size(&self) -> usize { self.alleles.len() }
    pub fn has(&self, id: &Option<u32>) -> bool {
        return !id.is_none() && self.alleles.contains_key(&id.unwrap())
    }
    pub fn eval(&mut self, max_exp_count: f32, n: &mut Node, _gene: &str, max_pool_size: u32) {
        let score: f32 = n.score;
        let weight: f32;

        let mut scores: Vec<f32> = self.alleles.iter().map(|a| a.1.avg_score).collect();
        scores.push(score);
        let (_min, max, avg) = util::mma(&scores);
        if (max - avg).abs() < 0.01 {
            weight = 1.0;
        } else if score <= avg {
            weight = 0.01;
        } else {
            weight = ((score - avg) / (max - avg)).powi(2).max(0.01);
            // eprintln!("{} {} {} {}", score, avg, max, weight);
        }



        // Search for identical node
        if !self.has(&n.allele) {
            for (key, value) in &self.alleles {
                if n.is_equivalent_to(&value.node) {
                    // eprintln!("Corrected identical node in gene {} -> {}\n{}\n{}", gene, &key, &node, &value.node);
                    n.allele = Some(*key);
                    break;
                }
            }
        }


        // Registering allele if new
        if !self.has(&n.allele) {
            self.received_alleles+= 1;
            if n.allele.is_none() {
                n.allele = Some(util::rand_int(999999999));
            }
            self.alleles.insert(n.allele.unwrap(), Allele {
                tot_score: 0.0 as f64,
                avg_score: 0.0,
                exp_count: 0.0,
                node: n.clone(),
                count: 0,
                score_history: vec![],
            });
        }


        // Get node allele
        let a = self.alleles.get_mut(&n.allele.expect(&format!("An node in the alleles has NONE as id: {}", n))).expect("WTF");
        // let score = score.max(a.avg_score);
        a.exp_count+= weight;
        a.tot_score+= (score*weight) as f64;
        a.avg_score = a.tot_score as f32 / a.exp_count as f32;
        a.node.score = a.avg_score;
        a.score_history.push(score);
        a.count+= 1;
        if a.exp_count > max_exp_count {
            let div = max_exp_count as f64 / a.exp_count as f64;
            a.tot_score*= div;
            a.exp_count*= div as f32;
        }


        // Remove worst alleles
        while self.alleles.len() > max_pool_size as usize {
            // eprintln!("removing one allele: {} > {}", self.alleles.len(), max_pool_size);
            //  Kill the worst
            let mut wi: i32 = -1;
            let mut wv = 0.;
            for (i, v) in &self.alleles {
                if wi < 0 || v.avg_score < wv {
                    wi = *i as i32;
                    wv = v.avg_score;
                }
            }
            let i = wi as u32;
            self.alleles.remove(&i).expect(&format!("Cannot remove allele {}", i));
        }
    }
    pub fn select(&self) -> &Allele {
        let allele;

        // { // proportionate selection
        //     let mut leaderboard: Vec<(u32, f32)> = self.alleles.iter()
        //     .map(|(id, allele)| (*id, allele.avg_score))
        //     .collect();
        //
        //     // best -> worst
        //     // println!("da {:?}", leaderboard.clone());
        //     leaderboard.sort_by(|b, a| a.1.partial_cmp(&b.1).expect("Sorting failed"));
        //
        //     let len = leaderboard.len();
        //     let mut candidates = &mut leaderboard[..len.min(5)];
        //
        //     let mut sum = candidates.iter().fold(0.0, |sum, &val| sum+val.1);
        //     if sum > 0. {
        //         for n in &mut candidates.iter_mut() { n.1 /= sum }
        //     }
        //     for n in &mut candidates.iter_mut() {
        //         if util::maybe(n.1) {
        //             return &self.alleles[&n.0];
        //         }
        //     }
        //
        //     allele = &self.alleles[&candidates[0].0]
        // }

        {
            // rank selection
            let mut leaderboard: Vec<(u32, f32)> = self.alleles.iter()
            .map(|(id, allele)| (*id, allele.avg_score))
            .collect();

            // worst -> best
            // leaderboard.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            // random
            // util::shuffle(&mut leaderboard);

            // best -> worst
            // println!("da {:?}", leaderboard.clone());
            leaderboard.sort_by(|b, a| a.1.partial_cmp(&b.1).expect("Sorting failed 1"));
            // println!("a  {:?}", leaderboard.clone());
            let i = (util::rand_float(0., 1.).powi(3) * (leaderboard.len()) as f32) as usize;
            // let i = leaderboard.len()-1;
            // let i = 0;

            // eprintln!("i  {}/{}", i, leaderboard.len());
            allele = &self.alleles[&leaderboard[i].0];
        }

        // {
        //     // tournament selection
        //     let a = util::rand_hash(&self.alleles).unwrap();
        //     let b = util::rand_hash(&self.alleles).unwrap();
        //     let (max, min) = if a.avg_score > b.avg_score { (a, b) } else { (b, a) };
        //     allele = if util::maybe(0.8) { max } else { min }
        // }

        allele
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Allele {
    pub tot_score: f64,
    pub avg_score: f32,
    pub exp_count: f32,
    pub count: u32,
    pub score_history: Vec<f32>,
    pub node: Node,
}
