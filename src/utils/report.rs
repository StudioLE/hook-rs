//! Lightweight error report with diagnostic rendering.

use crate::prelude::*;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme};
use std::any::type_name;
use std::error::Error as StdError;

/// Error report that wraps a typed context with an optional source chain.
pub struct Report<T> {
    context: T,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl<T: StdError + Send + Sync + 'static> Report<T> {
    /// Create a report from the given error context with no source.
    pub fn new(context: T) -> Self {
        Self {
            context,
            source: None,
        }
    }

    /// The typed context stored in this report.
    pub fn current_context(&self) -> &T {
        &self.context
    }

    /// Wrap this report as the source of a new context.
    #[allow(dead_code)]
    pub fn change_context<U: StdError + Send + Sync + 'static>(self, new_context: U) -> Report<U> {
        Report {
            context: new_context,
            source: Some(Box::new(self)),
        }
    }
}

impl<T: Display> Display for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.context, f)
    }
}

impl<T: StdError + Send + Sync + 'static> Debug for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let diagnostic: &dyn Diagnostic = self;
        let mut rendered = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
            .render_report(&mut rendered, diagnostic)
            .expect("should be able to render report");
        f.write_str(rendered.trim_end())
    }
}

impl<T: StdError + Send + Sync + 'static> StdError for Report<T> {
    #[expect(
        clippy::as_conversions,
        reason = "cast from boxed trait object to trait reference"
    )]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

impl<T: StdError + Send + Sync + 'static> Diagnostic for Report<T> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(short_code(&self.context)))
    }
}

/// Build a short diagnostic code from `type_name::<T>()` and the context's `Debug` output.
///
/// - Enum contexts: `crate::EnumName::Variant`
/// - Struct contexts: `crate::StructName`
fn short_code<T: Debug>(context: &T) -> String {
    let full = type_name::<T>();
    let segments: Vec<&str> = full.split("::").collect();
    let crate_name = segments.first().unwrap_or(&full);
    let type_segment = segments.last().unwrap_or(&full);
    let debug = format!("{context:?}");
    let first_word = debug.split([' ', '(', '{']).next().unwrap_or(&debug);
    let is_enum_variant = first_word != *type_segment;
    if is_enum_variant {
        format!("{crate_name}::{type_segment}::{first_word}")
    } else {
        format!("{crate_name}::{type_segment}")
    }
}

/// Convert fallible results into [`Report`] by changing the error context.
pub trait ResultExt<T> {
    /// Wrap the error in a [`Report`] with the given context.
    fn change_context<C: StdError + Send + Sync + 'static>(
        self,
        context: C,
    ) -> Result<T, Report<C>>;
}

impl<T, E: StdError + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn change_context<C: StdError + Send + Sync + 'static>(
        self,
        context: C,
    ) -> Result<T, Report<C>> {
        self.map_err(|error| Report {
            context,
            source: Some(Box::new(error)),
        })
    }
}
