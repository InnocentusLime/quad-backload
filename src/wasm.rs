use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use macroquad::miniquad;

pub struct BackgroundFileHandle {
    path: String,
    contents: Arc<Mutex<Option<miniquad::fs::Response>>>,
}

impl Future for BackgroundFileHandle {
    type Output = Result<Vec<u8>, macroquad::Error>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // NOTE: miniquad's file loader on wasm is already background
        let mut contents = self.contents.lock().unwrap();
        if let Some(contents) = contents.take() {
            Poll::Ready(match contents {
                Ok(data) => Ok(data),
                Err(e) => Err(macroquad::Error::FileError {
                    kind: e,
                    path: self.path.clone(),
                }),
            })
        } else {
            Poll::Pending
        }
    }
}

pub struct BackgroundLoader;

impl BackgroundLoader {
    pub fn new() -> Self {
        Self
    }

    /// Returns a future for background loading of a file.
    /// The loading will immediately start asynchronously.
    /// Awaiting the returned future will block until the file data is ready.
    pub fn load_file(&mut self, path: &str) -> impl Future<Output = Result<Vec<u8>, macroquad::Error>> {
        let contents = Arc::new(Mutex::new(None));
        let res_clone = contents.clone();
        miniquad::fs::load_file(path, move |out| {
            *res_clone.lock().unwrap() = Some(out);
        });

        BackgroundFileHandle {
            path: path.to_string(),
            contents,
        }
    }
}
