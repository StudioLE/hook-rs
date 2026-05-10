//! Application entrypoint that bootstraps services and runs a subcommand.

use crate::prelude::*;

/// Application entrypoint that bootstraps services and runs a subcommand.
pub struct Cli {
    services: ServiceProvider,
}

impl Cli {
    /// Create a new [`Cli`] with the default service registrations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: ServiceBuilder::new().with_app_services().build(),
        }
    }

    /// Run the CLI to completion.
    pub fn run(&self) {
        let cli_options = self
            .services
            .get::<CliOptions>()
            .expect("should be able to resolve CliOptions");
        let _logger = init_logger(cli_options.log_level);
        let handler = self
            .services
            .get::<SubcommandHandler>()
            .expect("should be able to resolve SubcommandHandler");
        let outcome = handler.run();
        if let Some(outcome) = outcome {
            info!("{outcome}");
            outcome.print_hook_output();
        } else {
            info!("No outcome");
        }
    }
}
