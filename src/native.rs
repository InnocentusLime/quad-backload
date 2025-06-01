use std::pin::Pin;
use std::sync::mpsc;
use std::task::{Context, Poll};
use std::thread;

use macroquad::miniquad;

struct Request {
    path: String,
    tx_result: mpsc::SyncSender<miniquad::fs::Response>,
}

pub struct BackgroundFileHandle {
    path: String,
    rx_result: mpsc::Receiver<miniquad::fs::Response>,
}

impl Future for BackgroundFileHandle {
    type Output = Result<Vec<u8>, macroquad::Error>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.rx_result.try_recv() {
            Ok(x) => match x {
                Ok(data) => Poll::Ready(Ok(data)),
                Err(e) => Poll::Ready(Err(macroquad::Error::FileError {
                    kind: e,
                    path: self.path.clone(),
                })),
            },
            Err(mpsc::TryRecvError::Empty) => Poll::Pending,
            Err(mpsc::TryRecvError::Disconnected) => panic!("The worker is dead"),
        }
    }
}

pub struct BackgroundLoader {
    requests: mpsc::Sender<Request>,
    _loader_thread: thread::JoinHandle<()>,
}

impl BackgroundLoader {
    pub fn new() -> Self {
        let (tx_requests, rx_requests) = mpsc::channel();
        let loader_thread = thread::spawn(|| {
            loader_body(rx_requests);
        });

        Self {
            requests: tx_requests,
            _loader_thread: loader_thread,
        }
    }

    /// Returns a future for background loading of a file.
    /// The loading will immediately start asynchronously.
    /// Awaiting the returned future will block until the file data is ready.
    pub fn load_file(&mut self, path: &str) -> impl Future<Output = Result<Vec<u8>, macroquad::Error>> {
        let (tx_result, rx_result) = mpsc::sync_channel(1);
        self.requests
            .send(Request {
                path: path.to_string(),
                tx_result,
            })
            .unwrap();

        BackgroundFileHandle {
            rx_result,
            path: path.to_string(),
        }
    }
}

fn loader_body(requests: mpsc::Receiver<Request>) {
    use std::sync::{Arc, Mutex};

    for request in requests {
        let res = Arc::new(Mutex::new(None));
        let res_clone = res.clone();
        miniquad::fs::load_file(&request.path, move |out| {
            *res_clone.lock().unwrap() = Some(out);
        });
        let _ = request.tx_result.send(res.lock().unwrap().take().unwrap());
    }
}
