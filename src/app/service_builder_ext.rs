//! Extension trait for configuring a [`ServiceBuilder`] with application services.

use crate::prelude::*;

/// Extension trait for configuring a [`ServiceBuilder`] with application services.
pub(crate) trait ServiceBuilderExt {
    /// Register all application types for dependency resolution.
    fn with_app_services(self) -> ServiceBuilder;

    /// Create a [`ServiceBuilder`] pre-loaded with mock instances and all types registered.
    #[cfg(test)]
    fn mock() -> ServiceBuilder;
}

impl ServiceBuilderExt for ServiceBuilder {
    fn with_app_services(self) -> Self {
        self.with_type::<BashEvaluator>()
            .with_type::<BashHandler>()
            .with_type::<BashRuleProvider>()
            .with_type::<CliOptions>()
            .with_type::<GlobHandler>()
            .with_type::<GrepHandler>()
            .with_type::<HostContext>()
            .with_type::<PathRuleFactory>()
            .with_type::<ReadHandler>()
            .with_type::<Settings>()
            .with_type::<SubcommandHandler>()
    }

    #[cfg(test)]
    fn mock() -> ServiceBuilder {
        ServiceBuilder::new()
            .with_app_services()
            .with_instance(HostContext::mock())
            .with_instance(Settings::mock())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end: container resolves handler and evaluates a command.
    #[test]
    fn service_builder_mock_bash_evaluation() {
        // Arrange
        let services = ServiceBuilder::mock().build();
        let handler = services.get::<BashHandler>().expect("should resolve");
        let input = BashInput {
            command: "git status".to_owned(),
        };

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }
}
