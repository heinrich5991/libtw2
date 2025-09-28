use std::future;
use std::future::Future;
use std::sync::OnceLock;
use std::thread;
use tokio::runtime;

pub fn spawn<F: Future<Output = ()> + Send + 'static>(future: F) {
    fn handle() -> &'static runtime::Handle {
        static HANDLE: OnceLock<runtime::Handle> = OnceLock::new();
        HANDLE.get_or_init(|| {
            let runtime = runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap();
            let handle = runtime.handle().clone();
            // give the runtime a thread to work with
            thread::spawn(move || runtime.block_on(future::pending::<()>()));
            handle
        })
    }
    let _ = handle().spawn(future);
}
