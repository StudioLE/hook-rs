//! Deny rules for `docker compose config`, which renders secrets.

use crate::prelude::*;

const REASON: &str =
    "`docker compose config` is blocked. Exposes secrets by rendering interpolated env vars";

/// Deny `docker compose config` and `docker-compose config`.
#[must_use]
pub fn docker_compose_rules() -> Vec<BashRule> {
    vec![docker_compose_config(), docker_compose_v1_config()]
}

/// Deny `docker compose config`.
///
/// - Matches `config` anywhere in the args, since global options such as `--profile` may precede it
/// - Also denies a service or profile named `config`, e.g. `docker compose logs config`
fn docker_compose_config() -> BashRule {
    BashRule {
        id: "docker_compose_config".to_owned(),
        command: "docker compose".to_owned(),
        with_any: Some(vec![ArgMatcher::new("config")]),
        outcome: Outcome::deny(REASON),
        ..Default::default()
    }
}

/// Deny `docker-compose config`.
fn docker_compose_v1_config() -> BashRule {
    BashRule {
        id: "docker_compose_v1_config".to_owned(),
        command: "docker-compose".to_owned(),
        with_any: Some(vec![ArgMatcher::new("config")]),
        outcome: Outcome::deny(REASON),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn docker_compose_config() {
        let result = eval_rules(docker_compose_rules(), "docker compose config");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn docker_compose_config_after_global_options() {
        let result = eval_rules(
            docker_compose_rules(),
            "docker compose -f compose.yml --profile backup config web 2>&1 | head -40",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn docker_compose_v1_config() {
        let result = eval_rules(docker_compose_rules(), "docker-compose config --services");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn docker_compose_ps() {
        let result = eval_rules(docker_compose_rules(), "docker compose ps");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
