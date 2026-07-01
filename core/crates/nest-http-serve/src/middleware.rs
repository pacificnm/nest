//! Middleware hook chain.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::context::RequestContext;
use crate::response::HttpResult;
use crate::router::Handler;

/// Type-erased middleware layer.
pub type MiddlewareLayer = Arc<
    dyn Fn(RequestContext, Next) -> Pin<Box<dyn Future<Output = HttpResult> + Send>>
        + Send
        + Sync,
>;

/// Continuation for middleware chains.
pub struct Next {
    middleware: Arc<Vec<MiddlewareLayer>>,
    index: usize,
    handler: Handler,
}

impl Next {
    /// Invokes the next middleware or the route handler.
    pub async fn run(self, ctx: RequestContext) -> HttpResult {
        if self.index < self.middleware.len() {
            let layer = self.middleware[self.index].clone();
            let next = Self {
                middleware: self.middleware,
                index: self.index + 1,
                handler: self.handler,
            };
            layer(ctx, next).await
        } else {
            (self.handler)(ctx).await
        }
    }
}

/// Converts middleware functions into middleware layers.
pub trait IntoMiddleware {
    /// Converts into a type-erased middleware layer.
    fn into_middleware(self) -> MiddlewareLayer;
}

impl<F, Fut> IntoMiddleware for F
where
    F: Fn(RequestContext, Next) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = HttpResult> + Send + 'static,
{
    fn into_middleware(self) -> MiddlewareLayer {
        Arc::new(move |ctx, next| Box::pin(self(ctx, next)))
    }
}

/// Wraps a handler with middleware layers.
pub(crate) fn wrap_with_middleware(handler: Handler, middleware: &[MiddlewareLayer]) -> Handler {
    if middleware.is_empty() {
        return handler;
    }

    let stack = Arc::new(middleware.to_vec());
    Arc::new(move |ctx| {
        let next = Next {
            middleware: stack.clone(),
            index: 0,
            handler: handler.clone(),
        };
        Box::pin(next.run(ctx))
    })
}
