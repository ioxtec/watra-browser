use gpui::{App, Window};
use raw_window_handle::RawWindowHandle;
use watra_engine_api::BrowserEngine;

#[derive(Clone, Copy)]
pub(crate) struct RendererLayout {
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) sidebar_open: bool,
    pub(crate) sidebar_width: f32,
}

/// Native surface lifecycle boundary between shared UI and rendering backends.
pub(crate) trait RendererHost {
    fn initialize(
        &mut self,
        parent: RawWindowHandle,
        layout: RendererLayout,
        tab_id: u64,
        initial_url: &str,
    ) -> Option<Box<dyn BrowserEngine>>;
    fn create_tab(
        &mut self,
        parent: RawWindowHandle,
        layout: RendererLayout,
        tab_id: u64,
        initial_url: &str,
    ) -> Option<Box<dyn BrowserEngine>>;
    fn activate_tab(&mut self, tab_id: u64);
    fn close_tab(&mut self, tab_id: u64);
    fn sync_layout(&mut self, parent: RawWindowHandle, layout: RendererLayout);
    fn focus_address(&self);
    fn set_address(&self, url: &str);
    fn submit_address(&self);
    fn can_start_window_drag(&self, x: f32, y: f32, viewport_width: f32) -> bool;
    fn begin_window_drag(&self, window: &Window, cx: &mut App);
    fn detach_session(&mut self);
}
