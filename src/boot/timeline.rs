/// Animation state machine: Entrance → Alive → Freeze → Done
pub enum Phase {
    Entrance,
    Alive,
    Freeze,
    Done,
}

pub struct EntranceTiming {
    pub border_start: u32,
    pub border_end: u32,
    pub stars_start: u32,
    pub stars_end: u32,
    pub celestial_start: u32,
    pub celestial_end: u32,
    pub status_start: u32,
    pub status_end: u32,
    pub footer_start: u32,
    pub entrance_done: u32,
}

pub struct ExitTiming {
    pub flash_frames: u32,
    pub collapse_frames: u32,
}

impl EntranceTiming {
    pub fn slow() -> Self {
        Self {
            border_start: 0,
            border_end: 18,
            stars_start: 6,
            stars_end: 12,
            celestial_start: 12,
            celestial_end: 17,
            status_start: 20,
            status_end: 35,
            footer_start: 30,
            entrance_done: 35,
        }
    }

    pub fn fast() -> Self {
        Self {
            border_start: 0,
            border_end: 6,
            stars_start: 2,
            stars_end: 4,
            celestial_start: 4,
            celestial_end: 6,
            status_start: 6,
            status_end: 12,
            footer_start: 10,
            entrance_done: 12,
        }
    }

    pub fn instant() -> Self {
        Self {
            border_start: 0,
            border_end: 0,
            stars_start: 0,
            stars_end: 0,
            celestial_start: 0,
            celestial_end: 0,
            status_start: 0,
            status_end: 0,
            footer_start: 0,
            entrance_done: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "fast" => Self::fast(),
            "instant" => Self::instant(),
            _ => Self::slow(),
        }
    }
}

impl ExitTiming {
    pub fn slow() -> Self {
        Self {
            flash_frames: 2,
            collapse_frames: 10,
        }
    }

    pub fn fast() -> Self {
        Self {
            flash_frames: 1,
            collapse_frames: 5,
        }
    }

    pub fn instant() -> Self {
        Self {
            flash_frames: 0,
            collapse_frames: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "fast" => Self::fast(),
            "instant" => Self::instant(),
            _ => Self::slow(),
        }
    }

    fn duration(&self) -> u32 {
        self.flash_frames + self.collapse_frames
    }
}

pub struct Timeline {
    pub frame: u32,
    pub phase: Phase,
    pub freeze_start: u32,
    entrance: EntranceTiming,
    exit: ExitTiming,
}

impl Timeline {
    pub fn new(entrance: &str, exit: &str) -> Self {
        Self {
            frame: 0,
            phase: Phase::Entrance,
            freeze_start: 0,
            entrance: EntranceTiming::from_str(entrance),
            exit: ExitTiming::from_str(exit),
        }
    }

    pub fn tick(&mut self) {
        self.frame += 1;

        match self.phase {
            Phase::Entrance => {
                if self.frame >= self.entrance.entrance_done {
                    self.phase = Phase::Alive;
                }
            }
            Phase::Freeze => {
                if self.frame - self.freeze_start >= self.exit.duration() {
                    self.phase = Phase::Done;
                }
            }
            _ => {}
        }
    }

    pub fn trigger_freeze(&mut self) {
        if matches!(self.phase, Phase::Entrance | Phase::Alive) {
            self.phase = Phase::Freeze;
            self.freeze_start = self.frame;
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self.phase, Phase::Done)
    }

    // --- Progress getters (0.0 to 1.0) ---

    pub fn border_progress(&self) -> f32 {
        ramp(self.frame, self.entrance.border_start, self.entrance.border_end)
    }

    pub fn star_visibility(&self) -> f32 {
        match self.phase {
            Phase::Entrance => ramp(self.frame, self.entrance.stars_start, self.entrance.stars_end),
            Phase::Freeze => {
                if self.is_freeze_flash() {
                    1.0
                } else {
                    0.0 // scene content gone during collapse
                }
            }
            Phase::Done => 0.0,
            _ => 1.0,
        }
    }

    pub fn celestial_visibility(&self) -> f32 {
        match self.phase {
            Phase::Entrance => ramp(self.frame, self.entrance.celestial_start, self.entrance.celestial_end),
            Phase::Freeze => {
                if self.is_freeze_flash() {
                    1.0
                } else {
                    0.0
                }
            }
            Phase::Done => 0.0,
            _ => 1.0,
        }
    }

    pub fn status_progress(&self) -> f32 {
        match self.phase {
            Phase::Entrance => ramp(self.frame, self.entrance.status_start, self.entrance.status_end),
            _ => 1.0,
        }
    }

    pub fn footer_visible(&self) -> bool {
        match self.phase {
            Phase::Entrance => self.frame >= self.entrance.footer_start,
            Phase::Alive => {
                // Blink: ~1s on, ~1s off
                ((self.frame - self.entrance.entrance_done) / 30) % 2 == 0
            }
            Phase::Freeze => true,
            Phase::Done => false,
        }
    }

    pub fn gradient_active(&self) -> bool {
        matches!(self.phase, Phase::Alive)
    }

    pub fn is_freeze_flash(&self) -> bool {
        matches!(self.phase, Phase::Freeze)
            && (self.frame - self.freeze_start) < self.exit.flash_frames
    }

    /// Collapse progress as 0.0..=1.0 with ease-out (fast start, slow finish).
    pub fn collapse_progress(&self) -> f32 {
        if self.exit.collapse_frames == 0 {
            return if matches!(self.phase, Phase::Freeze | Phase::Done) { 1.0 } else { 0.0 };
        }
        let t = match self.phase {
            Phase::Done => 1.0,
            Phase::Freeze => {
                let elapsed = self.frame - self.freeze_start;
                if elapsed < self.exit.flash_frames {
                    0.0
                } else {
                    let linear = (elapsed - self.exit.flash_frames) as f32
                        / self.exit.collapse_frames as f32;
                    linear.min(1.0)
                }
            }
            _ => 0.0,
        };
        // Ease-in quart: slow start, accelerates
        t * t * t * t
    }

    /// True when we're in the collapse portion of freeze (after flash).
    pub fn is_collapsing(&self) -> bool {
        matches!(self.phase, Phase::Freeze) && !self.is_freeze_flash()
    }
}

fn ramp(frame: u32, start: u32, end: u32) -> f32 {
    if start == end {
        return 1.0;
    }
    if frame < start {
        0.0
    } else if frame >= end {
        1.0
    } else {
        (frame - start) as f32 / (end - start) as f32
    }
}
