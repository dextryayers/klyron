use crate::error::BoaError;
use boa_engine::{Context, JsValue};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll, Waker};

pub struct BoaPromise {
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

impl BoaPromise {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PromiseInner {
                state: PromiseState::Pending,
                waker: None,
            })),
        }
    }

    pub fn resolve_js(val: &JsValue, context: &mut Context) -> Result<String, BoaError> {
        if val.is_null() || val.is_undefined() {
            return Ok(String::new());
        }
        let s = val.to_string(context)
            .map_err(|e| BoaError::ExecutionFailed(e.to_string()))?;
        Ok(s.to_std_string_escaped())
    }

    pub fn reject_js(val: &JsValue, context: &mut Context) -> Result<String, BoaError> {
        let s = val.to_string(context)
            .map_err(|e| BoaError::ExecutionFailed(e.to_string()))?;
        Err(BoaError::ExecutionFailed(s.to_std_string_escaped()))
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

impl Future for BoaPromise {
    type Output = Result<String, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
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
