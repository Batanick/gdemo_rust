extern crate time;

static NSEC_PER_SEC: u64 = 1_000_000_000;

pub struct FpsCounter {
    fps_counter: u32,
    last_tick_time: u64,
    time_counter: u64,

    pub current_fps: u32,
}

impl FpsCounter {
    pub fn new() -> Self {
        let current_time = time::precise_time_ns();
        FpsCounter {
            last_tick_time: current_time,
            time_counter: 0,
            fps_counter: 0,
            current_fps: 0,
        }
    }

    pub fn on_frame(&mut self) -> f32 {
        let current_time = time::precise_time_ns();
        let time_delta = current_time - self.last_tick_time;

        self.fps_counter += 1;
        self.time_counter += time_delta;

        if self.time_counter > NSEC_PER_SEC {
            self.current_fps = self.fps_counter;
            self.fps_counter = 0;
            self.time_counter = 0;
        }

        self.last_tick_time = current_time;
        (time_delta as f32) / (NSEC_PER_SEC as f32)
    }
}
