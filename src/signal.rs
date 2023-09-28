use async_trait::async_trait;
use std::{task::{Context, Poll}, future::Future, io};
use core::{pin::Pin, marker::PhantomData};
#[cfg(unix)]
use tokio::signal::unix as unix_signal;


/// Channel used to receive OS signals
#[async_trait]
pub trait SignalChannel {
	async fn recv(&mut self) -> Option<()>;
}

#[cfg(unix)]
#[async_trait]
impl SignalChannel for unix_signal::Signal {
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

/// Unix signal types accepted by new()
pub enum SignalKind {
	SIGTERM
}

/// Open a new signal channel
#[cfg(unix)]
pub fn new(kind: SignalKind) -> io::Result<Box<dyn SignalChannel>> {
	match kind {
		SignalKind::SIGTERM => Ok(Box::new(unix_signal::signal(unix_signal::SignalKind::terminate())?))
	}	
}

/// Open a new signal channel
/// (non-Unix platform, always returns Ok(SigNever))
#[cfg(not(unix))]
pub fn new(_kind: SignalKind) -> io::Result<Box<dyn SignalChannel>> {
	Ok(Box::new(SigNever))
}

