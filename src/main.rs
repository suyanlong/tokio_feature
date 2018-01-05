extern crate futures;
extern crate futures_cpupool;
extern crate httparse;
extern crate hyper;
extern crate parking_lot;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate threadpool;
extern crate tokio_service;

use parking_lot::Mutex;
use std::io;
use futures::{BoxFuture, Future};
use futures_cpupool::CpuPool;
use rand::Rng;
use std::sync::Arc;
use std::collections::HashMap;
use futures::sync::oneshot;
use std::collections::VecDeque;

use futures::future::FutureResult;
use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, NewService, Request, Response, Service};

use std::thread;
use std::time::Duration;
use std::ops::Deref;
use std::ops::DerefMut;

impl NewService for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Instance = Server;
    fn new_service(&self) -> io::Result<Server> {
        Ok(Server {
            hash_map: self.hash_map.clone(),
        })
    }
}

#[derive(Serialize)]
struct Message {
    id: i32,
    body: String,
}

struct Server {
    hash_map: Arc<Mutex<VecDeque<(String, oneshot::Sender<String>)>>>,
}

use futures::Stream;

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = BoxFuture<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        println!("method: {:?}, path: {:?}, ", req.method(), req.path());
        let (tx, rx) = oneshot::channel();
        //println!("-----post --{:?}----", req.slice((1, 100)));
        let random_id = rand::thread_rng().gen_range(1, 50000);
        println!("-----call -----{:?}-", random_id);
        {
            self.hash_map.lock().push_back((random_id.to_string(), tx));
        }

        //这个时候已经可以了啊，满足我们的需求了。
        rx.map(|item| {
            println!("=={:?}==", item);
            let mut res = Response::new();
            res.set_body(item);
            res
        }).map_err(|_| hyper::Error::Timeout)
            .boxed()

        //        rx.map_err(|_| hyper::Error::from(io::Error::from_raw_os_error(1)))
        //            .boxed()
        //                let mut body = req.body();
        //                loop {
        //                    let data = body.poll();
        //                    body.for_each();
        //                    println!("-----data---{:?}", data);
        //                }
        //                req.body().for_each(move |chunk| {
        //                    println!("-msg--{:?}", chunk);
        //                        let (tx, rx) = oneshot::channel();
        //                        let random_id = rand::thread_rng().gen_range(1, 50000);
        //                        self.thread_pool.lock().execute(move || {
        //                            println!(
        //                                "-thread id = {:?}-------k ={:?}",
        //                                thread::current().id(),
        //                                random_id
        //                            );
        //                            let mut response = Response::new();
        //                            response.set_body(random_id.to_string());
        //                            tx.send(response);
        //                        });
        //                    Ok(())
        //                });
        //                    .map(move |rx| {
        //                        let mut response = Response::new();
        //                        //                response.set_body(random_id.to_string());
        //                        response
        //                    })
        //                    .boxed()
        //                let mut response = Response::new();
        //                println!("--------");
        //                        response.set_body(random_id.to_string());
        //                        response
        //                futures::finished(response).boxed()
        //                rx.map_err(|_| hyper::Error::Version).boxed()
        //                rx.map_err()
        //                    .map(move |_| {
        //                        let res = rx.
        //
        //
        //                    })
        //                    .boxed()
    }
}

fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let arc_hash_map = Arc::new(Mutex::new(VecDeque::new()));
    let hash_map = arc_hash_map.clone();

    let _ = thread::spawn(move || {
        while true {
            thread::sleep(Duration::from_secs(1));
            let mut map = arc_hash_map.lock();
            let map: &mut VecDeque<(String, oneshot::Sender<String>)> = map.deref_mut();
            map.pop_front().map(|(k, v)| {
                println!("----k = {:?}", k);
                //                let mut response = Response::new();
                //                response
                //                    .headers_mut()
                //                    .set_raw("Content-Type", "application/json");
                //                response.set_body(k);
                v.send(k);
            });
        }
    });

    let server = Server { hash_map: hash_map };
    let server = Http::new().bind(&addr, server).unwrap();
    println!(
        "Listening on http://{} with 1 thread.",
        server.local_addr().unwrap()
    );
    server.run().unwrap();
}
