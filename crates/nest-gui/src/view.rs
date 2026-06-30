//! Root view contract for GUI applications.

use egui::Ui;
use nest_core::AppContext;
use nest_error::NestResult;

/// Root desktop view for a GUI application.
pub trait GuiView: Send + 'static {
    /// Renders the view into the given egui UI context.
    fn ui(&mut self, ui: &mut Ui, ctx: &AppContext) -> NestResult<()>;
}
