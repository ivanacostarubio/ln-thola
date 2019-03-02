//#[macro_use]
extern crate pbr;

extern crate crossbeam;
extern crate lazy_static;


extern crate grpc;
extern crate thola;


use std::thread;

use std::sync::{Arc, Mutex};


use thola::rpc_grpc::LightningClient;
use thola::rpc_grpc::Lightning;
use thola::tls_certificate::TLSCertificate;
use thola::macaroon_data::MacaroonData;

use thola::rpc;
use grpc::RequestOptions;


fn generate_client() -> LightningClient {

    use std::net::SocketAddr;

    let certificate = {
        let cert_filename = std::env::args()
            .into_iter().skip(2).next()
            .unwrap();
        TLSCertificate::from_path(cert_filename)
            .unwrap()
    };


    let default = "127.0.0.1:10009";
    let socket_addr_string = std::env::args()
        .into_iter().skip(4).next()
        .unwrap_or(default.to_owned());
    let socket_addr: SocketAddr = socket_addr_string.parse().unwrap();
    let host = socket_addr.ip().to_string();
    let conf = Default::default();

    let tls = certificate.into_tls(host.as_str())
        .unwrap();
    let c = grpc::Client::new_expl(&socket_addr, host.as_str(), tls, conf)
        .unwrap();
    return LightningClient::with_client(c)
}



fn metadata() -> RequestOptions {
    let macaroon_data = {
        let macaroon_file_path = std::env::args()
            .into_iter().skip(3).next()
            .unwrap();
        MacaroonData::from_file_path(macaroon_file_path)
            .unwrap()
    };

    
    return RequestOptions { metadata: macaroon_data.metadata(), };
}

fn my_query_routes(satoshis: i64, remote_node: &rpc::LightningNode) ->bool { 

    print!(".");

    let client = generate_client();

    let mut query_routes_req = rpc::QueryRoutesRequest::new();
    query_routes_req.pub_key = remote_node.pub_key.clone();
    query_routes_req.amt = satoshis;
    query_routes_req.num_routes = 1;

    //let node = remote_node.clone();

    let query_routes_res = client.query_routes( metadata(), query_routes_req);
    let r = match query_routes_res.wait(){
        Ok(_) => 1,
        Err(_) => 0
    };

    return r > 0;

}

fn main() {

    if std::env::args().len() < 5 {

        println!("Usage: lnd_thola -- %satoshis% %path_to_cert% %path_to_macaroon% %socket");
        println!("eg: ./thola 100000 ./tls.cert ./readonly.macaroon 192.168.1.128:10009");
        return
    }


    println!("Downloading Graph");

    let satoshis_env = std::env::args().into_iter().skip(1).next().unwrap();
    let satoshis = satoshis_env.parse::<i64>().unwrap();
    let client = generate_client();
    let graph_req = rpc::ChannelGraphRequest::new();
    let graph_resp = client.describe_graph( metadata(), graph_req); 
    let ww = graph_resp.wait().unwrap();

    /*
     * Progress Bar
     */

    let count = ww.1.nodes.len() as usize;
    let ccc = ww.1.nodes.clone();



    let mutex = Arc::new(Mutex::new(vec![]));
    let mut children = vec![];

    for node in ccc.into_iter() {

        let data = Arc::clone(&mutex);

        children.push(thread::spawn(move || {
            println!("new thread ");
            let result = my_query_routes(satoshis, &node);
            let mut data = data.lock().unwrap();
            data.push(result);
        }));

    }

    for child in children {
        let _ = child.join();

    }

    println!("{:?}", mutex);



//    let result: Vec<bool> = ccc.into_iter().map(|node| my_query_routes(satoshis, &node ) ).collect() ;


//    let reachable: Vec<bool> = result.into_iter().filter(|n| n.eq(&true) ).collect();




    //
    //
    //
    //
    //
    //










    //
    //
    //
    //



/*
    let r_percent = reachable.len() * 100 / count;



    println!("#############################");
    println!("{0: <20} | {1: <20}", "Sats:", satoshis);

    println!("{0: <20} | {1: <20}", "Nodes:", count);
    println!("{0: <20} | {1: <20}%", "Reachable:", r_percent);
*/

//    println!("Total Nodes: {:?}", count);
//    println!("{:?}% of reachable nodes", r_percent);
//    println!("#############################");

}



