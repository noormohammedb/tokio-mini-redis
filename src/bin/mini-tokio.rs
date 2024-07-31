// #[allow(dead_code, unused_variables)]

use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use std::{cell::RefCell, future::Future, pin::Pin, thread};

use futures::future::poll_fn;
use futures::task::{self, ArcWake};

const TIMEOUT: u64 = 5;

#[derive(Debug)]
struct Delay {
    when: Instant,
    count: RefCell<u32>,
    waker: Option<Arc<Mutex<Waker>>>,
}
impl Delay {
    fn new(delay: Duration) -> Delay {
        Delay {
            when: Instant::now() + delay,
            count: RefCell::new(0),
            waker: None,
        }
    }
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // let mut current = self.count.borrow_mut();
        // *current += 1;
        // println!("{current}: poll");
        if self.when <= Instant::now() {
            println!("===================Done===================");
            return Poll::Ready("Done");
        }

        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
            Poll::Pending
        } else {
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }
                println!("waked");
                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
            println!("pending");
            Poll::Pending
        }
    }
}

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}
impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> TaskFuture {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(self: &mut Self, cx: &mut Context) {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

struct Task {
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone()).unwrap();
    }
}
impl Task {
    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();

        task_future.poll(&mut cx);
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static, sender: &mpsc::Sender<Arc<Task>>) {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio { scheduled, sender }
    }
    fn run(&self) {
        // while let Ok(task) = self.scheduled.recv() {
        //     task.poll();
        // }
        loop {
            match self.scheduled.recv_timeout(Duration::from_secs(TIMEOUT)) {
                Ok(task) => task.poll(),
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }
    }
    fn spawn(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(future, &self.sender);
    }
}

// #[tokio::main]
fn main() {
    println!("mini-tokio");
    let future = Delay::new(Duration::from_millis(2000));
    let future_01 = Delay::new(Duration::from_millis(4000));

    let main_future = async move {
        let out = future.await;
        assert_eq!(out, "Done");
    };

    let fut2 = async move {
        let _ = future_01.await;
    };

    // let host_listen = async {
    //     dbg!("Foo");
    //     let foo = tokio::net::TcpListener::bind("0.0.0.0:9090").await;
    //     dbg!("bar");
    // };
    // mini_tokio.spawn(host_listen);

    let mut mini_tokio = MiniTokio::new();
    mini_tokio.spawn(main_future);
    mini_tokio.spawn(fut2);

    mini_tokio.run();
}
/*

#[tokio::main]
async fn main() {
    let mut delay = Some(Delay::new(Duration::from_millis(1000)));

    poll_fn(move |cx| {
        let mut delay = delay.take().unwrap();
        let res = Pin::new(&mut delay).poll(cx);
        assert!(res.is_pending());
        let foo = tokio::spawn(async move {
            delay.await;
        });

        // Poll::Ready(())
        Poll::<()>::Pending
    })
    .await;
}

 */
