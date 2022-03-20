
pub mod threads {
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use std::thread::JoinHandle;

    pub struct ThreadHandler {
        sender: Sender<String>,
        pub counter: ThreadCounter
    }

    pub struct ThreadCounter {
        pub count: i8,
        pub max_count: i8
    }

    impl ThreadHandler {
        pub fn create() -> ThreadHandler {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                loop {
                    let received = rx.recv().unwrap();
                    println!("{}", received);
                }
            });
            return ThreadHandler {
                sender: tx,
                counter: ThreadCounter {
                    count: 0,
                    max_count: 4
                }
            };
        }

        pub fn spawn<F, T>(&mut self, f: F) -> () where F : FnOnce() -> T, F: Send + 'static, T: Send + 'static {
            let thread_sender = self.sender.clone();
            thread::spawn(move || {
                thread_sender.send(String::from("thread start"));
                f();
                thread_sender.send(String::from("thread end"))
            });
        }
    }
}
