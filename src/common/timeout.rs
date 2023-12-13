use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

#[derive(Clone)]
pub struct Timeout {
    remaining_time: f64,
    clock: Instant,
    quit: Arc<AtomicBool>,
}

impl Timeout {
    pub fn new(time_out: f64) -> Timeout {
        return Timeout {
            remaining_time: time_out,
            clock: Instant::now(),
            quit: Arc::new(AtomicBool::new(false)),
        };
    }

    pub fn new_concurrent(time_out: f64, quit: Arc<AtomicBool>) -> Timeout {
        return Timeout {
            remaining_time: time_out,
            clock: Instant::now(),
            quit,
        };
    }

    pub fn time_finished(&self) -> bool {
        return self.remaining_time - self.clock.elapsed().as_secs_f64() < 0.0
            || self.quit.load(std::sync::atomic::Ordering::Relaxed);
    }

    pub fn remaining_time(&self) -> f64 {
        return self.remaining_time - self.clock.elapsed().as_secs_f64();
    }

    pub fn finish(&self) {
        self.quit.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn reset_finish(&self) {
        self.quit.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_finish_var(&self) -> Arc<AtomicBool> {
        return self.quit.clone();
    }

    pub fn clone_with_new_timeout(&self, time: f64) -> Timeout {
        let mut out = self.clone();
        out.remaining_time = self.clock.elapsed().as_secs_f64() + time;
        return out;
    }
}
