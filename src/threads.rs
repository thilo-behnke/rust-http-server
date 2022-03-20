
pub mod threads {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;

    #[derive(Debug)]
    pub struct ThreadHandler {
        receiver: Receiver<ThreadMessageEvent>,
        sender: Sender<ThreadMessageEvent>,
        counter: ThreadCounter
    }

    #[derive(Debug)]
    pub struct ThreadCounter {
        pub count: i8,
        pub max_count: i8
    }

    enum ThreadMessageEvent {
        OPEN, CLOSE, ERROR(String)
    }

    impl fmt::Display for ThreadMessageEvent {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                ThreadMessageEvent::OPEN => write!(f, "Open thread"),
                ThreadMessageEvent::CLOSE => write!(f, "Close thread"),
                ThreadMessageEvent::ERROR(e) => write!(f, "Error when handling thread: {}", e),
            }
        }
    }

    impl ThreadHandler {
        pub fn create() -> ThreadHandler {
            let (tx, rx) = mpsc::channel();
            let thread_handler = ThreadHandler {
                receiver: rx,
                sender: tx,
                counter: ThreadCounter {
                    count: 0,
                    max_count: 4
                }
            };
            return thread_handler;
        }

        // pub fn init(&'static mut self) {
        //     let mut counter: &'static mut ThreadCounter = &mut self.counter;
        //     let mut receiver: &'static mut Receiver<ThreadMessageEvent> = &mut self.receiver;
        //     thread::spawn(move || {
        //         loop {
        //             if let Ok(message) = receiver.recv() {
        //                 match message {
        //                     ThreadMessageEvent::OPEN => counter.count +=1,
        //                     ThreadMessageEvent::CLOSE => counter.count -=1,
        //                     ThreadMessageEvent::ERROR(e) => ()
        //                 }
        //                 println!("{:?}", counter);
        //             }
        //         }
        //     });
        // }

        pub fn spawn<F, T, E>(&mut self, f: F) -> () where F : FnOnce() -> Result<T, E>, F: Send + 'static, T: Send + 'static, E: Error, E: Send + 'static {
            let thread_sender = self.sender.clone();
            thread::spawn(move || {
                thread_sender.send(ThreadMessageEvent::OPEN).expect("unable to send message open");
                match f() {
                    Ok(_) => {
                        thread_sender.send(ThreadMessageEvent::CLOSE).expect("unable to send message close");
                    },
                    Err(e) => {
                        thread_sender.send(ThreadMessageEvent::ERROR(e.to_string())).expect("unable to send message error");
                    }
                };
            });
            for message in self.receiver.recv() {
                match message {
                    ThreadMessageEvent::
                }
            }
        }
    }
}
