// 
// use rpc::NetDescription;
// use rpc::Opts;
//
// use hashbrown::HashMap;
//
// use net::Net;
// use util;
// use dataset::Dataset;
// use gapool::GaPool;
//
//
// // This is the service definition. It looks a lot like a trait definition.
// // It defines one RPC, hello, which takes one arg, name, and returns a String.
// tarpc::service! {
//     rpc get(genes: Vec<NetDescription>) -> (HashMap<String, Vec<Net>>, Opts);
//     rpc record(nets: HashMap<String, Net>) -> ();
//     rpc record_and_get(nets: HashMap<String, Net>, genes: Vec<NetDescription>) -> (HashMap<String, Vec<Net>>, Opts);
// }
