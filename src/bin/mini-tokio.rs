// #[allow(dead_code, unused_variables)]

use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::task;

#[derive(Debug)]
struct Delay {
    when: Instant,
    count: RefCell<u32>,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut current = self.count.borrow_mut();
        *current += 1;
        println!("{current}: poll");
        if self.when <= Instant::now() {
            println!("===================Done===================");
            Poll::Ready("Done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

struct MiniTokio {
    tasks: VecDeque<Task>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }

    fn spawn(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.tasks.push_back(Box::pin(future))
    }

    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        println!("run");

        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task)
            }
        }
    }
}

// #[tokio::main]
fn main() {
    println!("mini-tokio");
    let when = Instant::now() + Duration::from_millis(500);
    let count = RefCell::new(0);
    let future = Delay { when, count };

    let main_future = async move {
        let out = future.await;
        assert_eq!(out, "Done");
    };

    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(main_future);

    mini_tokio.run();
}
