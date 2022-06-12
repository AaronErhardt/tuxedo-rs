pub mod dbus;
pub mod runtime;

use std::{future::Future, pin::Pin, task::Poll};

struct NeverFuture();

impl Future for NeverFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
