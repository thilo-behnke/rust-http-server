pub mod threads {
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};
    use std::sync::atomic::{AtomicI8, Ordering};
    use std::sync::mpsc::Sender;
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[derive(Debug)]
    pub struct ThreadHandler {
        sender: Sender<ThreadMessageEvent>,
        counter: Arc<ThreadCounter>,
    }

    #[derive(Debug)]
    pub struct ThreadCounter {
        pub count: AtomicI8,
        pub max_count: i8,
    }

    impl ThreadCounter {
        pub fn can_create(&self) -> bool {
            self.count.load(Ordering::SeqCst) < self.max_count
        }
    }

    enum ThreadMessageEvent {
        OPEN,
        CLOSE,
        ERROR(String),
    }

    #[derive(Debug)]
    pub enum ThreadError {
        NoRemainingThreads,
    }

    impl Display for ThreadError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                ThreadError::NoRemainingThreads => write!(f, "No remaining threads."),
            }
        }
    }

    impl std::error::Error for ThreadError {}

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
            let counter = Arc::from(ThreadCounter {
                max_count: 4,
                count: AtomicI8::from(0),
            });
            let thread_handler = ThreadHandler {
                sender: tx,
                counter: Arc::clone(&counter),
            };
            let count_ref = counter.clone();
            thread::spawn(move || loop {
                let counter = Arc::clone(&count_ref);
                if let Ok(message) = rx.recv() {
                    match message {
                        ThreadMessageEvent::OPEN => {
                            counter.count.fetch_add(1, Ordering::SeqCst);
                        }
                        ThreadMessageEvent::CLOSE => {
                            counter.count.fetch_sub(1, Ordering::SeqCst);
                        }
                        ThreadMessageEvent::ERROR(_) => (),
                    }
                    println!("{:?}", counter);
                }
            });
            return thread_handler;
        }

        pub fn spawn<F, T, E>(&mut self, f: F) -> Result<(), ThreadError>
        where
            F: FnOnce() -> Result<T, E>,
            F: Send + 'static,
            T: Send + 'static,
            E: Error,
            E: Send + 'static,
        {
            let thread_sender = self.sender.clone();
            if !self.counter.can_create() {
                return Err(ThreadError::NoRemainingThreads);
            }
            thread::spawn(move || {
                thread_sender
                    .send(ThreadMessageEvent::OPEN)
                    .expect("unable to send message open");
                match f() {
                    Ok(_) => {
                        thread_sender
                            .send(ThreadMessageEvent::CLOSE)
                            .expect("unable to send message close");
                    }
                    Err(e) => {
                        thread_sender
                            .send(ThreadMessageEvent::ERROR(e.to_string()))
                            .expect("unable to send message error");
                    }
                };
            });
            Ok(())
        }
    }
}
