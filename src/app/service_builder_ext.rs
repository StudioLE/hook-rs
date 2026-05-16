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
        self.with_logging(create_logger)
            .with_type::<BashEvaluator>()
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
            .with_instance(CliOptions::mock())
            .with_instance(HostContext::mock())
            .with_instance(Settings::mock())
    }
}

/// Create a [`Logger`] by resolving [`CliOptions`] from the service container.
fn create_logger(services: &ServiceProvider) -> Result<Logger, Report<ResolveError>> {
    let cli = services.get::<CliOptions>()?;
    let logger = LoggerBuilder::new()
        .with_level(cli.log_level.unwrap_or_default())
        .with_target("expansion", LogLevel::Info)
        .with_target("parse", LogLevel::Info)
        .build();
    Ok(logger)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end: container resolves handler and evaluates a command.
    #[test]
    fn service_builder_mock_bash_evaluation() {
        // Arrange
        let services = ServiceBuilder::mock().build().expect_init();
        let handler = services.expect::<BashHandler>();
        let input = BashInput {
            command: "git status".to_owned(),
        };

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }
}
