//#[macro_use]
extern crate grpc;
extern crate thola;
extern crate threadpool;

use threadpool::ThreadPool;

use std::sync::mpsc::channel;

use std::net::SocketAddr;

use grpc::RequestOptions;

use thola::macaroon_data::MacaroonData;
use thola::rpc;
use thola::rpc_grpc::Lightning;
use thola::rpc_grpc::LightningClient;
use thola::tls_certificate::TLSCertificate;

fn load_certificate() -> TLSCertificate {
    if let Some(cert_filename) = std::env::args().into_iter().skip(2).next() {
        if let Ok(r) = TLSCertificate::from_path(cert_filename) {
            return r;
        } else {
            panic!("Error loading Certificate");
        }
    } else {
        panic!("Error loading Certificate");
    }
}

fn generate_client() -> LightningClient {
    let socket_addr_string = match std::env::args().into_iter().skip(4).next() {
        Some(s) => s,
        None => String::from("127.0.0.1:10009"),
    };

    if let Ok(socket_addr) = socket_addr_string.parse::<SocketAddr>() {
        let host = socket_addr.ip().to_string();
        let conf = Default::default();

        let certificate = load_certificate();

        if let Ok(tls) = certificate.into_tls(host.as_str()) {
            if let Ok(c) = grpc::Client::new_expl(&socket_addr, host.as_str(), tls, conf) {
                return LightningClient::with_client(c);
            } else {
                panic!("Could not create LightningClient Connection");
            };
        } else {
            panic!("Could not create TLS");
        };
    } else {
        panic!("Could not parse lnd host. Please use IP:PORT");
    }
}

fn metadata() -> RequestOptions {
    if let Some(macaroon_file_path) = std::env::args().into_iter().skip(3).next() {
        if let Ok(macaroon_data) = MacaroonData::from_file_path(macaroon_file_path) {
            return RequestOptions {
                metadata: macaroon_data.metadata(),
            };
        } else {
            panic!("Error loading macaroon");
        };
    } else {
        panic!("Error loading macaroon");
    };
}

fn my_query_routes(satoshis: i64, remote_node: &rpc::LightningNode) -> bool {
    let client = generate_client();

    let mut query_routes_req = rpc::QueryRoutesRequest::new();
    query_routes_req.pub_key = remote_node.pub_key.clone();
    query_routes_req.amt = satoshis;
    query_routes_req.num_routes = 1;

    //let node = remote_node.clone();

    let query_routes_res = client.query_routes(metadata(), query_routes_req);
    let r = match query_routes_res.wait() {
        Ok(_) => 1,
        Err(_) => 0,
    };

    return r > 0;
}

fn parse_sats_amount() -> i64 {
    if let Some(satoshis_env) = std::env::args().into_iter().skip(1).next() {
        if let Ok(satoshis) = satoshis_env.parse::<i64>() {
            return satoshis;
        } else {
            panic!("Error parsing the satoshi amount");
        }
    } else {
        panic!("Error parsing the satoshi amount");
    }
}

fn query_graph() -> rpc::ChannelGraph {
    let client = generate_client();

    let graph_req = rpc::ChannelGraphRequest::new();
    let graph_resp = client.describe_graph(metadata(), graph_req);

    if let Ok(graph) = graph_resp.wait() {
        return graph.1;
    } else {
        panic!("Error downloading graph");
    }
}

fn main() {
    if std::env::args().len() < 5 {
        println!("Usage: lnd_thola -- %satoshis% %path_to_cert% %path_to_macaroon% %socket");
        println!("eg: ./thola 100000 ./tls.cert ./readonly.macaroon 192.168.1.128:10009");
        return;
    }

    println!("Downloading Graph");

    let graph = query_graph();

    let count = graph.nodes.len() as usize;
    let ccc = graph.nodes.clone();

    let n_workers = 100;
    let n_jobs = graph.nodes.len();
    let pool = ThreadPool::new(n_workers);

    let (tx, rx) = channel();

    let satoshis = parse_sats_amount();

    for node in ccc.into_iter() {
        let tx = tx.clone();

        pool.execute(move || {
            let r = my_query_routes(satoshis, &node);

            let _ = tx.send(r);
        });
    }

    let result: Vec<bool> = rx.iter().take(n_jobs).filter(|n| n.eq(&true)).collect();

    let reachable: Vec<bool> = result.into_iter().filter(|n| n.eq(&true)).collect();

    let r_percent = reachable.len() * 100 / count;

    println!("#############################");
    println!("{0: <20} | {1: <20}", "Sats:", satoshis);
    println!("{0: <20} | {1: <20}", "Nodes:", count);
    println!("Reachable Nodes      | {:?}% ", r_percent);
    println!("#############################");
}
