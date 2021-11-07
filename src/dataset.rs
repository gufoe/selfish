use rulinalg::matrix::{BaseMatrix, Matrix};
use mnist::MnistBuilder;
use std::sync::Arc;
use net::Net;
use util;
// pub trait Dataset {
//     fn to_net(&self, i: usize, net: &mut Net);
//     fn definition(&self) -> (Vec<String>, Vec<String>);
//     fn len(&self) -> usize;
//     fn output_at(&self, i: usize) -> &Vec<f32>;
//     fn print_input(&self, i: usize);
// }

#[derive(Clone,PartialEq)]
pub enum DsMode {
    Train,
    Val,
    Test,
}

#[derive(Clone)]
pub struct Dataset {
    pub mode: DsMode,
    train_input: Arc<Vec<Matrix<f32>>>,
    train_output: Arc<Vec<Vec<f32>>>,
    pub train_labels: Arc<Vec<u8>>,
    val_input: Arc<Vec<Matrix<f32>>>,
    val_output: Arc<Vec<Vec<f32>>>,
    pub val_labels: Arc<Vec<u8>>,
    test_input: Arc<Vec<Matrix<f32>>>,
    test_output: Arc<Vec<Vec<f32>>>,
    pub test_labels: Arc<Vec<u8>>,
}

impl Dataset {
    #[allow(dead_code)]
    pub fn describe(&self) -> (Vec<String>, Vec<String>) {
        let mut input = vec![];
        let mut output = vec![];
        for i in 0..28*28 { input.push(format!("in-{}", i)); }
        for i in 0..10 { output.push(format!("out-{}", i)); }
        (input, output)
    }
    #[allow(dead_code)]
    pub fn to_net(&self, i: usize, net: &mut Net) {
        for (i, pixel) in self.input()[i].iter().enumerate() {
            net.signal(0, format!("in-{}", i), *pixel + util::rand_float(-0.001, 0.001));
        }
    }
    pub fn output_at(&self, i: usize) -> Vec<f32> {
        self.output()[i].clone()
    }
    #[allow(dead_code)]
    pub fn print_input(&self, i: usize) {
        println!("{:}", self.input()[i]);
    }
    pub fn len(&self) -> usize {
        self.input().len()
    }
    pub fn input(&self) -> Arc<Vec<Matrix<f32>>> {
        match &self.mode {
            DsMode::Train => self.train_input.clone(),
            DsMode::Val => self.val_input.clone(),
            DsMode::Test => self.test_input.clone(),
        }
    }
    pub fn output(&self) -> Arc<Vec<Vec<f32>>> {
        match &self.mode {
            DsMode::Train => self.train_output.clone(),
            DsMode::Val => self.val_output.clone(),
            DsMode::Test => self.test_output.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> Arc<Vec<u8>> {
        match &self.mode {
            DsMode::Train => self.train_labels.clone(),
            DsMode::Val => self.val_labels.clone(),
            DsMode::Test => self.test_labels.clone(),
        }
    }
    pub fn new() -> Dataset {
        eprintln!("[ds] Loading dataset...");
        let mnist = MnistBuilder::new()
            .training_set_length(30_000)
            .validation_set_length(30_000)
            .test_set_length(10_000)
            .label_format_one_hot()
            .finalize();



        eprintln!("[ds] Converting...");
        let (input, output) = convert(&mnist.trn_img, &mnist.trn_lbl);
        let (vinput, voutput) = convert(&mnist.val_img, &mnist.val_lbl);
        let (tinput, toutput) = convert(&mnist.tst_img, &mnist.tst_lbl);
        eprintln!("[ds] Finished...");

        Dataset {
            mode: DsMode::Train,
            train_input: Arc::new(input),
            train_output: Arc::new(output),
            train_labels: Arc::new(mnist.trn_lbl),
            val_input: Arc::new(vinput),
            val_output: Arc::new(voutput),
            val_labels: Arc::new(mnist.val_lbl),
            test_input: Arc::new(tinput),
            test_output: Arc::new(toutput),
            test_labels: Arc::new(mnist.tst_lbl),
        }
    }

}

fn convert(img_set: &Vec<u8>, lbl_set: &Vec<u8>) -> (Vec<Matrix<f32>>, Vec<Vec<f32>>) {
    let img_set = Matrix::new(lbl_set.len()/10 * 28, 28, img_set.clone());
    let img_set: Matrix<f32> = img_set.try_into().unwrap() / 255.0;
    let mut input = vec![];
    let mut output = vec![];
    for _ in 0..lbl_set.len()/10 {
        let n = input.len();
        let mut label = Vec::with_capacity(10);
        for l in &lbl_set[n*10..(n+1)*10] { label.push(*l as f32) }
        let row_indexes = (n*28..(n+1)*28).collect::<Vec<_>>();
        let img = img_set.select_rows(&row_indexes);
        input.push(img);
        output.push(label);
    }
    (input, output)
}

#[allow(dead_code)]
pub fn display (m: &Matrix<f32>) {
    for y in 0..28 {
        for x in 0..28 {
            let v = m[[y, x]];
            if v < 0.4 {
                print!(" ");
            } else if v < 0.8 {
                print!("·");
            } else {
                print!("#");
            }
        }
        print!("\n");
    }
}
