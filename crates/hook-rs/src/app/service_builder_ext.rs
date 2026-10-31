//! Extension trait for configuring a [`ServiceBuilder`] with application services.

use crate::prelude::*;

/// Extension trait for configuring a [`ServiceBuilder`] with application services.
pub(crate) trait ServiceBuilderExt {
    /// Register all application types for dependency resolution.
    fn with_app_services(self) -> Self;

    /// Register all application services with fixtures for host state.
    #[cfg(test)]
    fn with_test_services(self) -> Self;
}

impl ServiceBuilderExt for ServiceBuilder {
    fn with_app_services(self) -> Self {
        self.with_logging(create_logger)
            .with_trait::<dyn ReadStdin, StdinReader>()
            .with_type::<BashEvaluator>()
            .with_type::<BashHandler>()
            .with_type::<BashRuleProvider>()
            .with_type::<CliArgs>()
            .with_type::<GlobHandler>()
            .with_type::<GrepHandler>()
            .with_type::<HostContext>()
            .with_type::<PathRuleFactory>()
            .with_type::<ReadHandler>()
            .with_type::<Settings>()
            .with_type::<SubcommandHandler>()
    }

    #[cfg(test)]
    fn with_test_services(self) -> Self {
        self.with_app_services()
            .with_trait::<dyn ReadStdin, ReadStdinStub>()
            .with_instance(CliArgs::fixture())
            .with_instance(HostContext::fixture())
            .with_instance(Settings::fixture())
    }
}

/// Create a [`Logger`] by resolving [`CliArgs`] from the service container.
fn create_logger(services: &ServiceProvider) -> Result<Logger, Report<ResolveError>> {
    let cli = services.get::<CliArgs>()?;
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
    fn service_builder_with_test_services() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect_init();
        let handler = services.expect::<BashHandler>();
        let input = HookInput::new(BashInput {
            command: "git status".to_owned(),
        });

        // Act
        let outcome = handler.run(input);

        // Assert
        assert_eq!(outcome.expect("should match").decision, Decision::Allow);
    }
}
