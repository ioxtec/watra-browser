mod new_tab;
mod sidebar;
#[cfg(target_os = "macos")]
#[path = "platform/macos/sidebar_tracking.rs"]
mod sidebar_tracking;
mod toolbar;

#[cfg(target_os = "macos")]
pub(crate) use sidebar_tracking::spawn as start_pointer_tracking;

use crate::{
    browser_session::NavigationCore,
    renderer_host::{RendererHost, RendererLayout},
};
use gpui::{prelude::*, *};
use raw_window_handle::HasWindowHandle;
use std::cell::Cell;
use watra_core::BrowserCore;
use watra_engine_api::BrowserEngine;

type BrowserSession = BrowserCore<Box<dyn BrowserEngine>>;

pub(crate) struct BrowserTab {
    pub(crate) id: u64,
    pub(crate) title: String,
    session: Option<BrowserSession>,
}

pub(crate) struct BrowserShell {
    renderer: Box<dyn RendererHost>,
    sidebar_open: bool,
    sidebar_width: f32,
    resizing_sidebar: bool,
    resize_offset: f32,
    divider_hovered: bool,
    navigation_core: NavigationCore,
    address_editing: std::rc::Rc<Cell<bool>>,
    rendered_address_editing: bool,
    address_hovered: bool,
    address_hover_generation: usize,
    address_focus_pending: bool,
    address_url: String,
    pub(crate) tabs: Vec<BrowserTab>,
    pub(crate) active_tab: u64,
    next_tab_id: u64,
}

impl BrowserShell {
    pub(crate) fn new(
        renderer: Box<dyn RendererHost>,
        navigation_core: NavigationCore,
        address_editing: std::rc::Rc<Cell<bool>>,
    ) -> Self {
        Self {
            renderer,
            sidebar_open: true,
            sidebar_width: 240.,
            resizing_sidebar: false,
            resize_offset: 0.,
            divider_hovered: false,
            navigation_core,
            address_editing,
            rendered_address_editing: false,
            address_hovered: false,
            address_hover_generation: 0,
            address_focus_pending: false,
            address_url: "Search or enter address".to_string(),
            tabs: vec![BrowserTab {
                id: 1,
                title: "New Tab".to_string(),
                session: None,
            }],
            active_tab: 1,
            next_tab_id: 2,
        }
    }

    pub(crate) fn focus_address(&mut self) {
        self.address_editing.set(true);
        self.rendered_address_editing = true;
        self.address_focus_pending = true;
    }

    pub(crate) fn take_browser(&mut self) {
        self.navigation_core.borrow_mut().take();
        self.renderer.detach_session();
    }

    pub(crate) fn initialize_renderer(&mut self, window: &Window) {
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        let layout = self.renderer_layout(window);
        if self.navigation_core.borrow().is_none()
            && let Some(engine) =
                self.renderer
                    .initialize(handle.as_raw(), layout, self.active_tab, &new_tab::url())
        {
            *self.navigation_core.borrow_mut() = Some(watra_core::BrowserCore::new(engine));
        }
        self.renderer.sync_layout(handle.as_raw(), layout);
    }

    pub(crate) fn sync_renderer_layout(&mut self, window: &Window) {
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        self.renderer
            .sync_layout(handle.as_raw(), self.renderer_layout(window));
    }

    fn renderer_layout(&self, window: &Window) -> RendererLayout {
        RendererLayout {
            viewport_width: f32::from(window.viewport_size().width),
            viewport_height: f32::from(window.viewport_size().height),
            sidebar_open: self.sidebar_open,
            sidebar_width: effective_sidebar_width(
                self.sidebar_open,
                self.sidebar_width,
                f32::from(window.viewport_size().width),
            ),
        }
    }

    pub(crate) fn focus_renderer_address(&self) {
        self.renderer.focus_address();
    }

    pub(crate) fn set_renderer_address(&self, url: &str) {
        self.renderer.set_address(url);
    }

    pub(crate) fn submit_renderer_address(&self) {
        self.renderer.submit_address();
    }

    pub(crate) fn begin_window_drag(&self, window: &Window, cx: &mut App) {
        self.renderer.begin_window_drag(window, cx);
    }

    pub(crate) fn can_start_window_drag(&self, x: f32, y: f32, viewport_width: f32) -> bool {
        self.renderer.can_start_window_drag(x, y, viewport_width)
    }

    pub(crate) fn go_back(&mut self) {
        self.run_navigation_command(|core| core.go_back());
    }

    pub(crate) fn go_forward(&mut self) {
        self.run_navigation_command(|core| core.go_forward());
    }

    pub(crate) fn reload(&mut self) {
        self.run_navigation_command(|core| core.reload());
    }

    pub(crate) fn create_tab(&mut self, window: &Window) {
        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        let Some(current_index) = self.tabs.iter().position(|tab| tab.id == self.active_tab) else {
            return;
        };
        self.tabs[current_index].session = self.navigation_core.borrow_mut().take();
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let layout = self.renderer_layout(window);
        let Some(engine) = self
            .renderer
            .create_tab(handle.as_raw(), layout, id, &new_tab::url())
        else {
            self.navigation_core.borrow_mut().replace(
                self.tabs[current_index]
                    .session
                    .take()
                    .unwrap_or_else(|| unreachable!("active tab session was just saved")),
            );
            return;
        };
        self.renderer.activate_tab(id);
        self.tabs[current_index].title = self.tabs[current_index]
            .session
            .as_ref()
            .and_then(|session| session.snapshot().title)
            .unwrap_or_else(|| self.tabs[current_index].title.clone());
        self.tabs.push(BrowserTab {
            id,
            title: "New Tab".to_string(),
            session: Some(BrowserCore::new(engine)),
        });
        self.active_tab = id;
        *self.navigation_core.borrow_mut() = self
            .tabs
            .iter_mut()
            .find(|tab| tab.id == id)
            .and_then(|tab| tab.session.take());
        self.address_url = "Search or enter address".to_string();
        self.set_renderer_address("New Tab");
        self.focus_address();
    }

    pub(crate) fn activate_tab(&mut self, tab_id: u64) {
        if tab_id == self.active_tab {
            return;
        }
        let Some(next_index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        let Some(active) = self.tabs.iter_mut().find(|tab| tab.id == self.active_tab) else {
            return;
        };
        active.session = self.navigation_core.borrow_mut().take();
        let next = &mut self.tabs[next_index];
        *self.navigation_core.borrow_mut() = next.session.take();
        self.active_tab = tab_id;
        self.address_url = self.display_current_url();
        self.renderer.activate_tab(tab_id);
        self.set_renderer_address(&self.address_url);
        self.address_editing.set(false);
        self.rendered_address_editing = false;
    }

    pub(crate) fn close_tab(&mut self, tab_id: u64, window: &Window) {
        if self.tabs.len() == 1 {
            self.create_tab(window);
            if self.tabs.len() == 1 {
                return;
            }
        }
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        let was_active = tab_id == self.active_tab;
        if was_active {
            self.navigation_core.borrow_mut().take();
        }
        self.tabs[index].session.take();
        self.renderer.close_tab(tab_id);
        self.tabs.remove(index);
        if was_active {
            let next_index = index.min(self.tabs.len() - 1);
            let next = self.tabs[next_index].id;
            self.active_tab = next;
            let tab = &mut self.tabs[next_index];
            *self.navigation_core.borrow_mut() = tab.session.take();
            self.renderer.activate_tab(next);
            self.address_url = self.display_current_url();
            self.set_renderer_address(&self.address_url);
        }
    }

    fn display_current_url(&self) -> String {
        match self.current_page_url() {
            Some(url) if url.starts_with("data:") => "New Tab".to_string(),
            Some(url) => url,
            None => "Search or enter address".to_string(),
        }
    }

    pub(crate) fn update_active_tab_title(&mut self, title: String) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == self.active_tab)
            && !title.trim().is_empty()
            && tab.title != title
        {
            tab.title = title;
            return true;
        }
        false
    }

    fn run_navigation_command(
        &mut self,
        command: impl FnOnce(
            &mut watra_core::BrowserCore<Box<dyn watra_engine_api::BrowserEngine>>,
        ) -> Result<(), watra_engine_api::BrowserEngineError>,
    ) {
        if let Some(core) = self.navigation_core.borrow_mut().as_mut()
            && let Err(error) = command(core)
        {
            crate::diagnostics::event("browser.command.failed");
            eprintln!("Browser command failed: {error}");
        }
    }

    pub(crate) fn current_page_url(&self) -> Option<String> {
        self.navigation_core
            .borrow()
            .as_ref()
            .and_then(|core| core.snapshot().url)
    }
}

pub(super) fn maximum_sidebar_width(available_width: f32) -> f32 {
    (available_width - 320.).clamp(180., 360.)
}

pub(super) fn effective_sidebar_width(
    open: bool,
    requested_width: f32,
    available_width: f32,
) -> f32 {
    if open {
        requested_width.clamp(180., maximum_sidebar_width(available_width))
    } else {
        0.
    }
}

impl Render for BrowserShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Reserve a separate content region for the native CEF child view.
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x202328))
            // Use an explicit system-installed family while validating text rendering.
            .text_size(px(15.))
            .text_color(rgb(0xe6e7e9))
            .child(toolbar::render(self, cx))
            .child(sidebar::render(self, window, cx))
    }
}
