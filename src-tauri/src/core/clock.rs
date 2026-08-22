//! Injectable time source so retention logic is unit-testable.

use crate::core::model::UnixMs;

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> UnixMs;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> UnixMs {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as UnixMs)
            .unwrap_or(0)
    }
}

#[cfg(test)]
pub struct MockClock(pub std::sync::atomic::AtomicI64);

#[cfg(test)]
impl MockClock {
    pub fn at(ms: UnixMs) -> Self {
        MockClock(std::sync::atomic::AtomicI64::new(ms))
    }
    pub fn set(&self, ms: UnixMs) {
        self.0.store(ms, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
impl Clock for MockClock {
    fn now_ms(&self) -> UnixMs {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}
