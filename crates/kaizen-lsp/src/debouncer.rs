//! Debouncer for rate-limiting analysis during rapid typing

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::Notify;
use tower_lsp::lsp_types::Url;

const DEFAULT_DEBOUNCE_DELAY_MS: u64 = 150;

#[allow(dead_code)]
pub struct Debouncer {
    delay: Duration,
    pending: Arc<DashMap<Url, Arc<Notify>>>,
}

impl Debouncer {
    pub fn new() -> Self {
        Self::with_delay(Duration::from_millis(DEFAULT_DEBOUNCE_DELAY_MS))
    }

    pub fn with_delay(delay: Duration) -> Self {
        Self {
            delay,
            pending: Arc::new(DashMap::new()),
        }
    }

    pub fn schedule<F, Fut>(&self, uri: Url, callback: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        if let Some(existing) = self.pending.get(&uri) {
            existing.notify_one();
        }

        let cancel = Arc::new(Notify::new());
        let cancel_clone = cancel.clone();
        let delay = self.delay;

        self.pending.insert(uri.clone(), cancel);

        let pending = self.pending.clone();
        let uri_clone = uri.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(delay) => {
                    pending.remove(&uri_clone);
                    callback().await;
                }
                _ = cancel_clone.notified() => {
                    // Cancelled, do nothing
                }
            }
        });
    }

    pub fn cancel(&self, uri: &Url) {
        if let Some((_, cancel)) = self.pending.remove(uri) {
            cancel.notify_one();
        }
    }

    #[allow(dead_code)]
    pub fn is_pending(&self, uri: &Url) -> bool {
        self.pending.contains_key(uri)
    }
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    fn test_uri(name: &str) -> Url {
        Url::parse(&format!("file:///test/{}.js", name)).unwrap()
    }

    #[tokio::test]
    async fn debounce_single_call_executes() {
        let debouncer = Arc::new(Debouncer::with_delay(Duration::from_millis(10)));
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        debouncer.schedule(test_uri("test"), move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn debounce_rapid_changes_executes_once() {
        let debouncer = Arc::new(Debouncer::with_delay(Duration::from_millis(50)));
        let counter = Arc::new(AtomicUsize::new(0));
        let uri = test_uri("test");

        for _ in 0..5 {
            let counter_clone = counter.clone();
            debouncer.schedule(uri.clone(), move || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            });
            sleep(Duration::from_millis(10)).await;
        }

        sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cancel_prevents_execution() {
        let debouncer = Arc::new(Debouncer::with_delay(Duration::from_millis(50)));
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let uri = test_uri("test");

        debouncer.schedule(uri.clone(), move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        debouncer.cancel(&uri);

        sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn different_uris_debounce_independently() {
        let debouncer = Arc::new(Debouncer::with_delay(Duration::from_millis(20)));
        let counter = Arc::new(AtomicUsize::new(0));

        let uri1 = test_uri("file1");
        let uri2 = test_uri("file2");

        let counter1 = counter.clone();
        debouncer.schedule(uri1, move || {
            let c = counter1.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        let counter2 = counter.clone();
        debouncer.schedule(uri2, move || {
            let c = counter2.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        sleep(Duration::from_millis(60)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn is_pending_returns_true_for_scheduled() {
        let debouncer = Debouncer::with_delay(Duration::from_secs(10));
        let uri = test_uri("test");

        debouncer.schedule(uri.clone(), || async {});

        assert!(debouncer.is_pending(&uri));
    }

    #[test]
    fn is_pending_returns_false_for_unknown() {
        let debouncer = Debouncer::new();
        let uri = test_uri("unknown");

        assert!(!debouncer.is_pending(&uri));
    }
}
