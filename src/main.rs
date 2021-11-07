
extern crate clap;
extern crate hashbrown;

extern crate rand;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate time;

extern crate getopts;
use getopts::Options;

extern crate mnist;

extern crate rulinalg;

extern crate yaml_rust;

extern crate noise;

extern crate bincode;


#[macro_use]
extern crate rouille;


use std::env;

mod node;
mod link;
mod ga;
mod gene;
mod net;
mod gapool;
mod util;
mod com;
mod server;
mod client;
mod rpc;
mod fitness;
mod storage;
mod dataset;

fn main() {
    // use noise::{NoiseFn, Perlin};
    //
    // loop {
    //     let perlin = Perlin::new();
    //
    //     let now = util::now();
    //     println!("-------------");
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 37.7, 2.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 36547.7, 2000.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 37.7, 200000.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 1237.7, 1000002.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 432137.7, 57777.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 3654457.7, 2.8]) as f32);
    //     println!("{}", 0.5+0.5*perlin.get([now as f64/500000.0, 33457.7, 2.8]) as f32);
    //     util::sleep(200);
    // }

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("w", "write", "checkpoint file path", "<path>");
    opts.optopt("p", "pool-size", "max pool size", "<size>");
    opts.optopt("r", "max-record", "max record memory", "<size>");
    opts.optopt("m", "mrate", "mutation rate", "<0-1>");
    opts.optopt("n", "samples", "samples to test before sending back the best", "<num>");
    opts.optopt("s", "serve", "serve on port", "<port>");
    opts.optopt("c", "connect", "connect to ip", "<ip:port>");
    opts.optopt("l", "lead-size", "leaderboard size", "<size>");
    opts.optopt("", "forget-lb", "forget leadership in checkfile", "<checkfile>");
    opts.optopt("", "test", "loads a network from a file and tests it", "<net-file>");
    opts.optopt("", "config", "config file path", "config.yaml");
    opts.optopt("t", "threads", "used with -c to specify how many threads to run", "<num>");
    opts.optflag("h", "help", "print this help menu");

    let args = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { eprintln!("{}", f.to_string()); return; }
    };

    if args.opt_present("h") {
        let brief = format!("Usage: {} FILE [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }
    if args.opt_present("forget-lb") {
        let file = args.opt_str("forget-lb").unwrap();
        let mut ga = storage::load_ga(&file);
        ga.leaderboard.clear();
        storage::store_ga(&file, &ga);
        println!("File cleared");
    }



    let ds = dataset::Dataset::new();

    if args.opt_present("serve") {
        server::server(args.clone(), ds.clone());
    }

    if args.opt_present("connect") {
        util::sleep(100);
        client::client(args.clone(), ds.clone());
    }

    if args.opt_present("test") {
        client::test_best(&args.opt_str("test").unwrap(), ds.clone());
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use dataset;
        let mut ds = dataset::Dataset::new();

        println!("Train");
        ds.mode = dataset::DsMode::Train;
        println!("{:}", ds.input()[10]);
        println!("{:?}", ds.output()[10]);
        assert_eq!(ds.len(), 20_000);

        println!("Val");
        ds.mode = dataset::DsMode::Val;
        println!("{:}", ds.input()[10]);
        println!("{:?}", ds.output()[10]);
        assert_eq!(ds.len(), 40_000);

        println!("Test");
        ds.mode = dataset::DsMode::Test;
        println!("{:}", ds.input()[10]);
        println!("{:?}", ds.output()[10]);
        assert_eq!(ds.len(), 10_000);
    }

    #[test]
    fn scoring_test() {
        use ga;
        use net;
        use link;
        use serde_json;

        let mut ga = ga::GA::new();
        let input = vec!["x".to_string()];
        let output = vec!["y".to_string()];
        let mut net = net::Net::from_ga(&mut ga, &input, &output);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());


        {
            let mut l = link::Link::new("y".to_string());
            l.weight = 2.0;
            l.delay = 2;
            let node = &mut net.node(&"x".to_string());
            node.links.push(l);
        }
        {
            let mut l = link::Link::new("y".to_string());
            l.weight = 1.0;
            l.delay = 1;
            let node = &mut net.node(&"y".to_string());
            node.threshold = 0.1;
            node.links.push(l);
        }

        //         r0t0       r0t1
        //        +---+  *  +---+
        //     -> | x | --> | y |
        //        +---+  2  +---+
        eprintln!("c0");
        net.reset();
        net.signal(0, "x".to_string(), 0.2);

        assert_eq!(0.0, net.val(&"y".to_string()));
        assert_eq!(0.0, net.val(&"x".to_string()));
        eprintln!("c1");
        net.cycle(1, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());

        assert_eq!(0.0, net.val(&"x".to_string()));
        assert_eq!(0.0, net.val(&"y".to_string()));
        assert_eq!(1, net.actions.len());

        eprintln!("c2");
        let activations = net.cycle(2, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());
        assert_eq!(0.0, net.val(&"x".to_string()));
        assert_eq!(0.4, activations["y"].value);

        eprintln!("c3");
        let activations = net.cycle(1, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());
        let activations = net.cycle(1, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());
        let activations = net.cycle(1, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());
        let activations = net.cycle(1, false);
        eprintln!("t{}: {}", net.time, serde_json::to_string_pretty(&net).unwrap());
        assert_eq!(0.0, net.val(&"x".to_string()));
        assert_eq!(0.4, net.val(&"y".to_string()));

        // debug!("0: {}", net);


    }
}
