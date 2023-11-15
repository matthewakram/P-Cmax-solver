use std::time::Instant;


pub struct Timeout{
    remaining_time: f64,
    clock: Instant,
}

impl Timeout {
    pub fn new(time_out: f64) -> Timeout{
        return Timeout { remaining_time: time_out, clock: Instant::now() }
    }

    pub fn time_finished(&self) -> bool{
        return self.remaining_time - self.clock.elapsed().as_secs_f64() < 0.0;
    }

    pub fn remaining_time(&self) -> f64{
        return self.remaining_time - self.clock.elapsed().as_secs_f64();
    }
}