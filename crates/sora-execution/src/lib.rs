use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionOptions {
    pub parallel: bool,
    pub jobs: Option<usize>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            jobs: None,
        }
    }
}

#[derive(Clone)]
pub struct ExecutionContext {
    options: ExecutionOptions,
    pool: Option<Arc<ThreadPool>>,
    cancelled: Arc<AtomicBool>,
    cancellation_probe: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}

impl ExecutionContext {
    pub fn new(options: ExecutionOptions) -> Result<Self, ThreadPoolBuildError> {
        let pool = match (options.parallel, options.jobs) {
            (true, Some(jobs)) => Some(Arc::new(
                ThreadPoolBuilder::new().num_threads(jobs).build()?,
            )),
            _ => None,
        };

        Ok(Self {
            options,
            pool,
            cancelled: Arc::new(AtomicBool::new(false)),
            cancellation_probe: None,
        })
    }

    pub fn serial() -> Self {
        Self {
            options: ExecutionOptions {
                parallel: false,
                jobs: Some(1),
            },
            pool: None,
            cancelled: Arc::new(AtomicBool::new(false)),
            cancellation_probe: None,
        }
    }

    pub fn options(&self) -> ExecutionOptions {
        self.options
    }

    /// Requests cooperative cancellation for work using this context.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns whether cooperative cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
            || self
                .cancellation_probe
                .as_ref()
                .is_some_and(|probe| probe())
    }

    /// Returns a derived context that also observes an adapter cancellation source.
    pub fn with_cancellation_probe(
        mut self,
        probe: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Self {
        self.cancellation_probe = Some(Arc::new(probe));
        self
    }

    pub fn map<T, R, E, F>(&self, items: Vec<T>, map_item: F) -> Result<Vec<R>, E>
    where
        T: Send,
        R: Send,
        E: Send,
        F: Fn(T) -> Result<R, E> + Send + Sync,
    {
        if !self.options.parallel || items.len() <= 1 {
            return items.into_iter().map(map_item).collect();
        }

        match &self.pool {
            Some(pool) => pool.install(|| items.into_par_iter().map(map_item).collect()),
            None => items.into_par_iter().map(map_item).collect(),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new(ExecutionOptions::default())
            .expect("default execution context must use a valid thread pool configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_context_preserves_order() {
        let context = ExecutionContext::serial();

        let values = context
            .map(vec![1, 2, 3], |value| Ok::<_, ()>(value * 2))
            .unwrap();

        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn parallel_context_preserves_order() {
        let context = ExecutionContext::new(ExecutionOptions {
            parallel: true,
            jobs: Some(2),
        })
        .unwrap();

        let values = context
            .map(vec![1, 2, 3], |value| Ok::<_, ()>(value * 2))
            .unwrap();

        assert_eq!(values, vec![2, 4, 6]);
    }
}
