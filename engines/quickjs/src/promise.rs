//! Promise / future handling for Quickjs

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct QuickjsPromise<T> {
    value: Option<T>,
}

impl<T> QuickjsPromise<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    pub fn pending() -> Self {
        Self { value: None }
    }

    pub fn resolve(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }
}

impl<T: Unpin> Future for QuickjsPromise<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        if let Some(value) = this.value.take() {
            Poll::Ready(value)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
