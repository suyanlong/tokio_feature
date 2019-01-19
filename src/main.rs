extern crate chrono;
extern crate futures;
extern crate tokio_core;

use std::error::Error;

use chrono::prelude::*;
use chrono::*;
use futures::done;
use futures::future::join_all;
use futures::future::{err, ok};
use futures::prelude::*;
use futures::prelude::*;
use futures::*;
use tokio_core::reactor::Core;

#[derive(Debug)]
struct WaitForIt {
    message: String,
    until: DateTime<Utc>,
    polls: u64,
}

impl WaitForIt {
    pub fn new(message: String, delay: Duration) -> WaitForIt {
        WaitForIt {
            polls: 0,
            message: message,
            until: Utc::now() + delay,
        }
    }
}

impl Future for WaitForIt {
    type Item = String;
    type Error = Box<Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let now = Utc::now();
        if self.until < now {
            Ok(Async::Ready(format!(
                "{} after {} polls!",
                self.message, self.polls
            )))
        } else {
            self.polls += 1;

            println!("not ready yet --> {:?}", self);
            // 只有调用这个方法吗，在poll内部，难道没有其他的办法吗？一直调用poll方法啊！！！！。
            // 一秒钟被调用35474次poll方法，就是一直循环的调用，如果注释这句，却只能背调用一次，难道没有通知机制？？
            // 频繁的调用，多占用CPU！浪费资源。poll，不亏叫这个名字，就只一直循环的poll。
            futures::task::current().notify();
            Ok(Async::NotReady)
        }
    }
}

fn main() {
    let mut reactor = Core::new().unwrap();

    let wfi_1 = WaitForIt::new("I'm done:".to_owned(), Duration::seconds(1));
    println!("wfi_1 == {:?}", wfi_1);
    let wfi_2 = WaitForIt::new("I'm done too:".to_owned(), Duration::seconds(2));
    println!("wfi_2 == {:?}", wfi_2);

    let v = vec![wfi_1, wfi_2];

    //let sel = join_all(v);
    let sel = select_ok(v);

    let ret = reactor.run(sel).unwrap();
    println!("ret == {:?}", ret);
}
//....
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35468 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35469 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35470 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35471 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35472 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35473 }
//not ready yet --> WaitForIt { message: "I\'m done:", until: 2019-01-19T14:12:10.691746Z, polls: 35474 }
//ret == ["I\'m done: after 35474 polls!"]
