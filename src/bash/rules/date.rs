//! Allow rules for `date` without setting the clock or reading files.

use crate::prelude::*;

/// Rules for `date`.
#[must_use]
pub fn date_rules() -> Vec<BashRule> {
    vec![date()]
}

/// Allow `date` without `-s`/`--set`, the `MMDDhhmm` operand, or `-f`/`--file`.
///
/// - `-s` and the `MMDDhhmm[[CC]YY][.ss]` operand set the clock
/// - `-f` echoes each line of the file in its error output
/// - `--s*` and `--f*` also catch GNU abbreviations such as `--se`
/// - The 8-digit glob also catches `-d 20260101`, which then prompts
fn date() -> BashRule {
    BashRule {
        id: "date".to_owned(),
        command: "date".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("-s"),
            ArgMatcher::new("--s*"),
            ArgMatcher::new("[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]*"),
            ArgMatcher::new("-f"),
            ArgMatcher::new("--f*"),
        ]),
        outcome: Outcome::allow("Read-only `date`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn date_format() {
        let result = eval_rules(date_rules(), "date '+%Y-%m-%d %H:%M UTC'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn date_epoch_with_format() {
        let result = eval_rules(date_rules(), "date -d @1500000000 '+%Y/%m/%d'");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn date_set() {
        let result = eval_rules(date_rules(), "date -us 10:00");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// GNU accepts `--se` as an abbreviation of `--set`.
    #[test]
    fn date_set_abbreviated() {
        let result = eval_rules(date_rules(), "date --se=10:00");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn date_set_operand() {
        let result = eval_rules(date_rules(), "date 010112002026");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn date_file() {
        let result = eval_rules(date_rules(), "date --file=.env");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
