//! Produce the full set of [`BashRule`] for evaluation.

use crate::prelude::*;

/// Produce [`BashRule`] instances for evaluation.
///
/// - Defaults to the full rule set
/// - Tests can construct directly to control which rules activate
pub struct BashRuleProvider {
    rules: Vec<BashRule>,
}

impl BashRuleProvider {
    /// All loaded rules.
    pub fn get(&self) -> &[BashRule] {
        &self.rules
    }

    /// Build the complete set of rules.
    fn all() -> Vec<BashRule> {
        let mut rules = Vec::new();
        rules.extend(rm_rules());
        rules.push(awk());
        rules.extend(cargo_rules());
        rules.extend(cd_rules());
        rules.extend(curl_rules());
        rules.extend(fd_rules());
        rules.extend(find_rules());
        rules.extend(gh_rules());
        rules.extend(git_deny_rules());
        rules.extend(git_allow_rules());
        rules.extend(git_c_rules());
        rules.extend(git_worktree_rules());
        rules.extend(journalctl_rules());
        rules.extend(cd_git_rules());
        rules.extend(chained_push_rules());
        rules.extend(python_rules());
        rules.extend(sed_rules());
        rules.extend(sops_rules());
        rules.extend(read_only_rules());
        rules
    }
}

#[cfg(test)]
impl BashRuleProvider {
    /// Create a provider with specific rules for testing.
    pub(crate) fn new(rules: Vec<BashRule>) -> Self {
        Self { rules }
    }
}

impl FromServices for BashRuleProvider {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self { rules: Self::all() })
    }
}
