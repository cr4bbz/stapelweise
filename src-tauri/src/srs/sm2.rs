use chrono::{NaiveDate, Duration};

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
    pub fn new(today: NaiveDate, config: &Sm2Config) -> Self {
        Self {
            interval: 0,
            ease_factor: config.initial_ease_factor,
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
        let new_ef = (state.ease_factor - config.ease_penalty).max(config.min_ease_factor);
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
        let new_ef = new_ef.max(config.min_ease_factor);

        Ok(Sm2State {
            interval: new_interval,
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

    fn default_config() -> Sm2Config {
        Sm2Config::default()
    }

    #[test]
    fn test_new_card() {
        let today = date("2026-07-01");
        let config = default_config();
        let state = Sm2State::new(today, &config);
        assert_eq!(state.interval, 0);
        assert_eq!(state.ease_factor, 2.5);
        assert_eq!(state.repetitions, 0);
        assert_eq!(state.next_review, today);
    }

    #[test]
    fn test_new_card_custom_ef() {
        let today = date("2026-07-01");
        let config = Sm2Config { initial_ease_factor: 3.0, ..default_config() };
        let state = Sm2State::new(today, &config);
        assert_eq!(state.ease_factor, 3.0);
    }

    #[test]
    fn test_first_pass_quality_4() {
        let today = date("2026-07-01");
        let config = default_config();
        let state = Sm2State::new(today, &config);
        let next = sm2_advance(&state, 4, today, &config).unwrap();

        assert_eq!(next.interval, 1);
        assert_eq!(next.repetitions, 1);
        assert_eq!(next.next_review, date("2026-07-02"));
    }

    #[test]
    fn test_second_pass_quality_4() {
        let today = date("2026-07-01");
        let config = default_config();
        let state = Sm2State {
            interval: 1,
            ease_factor: 2.5,
            repetitions: 1,
            next_review: today,
        };
        let next = sm2_advance(&state, 4, today, &config).unwrap();

        assert_eq!(next.interval, 6);
        assert_eq!(next.repetitions, 2);
        assert_eq!(next.next_review, date("2026-07-07"));
    }

    #[test]
    fn test_third_pass_quality_4() {
        let today = date("2026-07-07");
        let config = default_config();
        let state = Sm2State {
            interval: 6,
            ease_factor: 2.5,
            repetitions: 2,
            next_review: today,
        };
        let next = sm2_advance(&state, 4, today, &config).unwrap();

        // interval = 6 * 2.5 = 15
        assert_eq!(next.interval, 15);
        assert_eq!(next.repetitions, 3);
        assert_eq!(next.next_review, date("2026-07-22"));
    }

    #[test]
    fn test_failure_resets() {
        let today = date("2026-07-15");
        let config = default_config();
        let state = Sm2State {
            interval: 30,
            ease_factor: 2.5,
            repetitions: 5,
            next_review: today,
        };
        let next = sm2_advance(&state, 2, today, &config).unwrap(); // quality 2 = failed

        assert_eq!(next.interval, 1);
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.ease_factor, 2.3); // 2.5 - 0.2
        assert_eq!(next.next_review, date("2026-07-16"));
    }

    #[test]
    fn test_custom_pass_threshold() {
        let today = date("2026-07-01");
        let config = Sm2Config { pass_threshold: 4, ..default_config() };
        let state = Sm2State::new(today, &config);

        // quality 3 should now fail (threshold is 4)
        let next = sm2_advance(&state, 3, today, &config).unwrap();
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.interval, 1);

        // quality 4 should pass
        let next = sm2_advance(&state, 4, today, &config).unwrap();
        assert_eq!(next.repetitions, 1);
    }

    #[test]
    fn test_ease_factor_never_below_1_3() {
        let today = date("2026-07-01");
        let config = default_config();
        let mut state = Sm2State {
            interval: 1,
            ease_factor: 1.3,
            repetitions: 0,
            next_review: today,
        };

        for _ in 0..10 {
            state = sm2_advance(&state, 0, today, &config).unwrap(); // repeated failures
        }

        assert!(state.ease_factor >= 1.3);
    }

    #[test]
    fn test_perfect_recall_increases_ease() {
        let today = date("2026-07-01");
        let config = default_config();
        let state = Sm2State {
            interval: 1,
            ease_factor: 2.5,
            repetitions: 1,
            next_review: today,
        };
        let next = sm2_advance(&state, 5, today, &config).unwrap(); // perfect recall

        // EF increase for quality 5: 0.1 - 0 = 0.1
        assert!((next.ease_factor - 2.6).abs() < 0.01);
    }
}
