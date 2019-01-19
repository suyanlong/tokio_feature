extern crate chrono;
extern crate futures;
extern crate tokio_core;

use std::error::Error;
use std::thread::{sleep, spawn};

use chrono::prelude::*;
use chrono::*;
use futures::prelude::*;
use futures::*;
use tokio_core::reactor::Core;

pub struct WaitInAnotherThread {
    end_time: DateTime<Utc>,
    running: bool,
    count: u64,
}

impl WaitInAnotherThread {
    pub fn new(how_long: Duration) -> WaitInAnotherThread {
        WaitInAnotherThread {
            end_time: Utc::now() + how_long,
            running: false,
            count: 0,
        }
    }

    fn run(&mut self, task: task::Task) {
        let lend = self.end_time;

        spawn(move || {
            while Utc::now() < lend {
                let delta_sec = lend.timestamp() - Utc::now().timestamp();
                if delta_sec > 0 {
                    sleep(::std::time::Duration::from_secs(delta_sec as u64));
                }
                // 通知，完成了这个任务！！！
                task.notify();
            }
            println!("the time has come == {:?}!", lend);
        });
    }
}

impl Future for WaitInAnotherThread {
    type Item = ();
    type Error = Box<Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.count += 1;
        println!("count = {}", self.count);
        if Utc::now() < self.end_time {
            println!("not ready yet! parking the task.");

            if !self.running {
                println!("side thread not running! starting now!");
                // 终于找到了，这个方法了。这个就是通知的方法呀。
                // 当前的只有自己，因为reactor是单线程的，正执行到这个方法内部，肯定是自己的token id了！！！！
                self.run(task::current());
                self.running = true;
            }

            Ok(Async::NotReady)
        } else {
            println!("ready! the task will complete.");
            Ok(Async::Ready(()))
        }
    }
}

// https://zhuanlan.zhihu.com/p/51784496 重要！！！！
fn main() {
    let mut reactor = Core::new().unwrap();

    let wiat = WaitInAnotherThread::new(Duration::seconds(3));
    println!("wait future started");
    let ret = reactor.run(wiat).unwrap();
    println!("wait future completed. ret == {:?}", ret);
}
