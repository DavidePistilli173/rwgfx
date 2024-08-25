use crate::asset;
use crate::renderer::FrameCtx;

/// Trait for drawable objects.
pub trait Drawable {
    fn draw<'pass, 'ctx>(
        &'pass self,
        ctx: &'ctx FrameCtx<'pass>,
        asset_manager: &'pass asset::Manager,
    ) where
        'pass: 'ctx;
}
