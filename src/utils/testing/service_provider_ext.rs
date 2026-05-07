//! Extension trait for service resolution in tests.

use crate::prelude::*;
use std::any::type_name;

/// Extension trait for [`ServiceProvider`] resolution and test construction.
pub(crate) trait ServiceProviderExt {
    /// Resolve a type or panic with a rendered error.
    fn expect<T: Send + Sync + 'static>(&self) -> Arc<T>;

    /// Build a [`ServiceProvider`] with mock instances.
    fn mock() -> Self;
}

impl ServiceProviderExt for ServiceProvider {
    #[expect(clippy::panic, reason = "expect fn")]
    fn expect<T: Send + Sync + 'static>(&self) -> Arc<T> {
        match self.get::<T>() {
            Ok(a) => a,
            Err(e) => {
                error!("{}", e.render());
                panic!("Failed to resolve type: {}", type_name::<T>());
            }
        }
    }

    #[cfg(test)]
    fn mock() -> Self {
        ServiceBuilder::mock().build()
    }
}
