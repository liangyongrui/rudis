use std::sync::Arc;

use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct Limit(Arc<Semaphore>);

impl Limit {
    pub fn new(permits: usize) -> Self {
        Self(Arc::new(Semaphore::new(permits)))
    }

    pub async fn acquire(&self) {
        // Wait for a permit to become available
        //
        // `acquire` returns a permit that is bound via a lifetime to the
        // semaphore. When the permit value is dropped, it is automatically
        // returned to the semaphore. This is convenient in many cases.
        // However, in this case, the permit must be returned in a different
        // task than it is acquired in (the handler task). To do this, we
        // "forget" the permit, which drops the permit value **without**
        // incrementing the semaphore's permits. Then, in the handler task
        // we manually add a new permit when processing completes.
        //
        // `acquire()` returns `Err` when the semaphore has been closed. We
        // don't ever close the sempahore, so `unwrap()` is safe.
        self.0.acquire().await.unwrap().forget();
    }
}

impl Drop for Limit {
    fn drop(&mut self) {
        // Add a permit back to the semaphore.
        self.0.add_permits(1);
    }
}
