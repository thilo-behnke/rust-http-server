
pub mod threads {
    use std::thread;
    use std::thread::JoinHandle;

    pub struct ThreadHandler {
        pub counter: ThreadCounter
    }

    pub struct ThreadCounter {
        pub count: i8,
        pub max_count: i8,
        active_handles: Vec<JoinHandle<()>>
    }

    impl ThreadHandler {
        pub fn create() -> ThreadHandler {
            return ThreadHandler {
                counter: ThreadCounter {
                    count: 0,
                    max_count: 4,
                    active_handles: vec![]
                }
            };
        }

        pub fn init(&mut self) {
            thread::spawn(|| {
                loop {
                    let handles = self.counter.active_handles;
                    println!("{:?}", handles);
                }
            });
        }

        pub fn spawn<F, T>(&mut self, f: F) -> () where F : FnOnce() -> T, F: Send + 'static, T: Send + 'static {
            // let counter = &mut self.counter;
            let handle = thread::spawn(|| {
                // counter.count += 1;
                // println!("ThreadHandler spawned thread, currently open threads: {}", counter.count.to_string());
                f();
                // counter.count -= 1;
                // println!("ThreadHandler terminates thread, currently open threads: {}", counter.count.to_string());
            });
            self.counter.active_handles.push(handle);
        }
    }
}
