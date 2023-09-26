use async_trait::async_trait;
use std::{task::{Context, Poll}, future::Future};
use core::{pin::Pin, marker::PhantomData};
use tokio::signal;

/// Channel used to receive OS signals
#[async_trait]
pub trait SignalChannel {
	async fn recv(&mut self) -> Option<()>;
}

#[cfg(unix)]
#[async_trait]
impl SignalChannel for signal::unix::Signal {
	async fn recv(&mut self) -> Option<()> {
		self.recv().await
	}
}

/// A Future that never resolves
pub struct Never<T> {
	output_type: PhantomData<T>
}

impl<T> Never<T> {
	fn new() -> Self {
		Self {
			output_type: PhantomData
		}
	}
}

impl<T> Future for Never<T> {
	type Output = T;
	fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
		return Poll::Pending;
	}
}

/// A SinalChannel that never fires.
/// Used as a placeholder on platforms where certain signals aren't supported.
pub struct SigNever;

#[async_trait]
impl SignalChannel for SigNever {
	async fn recv(&mut self) -> Option<()> {
		Never::<Option<()>>::new().await
	}
}
