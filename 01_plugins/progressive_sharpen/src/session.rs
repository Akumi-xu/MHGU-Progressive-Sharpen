//! Pure accounting; frames are sampled before and after the native update.
pub fn restore_amount(delta: u32, current: u32, maximum: u32) -> u32 {
    delta.min(maximum.saturating_sub(current))
}

#[derive(Default)]
pub struct Session {
    pub count: u32,
    pub total: u32,
    pub completed: u32,
    pub issued: u32,
    pub consumed: bool,
    last: Option<f32>,
}

impl Session {
    pub fn begin(count: u32, total: u32) -> Option<Self> {
        if !(1..=5).contains(&count) || ![100, 150].contains(&total) {
            return None;
        }
        Some(Self {
            count,
            total,
            ..Self::default()
        })
    }

    // Native skips land exactly on the final stroke; backward jumps begin a new pass.
    pub fn observe(&mut self, before: f32, after: f32) -> u32 {
        if !before.is_finite() || !after.is_finite() {
            return 0;
        }
        let previous = self.last.unwrap_or(before);
        let mut events = 0;
        if before >= previous {
            for frame in [100.0, 144.0, 192.0, 238.0] {
                if previous < frame && before >= frame {
                    events += 1;
                }
            }
        }
        if after == 238.0 && before < 238.0 {
            events += 1;
        }
        self.last = Some(after);
        events.min(self.count.saturating_sub(self.completed))
    }

    pub fn commit(&mut self) -> u32 {
        if self.completed >= self.count {
            return 0;
        }
        self.completed += 1;
        let target = self.total * self.completed / self.count;
        let delta = target - self.issued;
        self.issued = target;
        self.consumed = true;
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_sharpness_is_restored_only_once() {
        for count in 1..=5 {
            let mut session = Session::begin(count, 150).unwrap();
            let mut current = 340;
            let mut native_calls = 0;
            for _ in 0..count {
                let amount = restore_amount(session.commit(), current, 350);
                if amount > 0 {
                    native_calls += 1;
                    current += amount;
                }
            }
            assert_eq!(current, 350);
            assert_eq!(native_calls, 1);
            assert_eq!(session.issued, 150);
        }
        assert_eq!(restore_amount(38, 350, 350), 0);
        assert_eq!(restore_amount(38, 351, 350), 0);
        assert_eq!(restore_amount(38, 0, 0), 0);
        assert_eq!(restore_amount(38, 50, 350), 38);
    }
    #[test]
    fn totals_and_rounding() {
        for n in 1..=5 {
            for total in [100, 150] {
                let mut s = Session::begin(n, total).unwrap();
                assert_eq!((0..n).map(|_| s.commit()).sum::<u32>(), total);
                assert_eq!(s.commit(), 0);
            }
        }
        let mut s = Session::begin(4, 150).unwrap();
        assert_eq!(
            [s.commit(), s.commit(), s.commit(), s.commit()],
            [37, 38, 37, 38]
        );
    }
    #[test]
    fn skip_repeat_and_restart() {
        let mut s = Session::begin(1, 150).unwrap();
        assert_eq!(s.observe(0.0, 0.0), 0);
        assert_eq!(s.observe(96.0, 238.0), 1);
        s.commit();
        assert_eq!(s.observe(238.0, 238.0), 0);
        let mut s = Session::begin(5, 150).unwrap();
        s.observe(0.0, 0.0);
        for frame in [100.0, 144.0, 192.0, 238.0] {
            assert_eq!(s.observe(frame, frame), 1);
            s.commit();
            assert_eq!(s.observe(frame, frame), 0);
        }
        assert_eq!(s.observe(246.0, 200.0), 0);
        assert_eq!(s.observe(238.0, 238.0), 1);
        s.commit();
        assert_eq!(s.issued, 150);
        let restarted = Session::begin(4, 150).unwrap();
        assert!(!restarted.consumed);
        assert_eq!(restarted.issued, 0);
    }
}
