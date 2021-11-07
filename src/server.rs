use std::sync::{Arc,RwLock};
use hashbrown::HashMap;
// use std::thread;
use getopts::Matches;
use storage;
use rpc;
use net;
use util;
use dataset::Dataset;
use rouille;

pub fn server(args: Matches, ds: Dataset) {

    use yaml_rust::yaml::YamlLoader;
    let opts = &YamlLoader::load_from_str(&util::read_file(&util::arg(&args, "config", "config.yaml".to_string()))).expect("Yaml is not well formatted")[0];


    // let net = load_net(&format!("{}.best", session_path));
    // if ga.leaderboard.is_empty() {
    //     ga.leaderboard.insert(net);
    //     ga.genes.clear();
    // }

    let session_path = util::arg(&args, "write", "/tmp/checkpoint".to_string());
    let threads: usize = util::arg(&args, "threads", 1);

    let ga_pools_opt = storage::load_ga_pool(&session_path, opts);
    let loaded = !ga_pools_opt.is_none();

    let mut ga_pools = match ga_pools_opt {
        Some(gaps) => {
            gaps
        },
        None => HashMap::new()
    };

    if loaded {
        eprintln!("Increasing vset");
        for gap in ga_pools.values_mut() {
            gap.incr_vset(opts, &ds);
        }
    }

    for gap in ga_pools.values_mut() {
        gap.debug = true;
    }

    let ga_pools = Arc::new(RwLock::new(ga_pools));

    for _ in 0..threads {
        let args = args.clone();
        let opts = opts.clone();
        let ga_pools = ga_pools.clone();
        let ds = ds.clone();

        thread::spawn(move || {
            rpc::server(&args, &opts, ga_pools, ds);
        });
    }

    use std::thread;
    let gaps_server = ga_pools.clone();
    thread::spawn(move || {
        // let gaps = ga_pools.clone();
        rouille::start_server("0.0.0.0:8090", move |request| {
            router!(request,
                (GET) (/stats) => {
                    let gaps = gaps_server.read().unwrap().clone();
                    // If the request's URL is `/`, we jump here.
                    // This block builds a `Response` object that redirects to the `/hello/world`.
                    // let json = serde_json::to_string(&gaps).expect("[storage] Couldn't serialize GA");
                    rouille::Response::json(&gaps).with_unique_header("Access-Control-Allow-Origin", "*")
                },
                _ => rouille::Response::empty_204().with_unique_header("Access-Control-Allow-Origin", "*")
            )
        });
    });

    loop {
        util::sleep(5000);
        {
            let mut gapsw = ga_pools.write().unwrap();
            for gap in gapsw.values_mut() {
                gap.fix_overfitting(opts, &ds);
            }
        }

        // Check saves
        eprintln!("[s] Saving...");

        {
            let gapsr = ga_pools.read().unwrap().clone();
            // for (ganame, ga) in gapsr.iter() {
            //     for (name, gene) in ga.src().genes.iter() {
            //         if gene.received_alleles > 1 {
            //             eprintln!("{}: gene {: >9} - {}", ganame, name, gene.received_alleles);
            //         }
            //     }
            // }
            storage::store_ga_pools(&session_path, &gapsr);
            let mut leaders: HashMap<String, net::Net> = HashMap::new();
            for (gap_name, gap) in gapsr.iter() {
                let ga = gap.src();
                if ga.has_leader() {
                    leaders.insert(gap_name.to_string(), ga.leaderboard.iter().next().unwrap().clone());
                }
            }
            storage::store(&format!("{}.best", &session_path), &leaders);

            // Getting top 10 genes
            for (gap_name, gap) in gapsr.iter() {
                let ga = gap.src();
                let top_n = 10;
                let mut top: Vec<(String, &crate::gene::Allele)> = Vec::with_capacity(top_n);

                for (gene_name, gene) in ga.genes.iter() {
                    let mut best: Option<&crate::gene::Allele> = None;
                    for allele in gene.alleles.values() {
                        if best.is_none() || best.unwrap().avg_score < allele.avg_score {
                            best = Some(allele);
                        }
                    }
                    top.push((gene_name.to_string(), best.unwrap()));
                }


                // eprintln!("Top 10 {:?}", top.iter().map(|x| x.1.avg_score).cloned());
                top.sort_by(|b, a| a.1.avg_score.partial_cmp(&b.1.avg_score).expect("Sorting failed 3"));
                eprintln!("Top 10 {}:", gap_name);
                for (i, el) in top.iter().enumerate() {
                    if i > 10 && i < top.len()-5 { continue }
                    let a = el.1;
                    eprintln!("s={:.4}\tlinks={}\tgr={}\tar={}\tg={}", a.avg_score, a.node.links.len(), ga.genes[&el.0].received_alleles, a.count, el.0);
                }
            }

        }
    }
}
