use util;

#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    pub weight: f32,
    pub delay: u32,
    pub out: String,
    pub score: f32,
}
impl Link {
    pub fn new (out: String) -> Link {
        Link {
            out,
            score: 0.0,
            weight: 1.0,
            delay: util::rand_intr(2, 3) as u32,
            // delay: 2,
        }
    }
}
