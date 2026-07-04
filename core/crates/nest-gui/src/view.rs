//! Root view contracts for GUI applications.

use egui::{Context, Ui};
use nest_core::AppContext;
use nest_error::NestResult;

/// Root desktop view for a GUI application (single central panel host).
pub trait GuiView: Send + 'static {
    /// Renders the view into the given egui UI context.
    fn ui(&mut self, ui: &mut Ui, ctx: &AppContext) -> NestResult<()>;
}

/// Full-window IDE-style view that owns panel layout on the root [`Context`].
pub trait WorkbenchView: Send + 'static {
    /// Renders the workbench directly on the root egui context.
    fn ui(&mut self, ctx: &Context, app_ctx: &AppContext) -> NestResult<()>;
}

/// Root view kind registered with [`crate::GuiApp`].
pub(crate) enum RootView {
    /// Standard central-panel view.
    Standard(Box<dyn GuiView>),
    /// Full-window workbench view.
    Workbench(Box<dyn WorkbenchView>),
}
