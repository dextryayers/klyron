use crate::error::JSCError;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct JSCPromise {
    inner: Arc<Mutex<PromiseInner>>,
}

struct PromiseInner {
    state: PromiseState,
    waker: Option<Waker>,
}

enum PromiseState {
    Pending,
    Resolved(String),
    Rejected(String),
}

impl JSCPromise {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PromiseInner {
                state: PromiseState::Pending,
                waker: None,
            })),
        }
    }

    pub fn resolve_promise() -> Result<String, JSCError> {
        Err(JSCError::ExecutionFailed("JSC native promise resolver not available".into()))
    }

    pub fn reject_promise() -> Result<String, JSCError> {
        Err(JSCError::ExecutionFailed("JSC native promise rejection not available".into()))
    }

    pub fn resolve(&mut self, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = PromiseState::Resolved(value);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }

    pub fn reject(&mut self, reason: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = PromiseState::Rejected(reason);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}

impl Future for JSCPromise {
    type Output = Result<String, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();
        match &inner.state {
            PromiseState::Pending => {
                inner.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            PromiseState::Resolved(val) => Poll::Ready(Ok(val.clone())),
            PromiseState::Rejected(err) => Poll::Ready(Err(err.clone())),
        }
    }
}
