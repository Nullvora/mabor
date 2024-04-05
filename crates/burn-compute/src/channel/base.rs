use crate::server::{BufHandle, ComputeServer, TensorBufHandle};
use alloc::vec::Vec;
use burn_common::reader::Reader;

/// The ComputeChannel trait links the ComputeClient to the ComputeServer
/// while ensuring thread-safety
pub trait ComputeChannel<Server: ComputeServer>: Clone + core::fmt::Debug + Send + Sync {
    /// Given a handle, returns owned resource as bytes
    fn read(&self, handle: BufHandle<Server>) -> Reader<Vec<u8>>;

    /// Given a resource as bytes, stores it and returns the resource handle
    fn create(&self, data: &[u8]) -> TensorBufHandle<Server>;

    /// Reserves `size` bytes in the storage, and returns a handle over them
    fn empty(&self, size: usize) -> TensorBufHandle<Server>;

    /// Executes the `kernel` over the given `handles`.
    fn execute(&self, kernel: Server::Kernel, handles: Vec<BufHandle<Server>>);

    /// Wait for the completion of every task in the server.
    fn sync(&self);
}
