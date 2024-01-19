use loom::thread;
use skiplist_rs::{AllocationRecorder, ByteWiseComparator, MemoryLimiter, Skiplist};

#[derive(Clone, Default)]
struct DummyLimiter {}

impl AllocationRecorder for DummyLimiter {
    fn alloc(&self, _: usize, _: usize) {}

    fn free(&self, _: usize, _: usize) {}
}

impl MemoryLimiter for DummyLimiter {
    fn acquire(&self, _: usize) -> bool {
        true
    }

    fn mem_usage(&self) -> usize {
        0
    }

    fn reclaim(&self, _: usize) {}
}

#[test]
fn concurrent_put_and_remove() {
    loom::model(|| {
        let sl = Skiplist::new(ByteWiseComparator {}, std::sync::Arc::new(DummyLimiter {}));
        let n = 100;
        for i in (0..n).step_by(2) {
            let k = format!("k{:04}", i);
            let v = format!("v{:04}", i);
            sl.put(k, v);
        }
        let sl1 = sl.clone();
        let h1 = thread::spawn(move || {
            for i in (1..n).step_by(2) {
                let k = format!("k{:04}", i);
                let v = format!("v{:04}", i);
                sl1.put(k, v);
            }
        });
        let sl1 = sl.clone();
        let h2 = thread::spawn(move || {
            for i in (0..n).step_by(2) {
                let k = format!("k{:04}", i);
                sl1.remove(k.as_bytes());
            }
        });
        h1.join().unwrap();
        h2.join().unwrap();

        for i in (1..n).step_by(2) {
            let k = format!("k{:04}", i);
            let v = format!("v{:04}", i);
            assert_eq!(sl.get(k.as_bytes()).unwrap(), v.as_bytes());
        }
    });
}
