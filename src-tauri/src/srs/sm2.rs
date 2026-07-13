use chrono::{Duration, NaiveDate};

/// User-configurable SM-2 parameters.
#[derive(Debug, Clone)]
pub struct Sm2Config {
    pub initial_ease_factor: f64,
    pub min_ease_factor: f64,
    pub ease_penalty: f64,
    pub pass_threshold: u8,
    pub intervals: [u32; 2],
}

impl Default for Sm2Config {
    fn default() -> Self {
        Self {
            initial_ease_factor: 2.5,
            min_ease_factor: 1.3,
            ease_penalty: 0.2,
            pass_threshold: 3,
            intervals: [1, 6],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sm2State {
    pub interval: u32,
    pub ease_factor: f64,
    pub repetitions: u32,
    pub next_review: NaiveDate,
}

impl Sm2State {
    /// Create a fresh SM-2 state for a card that has never been reviewed.
    #[cfg(test)]
    pub fn new(today: NaiveDate, config: &Sm2Config) -> Self {
        Self {
            interval: 0,
            ease_factor: config.initial_ease_factor.clamp(config.min_ease_factor, 5.0),
            repetitions: 0,
            next_review: today,
        }
    }
}

/// Advance the SM-2 state by one review.
///
/// `quality` is on the standard SM-2 scale:
/// - 0: complete blackout
/// - 1: incorrect, but answer felt familiar
/// - 2: incorrect, but answer seemed easy to recall once seen
/// - 3: correct, but required serious effort to recall
/// - 4: correct, after some hesitation
/// - 5: perfect, effortless recall
///
/// Today's date is passed in explicitly so this function stays pure
/// (no system clock dependency).
pub fn sm2_advance(state: &Sm2State, quality: u8, today: NaiveDate, config: &Sm2Config) -> Result<Sm2State, String> {
    if quality > 5 {
        return Err("quality must be 0-5".to_string());
    }

    if quality < config.pass_threshold {
        // Failed: reset
        let new_ef = (state.ease_factor - config.ease_penalty).clamp(config.min_ease_factor, 5.0);
        Ok(Sm2State {
            interval: 1,
            ease_factor: new_ef,
            repetitions: 0,
            next_review: today + Duration::days(1),
        })
    } else {
        // Passed: advance
        let new_interval = match state.repetitions {
            0 => config.intervals[0],
            1 => config.intervals[1],
            _ => {
                let interval = state.interval as f64 * state.ease_factor;
                interval.round() as u32
            }
        };

        let q = quality as f64;
        let new_ef = state.ease_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
        let new_ef = new_ef.clamp(config.min_ease_factor, 5.0);

        Ok(Sm2State {
            interval: new_interval.max(1),
            ease_factor: new_ef,
            repetitions: state.repetitions + 1,
            next_review: today + Duration::days(new_interval as i64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_new_card() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let st = Sm2State::new(t, &cfg);
        assert_eq!(st.interval, 0);
        assert_eq!(st.ease_factor, 2.5);
        assert_eq!(st.repetitions, 0);
        assert_eq!(st.next_review, t);
    }

    #[test]
    fn test_new_card_custom_ef() {
        let t = date("2026-01-01");
        let cfg = Sm2Config {
            initial_ease_factor: 2.8,
            ..Default::default()
        };
        let st = Sm2State::new(t, &cfg);
        assert_eq!(st.ease_factor, 2.8);
    }

    #[test]
    fn test_first_pass_quality_4() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let init = Sm2State::new(t, &cfg);
        let st = sm2_advance(&init, 4, t, &cfg).unwrap();
        assert_eq!(st.interval, 1);
        assert_eq!(st.repetitions, 1);
        assert_eq!(st.next_review, date("2026-01-02"));
    }

    #[test]
    fn test_second_pass_quality_4() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let init = Sm2State::new(t, &cfg);
        let st1 = sm2_advance(&init, 4, t, &cfg).unwrap();
        let t2 = date("2026-01-02");
        let st2 = sm2_advance(&st1, 4, t2, &cfg).unwrap();
        assert_eq!(st2.interval, 6);
        assert_eq!(st2.repetitions, 2);
        assert_eq!(st2.next_review, date("2026-01-08"));
    }

    #[test]
    fn test_third_pass_quality_4() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let init = Sm2State::new(t, &cfg);
        let st1 = sm2_advance(&init, 4, t, &cfg).unwrap();
        let st2 = sm2_advance(&st1, 4, t, &cfg).unwrap();
        let st3 = sm2_advance(&st2, 4, t, &cfg).unwrap();
        assert_eq!(st3.interval, 15);
        assert_eq!(st3.repetitions, 3);
    }

    #[test]
    fn test_failure_resets() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let init = Sm2State::new(t, &cfg);
        let st1 = sm2_advance(&init, 4, t, &cfg).unwrap();
        let st2 = sm2_advance(&st1, 4, t, &cfg).unwrap();
        let st3 = sm2_advance(&st2, 1, t, &cfg).unwrap();
        assert_eq!(st3.interval, 1);
        assert_eq!(st3.repetitions, 0);
        assert_eq!(st3.ease_factor, 2.3);
        assert_eq!(st3.next_review, date("2026-01-02"));
    }

    #[test]
    fn test_ease_factor_never_below_1_3() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let mut st = Sm2State::new(t, &cfg);
        for _ in 0..10 {
            st = sm2_advance(&st, 0, t, &cfg).unwrap();
        }
        assert_eq!(st.ease_factor, 1.3);
    }

    #[test]
    fn test_ease_factor_max_clamped_at_5() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let mut st = Sm2State {
            interval: 10,
            ease_factor: 4.95,
            repetitions: 5,
            next_review: t,
        };
        st = sm2_advance(&st, 5, t, &cfg).unwrap();
        assert_eq!(st.ease_factor, 5.0);
    }

    #[test]
    fn test_invalid_quality_returns_err() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let st = Sm2State::new(t, &cfg);
        let res = sm2_advance(&st, 6, t, &cfg);
        assert!(res.is_err());
    }

    #[test]
    fn test_perfect_recall_increases_ease() {
        let t = date("2026-01-01");
        let cfg = Sm2Config::default();
        let init = Sm2State::new(t, &cfg);
        let st = sm2_advance(&init, 5, t, &cfg).unwrap();
        assert!(st.ease_factor > 2.5);
    }

    #[test]
    fn test_custom_pass_threshold() {
        let t = date("2026-01-01");
        let cfg = Sm2Config {
            pass_threshold: 4,
            ..Default::default()
        };
        let init = Sm2State::new(t, &cfg);

        let st_failed = sm2_advance(&init, 3, t, &cfg).unwrap();
        assert_eq!(st_failed.repetitions, 0);

        let st_passed = sm2_advance(&init, 4, t, &cfg).unwrap();
        assert_eq!(st_passed.repetitions, 1);
    }
}
