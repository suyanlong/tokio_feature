#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate futures_cpupool;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;
extern crate parking_lot;
extern crate threadpool;

use parking_lot::Mutex;
use std::io;
use futures::{BoxFuture, Future};
use futures_cpupool::CpuPool;
use rand::Rng;
use tokio_minihttp::{Request, Response};
use tokio_proto::TcpServer;
use tokio_service::Service;
use std::sync::Arc;
use std::collections::HashMap;
use futures::sync::oneshot;
use std::collections::VecDeque;

#[derive(Serialize)]
struct Message {
    id: i32,
    body: String,
}

struct Server {
    hash_map: Arc<Mutex<VecDeque<(String, oneshot::Sender<Response>)>>>,
    thread_pool: Arc<Mutex<threadpool::ThreadPool>>,
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = BoxFuture<Response, io::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let (tx, rx) = oneshot::channel();
        println!("-----call ------");
        let random_id = rand::thread_rng().gen_range(1, 50000);
        self.thread_pool.lock().execute(move || {
            println!(
                "-thread id = {:?}-------k ={:?}",
                thread::current().id(),
                random_id
            );
            let mut response = Response::new();
            response.header("Content-Type", "application/json");
            response.body(&random_id.to_string());
            tx.send(response);
        });
        //        self.hash_map.lock().push_back((random_id.to_string(), tx));
        rx.map_err(|_| io::Error::from_raw_os_error(1)).boxed()
    }
}

use std::thread;
use std::time::Duration;
use std::ops::Deref;
fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let arc_hash_map = Arc::new(Mutex::new(VecDeque::new()));
    let threadpool = Arc::new(Mutex::new(threadpool::ThreadPool::new(10)));
    let hash_map = arc_hash_map.clone();
    let _ = thread::spawn(move || while false {
        thread::sleep(Duration::from_millis(3));
        let mut map = arc_hash_map.lock();
        let map: &mut VecDeque<(String, oneshot::Sender<Response>)> = map.deref_mut();
        map.pop_front().map(|(k, v)| {
            println!("----k ={:?}", k);
            let mut response = Response::new();
            response.header("Content-Type", "application/json");
            response.body(&k);
            v.send(response);
        });
    });

    let server = Server {
        hash_map: hash_map,
        thread_pool: threadpool,
    };
    TcpServer::new(tokio_minihttp::Http, addr).serve(server);
}

use std::ops::DerefMut;
use tokio_service::NewService;
impl NewService for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Instance = Server;
    fn new_service(&self) -> io::Result<Server> {
        Ok(Server {
            hash_map: self.hash_map.clone(),
            thread_pool: self.thread_pool.clone(),
        })
    }
}
