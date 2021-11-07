use hashbrown::HashMap;
use dataset::{Dataset};
use dataset;
use net::Net;
use util;

pub fn eval_net(nets: &mut HashMap<String, Net>, ds: &Dataset, set: &Vec<usize>, _max_errors: usize, _debug: bool) -> f32 {
    let _perf = util::time();
    let mut success = 0.;
    let mut main_net = nets.get_mut("main_net").unwrap();
    let mut scores = [0_f32; 10];

    for index in set.clone() {

        let input = &ds.input()[index];
        let expect = ds.output_at(index);

        main_net.reset();
        for x in 0..28 {
            for y in 0..28 {
                main_net.signal(0, format!("!in {}-{}", x, y), input[[x, y]]);
            }
        }

        // main_net.cycle(1, false);
        let active = main_net.cycle(10, true);

        let mut out = Vec::with_capacity(10);

        let mut max = -999999.0;
        for i in 0..expect.len() as u8 {
            let name = format!("!out {}", i);
            let res = active.contains_key(&name);
            if res {
                let n = &active[&name];
                out.push(n.activated());
                if n.activated() > max {
                    max = n.activated();
                }
            } else {
                out.push(0.0);
            };
        }

        let guess = util::max_i(&out);
        let expected = util::max_i(&expect);
        let ok = !active.is_empty() && guess == expected;
        success+= if ok { 1.0 } else { 0.0 };
        assert!(out[guess] >= 0.0);
        assert!(out[guess] <= 1.0);

        if ok {
            scores[guess]+= 100.0 * out[guess];
        }
        for i in 0..expect.len() {
            let err = (expect[i] - out[i]).abs();
            assert!(err >= 0.0);
            assert!(err <= 1.0);
            scores[i]+= 1.0-err;
        }


        if _debug {
            println!("-------");
            println!("guess={} expected={}", guess, expected);
            println!("out={:?}", &out);
            println!("time={}", main_net.time);
            dataset::display(input);
            println!("-------");
        }
    }

    main_net.reset_score();
    let input_map = main_net.input_map();
    main_net.success = success / set.len() as f32;
    eprintln!("{} {}", success, set.len());
    for i in 0..10 {
        main_net.feedback(format!("!out {}", i), scores[i], &input_map);
    }
    // score/= set.len() as f32;
    // let scores: Vec<f32> = main_net.nodes.values().map(|x| x.score).collect();
    // let mma = util::mma(&scores);
    main_net.score = main_net.success;
    return main_net.score;
}
