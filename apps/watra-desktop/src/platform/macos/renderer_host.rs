// CEF child views are hosted and resized through the AppKit FFI.
#![allow(unsafe_code)]

use crate::{
    browser_session::NavigationCore,
    browser_shell::effective_sidebar_width,
    diagnostics,
    platform::macos::native_address::NativeAddress,
    renderer_host::{RendererHost, RendererLayout},
};
use ::cef::{Browser, ImplBrowser, ImplBrowserHost};
use cocoa::{
    appkit::{NSColor, NSEvent, NSView, NSViewHeightSizable, NSViewWidthSizable, NSWindow},
    base::id,
    foundation::{NSPoint, NSSize},
};
use gpui::{App, Window};
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use watra_engine_api::BrowserEngine;
use watra_engine_cef::{CefEngine, LiveCefBrowser, create_client};

struct BrowserTab {
    id: u64,
    browser: Browser,
    frame: Option<(f64, f64, f64)>,
}

pub(crate) struct CefRendererHost {
    tabs: Vec<BrowserTab>,
    active_tab: Option<u64>,
    native_address: Option<NativeAddress>,
    runtime_browsers: Rc<RefCell<Vec<Browser>>>,
    browsers_created: Rc<Cell<usize>>,
    browsers_closed: Rc<Cell<usize>>,
    navigation_core: NavigationCore,
    address_editing: Rc<Cell<bool>>,
}

impl CefRendererHost {
    pub(crate) fn new(
        runtime_browsers: Rc<RefCell<Vec<Browser>>>,
        browsers_created: Rc<Cell<usize>>,
        browsers_closed: Rc<Cell<usize>>,
        navigation_core: NavigationCore,
        address_editing: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            native_address: None,
            runtime_browsers,
            browsers_created,
            browsers_closed,
            navigation_core,
            address_editing,
        }
    }

    fn create_browser(
        &mut self,
        parent: &AppKitWindowHandle,
        layout: RendererLayout,
        tab_id: u64,
        initial_url: &str,
    ) -> Option<Box<dyn BrowserEngine>> {
        let native_window = unsafe { (parent.ns_view.as_ptr() as id).window() };
        if !native_window.is_null() {
            unsafe {
                let background = NSColor::colorWithSRGBRed_green_blue_alpha_(
                    cocoa::base::nil,
                    32. / 255.,
                    35. / 255.,
                    40. / 255.,
                    1.,
                );
                native_window.setBackgroundColor_(background);
            }
        }

        let content_left = if layout.sidebar_open {
            layout.sidebar_width + 1.
        } else {
            0.
        };
        let content_top = 70.;
        let info = cef::WindowInfo {
            runtime_style: cef::RuntimeStyle::ALLOY,
            ..Default::default()
        }
        .set_as_child(
            parent.ns_view.as_ptr().cast(),
            &cef::Rect {
                x: content_left as i32 + 8,
                y: 8,
                width: (layout.viewport_width - content_left - 16.).max(1.) as i32,
                height: (layout.viewport_height - content_top).max(1.) as i32,
            },
        );
        let mut client = create_client(self.browsers_closed.clone());
        let browser = cef::browser_host_create_browser_sync(
            Some(&info),
            Some(&mut client),
            Some(&cef::CefString::from(initial_url)),
            Some(&cef::BrowserSettings::default()),
            None,
            None,
        )?;

        self.browsers_created.set(self.browsers_created.get() + 1);
        self.runtime_browsers.borrow_mut().push(browser.clone());
        self.tabs.push(BrowserTab {
            id: tab_id,
            browser: browser.clone(),
            frame: None,
        });
        self.activate_tab(tab_id);
        if self.native_address.is_none() {
            self.native_address = Some(unsafe {
                NativeAddress::new(
                    parent.ns_view.as_ptr(),
                    self.navigation_core.clone(),
                    self.address_editing.clone(),
                )
            });
        }
        eprintln!("CEF browser created for tab {tab_id}");
        diagnostics::event("cef.browser.created");
        Some(Box::new(CefEngine::new(LiveCefBrowser::new(browser))))
    }

    fn set_tab_visible(tab: &BrowserTab, visible: bool) {
        if let Some(host) = tab.browser.host() {
            let view = host.window_handle() as id;
            if !view.is_null() {
                let native_view = unsafe { &*view.cast::<objc2_app_kit::NSView>() };
                native_view.setHidden(!visible);
            }
        }
    }
}

impl RendererHost for CefRendererHost {
    fn initialize(
        &mut self,
        parent: RawWindowHandle,
        layout: RendererLayout,
        tab_id: u64,
        initial_url: &str,
    ) -> Option<Box<dyn BrowserEngine>> {
        let RawWindowHandle::AppKit(parent) = parent else {
            return None;
        };
        self.create_browser(&parent, layout, tab_id, initial_url)
    }

    fn create_tab(
        &mut self,
        parent: RawWindowHandle,
        layout: RendererLayout,
        tab_id: u64,
        initial_url: &str,
    ) -> Option<Box<dyn BrowserEngine>> {
        let RawWindowHandle::AppKit(parent) = parent else {
            return None;
        };
        self.create_browser(&parent, layout, tab_id, initial_url)
    }

    fn activate_tab(&mut self, tab_id: u64) {
        for tab in &self.tabs {
            Self::set_tab_visible(tab, tab.id == tab_id);
        }
        self.active_tab = Some(tab_id);
    }

    fn close_tab(&mut self, tab_id: u64) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        let tab = self.tabs.remove(index);
        if let Some(host) = tab.browser.host() {
            let view = unsafe {
                objc2::rc::Retained::<objc2_app_kit::NSView>::retain(host.window_handle().cast())
            };
            host.close_browser(1);
            if let Some(view) = view {
                view.removeFromSuperview();
            }
        }
        self.runtime_browsers
            .borrow_mut()
            .retain(|browser| browser.identifier() != tab.browser.identifier());
        if self.active_tab == Some(tab_id) {
            self.active_tab = None;
        }
    }

    fn sync_layout(&mut self, _: RawWindowHandle, layout: RendererLayout) {
        if let Some(address) = &self.native_address {
            address.update_frame();
        }
        for tab in &self.tabs {
            Self::set_tab_visible(tab, Some(tab.id) == self.active_tab);
        }
        let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|tab| Some(tab.id) == self.active_tab)
        else {
            return;
        };
        let Some(host) = tab.browser.host() else {
            return;
        };
        let view = host.window_handle() as id;
        if view.is_null() {
            return;
        }
        let native_view = unsafe { &*view.cast::<objc2_app_kit::NSView>() };
        let Some(parent) = (unsafe { native_view.superview() }) else {
            return;
        };
        let bounds = parent.bounds();
        let sidebar_width = effective_sidebar_width(
            layout.sidebar_open,
            layout.sidebar_width,
            bounds.size.width as f32,
        );
        let content_left = if layout.sidebar_open {
            sidebar_width + 1.
        } else {
            0.
        };
        let content_top = 70.;
        let frame = (
            (content_left + 8.) as f64,
            (bounds.size.width - content_left as f64 - 16.).max(1.),
            (bounds.size.height - content_top).max(1.),
        );
        if tab.frame == Some(frame) {
            return;
        }
        unsafe {
            view.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
            view.setFrameOrigin(NSPoint::new(frame.0, 8.));
            view.setFrameSize(NSSize::new(frame.1, frame.2));
            native_view.setWantsLayer(true);
            if let Some(layer) = native_view.layer() {
                let _: () = objc2::msg_send![&*layer, setCornerRadius: 8.0_f64];
                layer.setMasksToBounds(true);
            }
        }
        tab.frame = Some(frame);
    }

    fn focus_address(&self) {
        if let Some(address) = &self.native_address {
            address.update_frame();
            address.focus();
        } else {
            diagnostics::event("address.focus.renderer_unavailable");
        }
    }

    fn set_address(&self, url: &str) {
        if let Some(address) = &self.native_address {
            address.set_url(url);
        }
    }

    fn submit_address(&self) {
        if let Some(address) = &self.native_address {
            address.submit();
        }
    }

    fn can_start_window_drag(&self, x: f32, y: f32, viewport_width: f32) -> bool {
        let address_right = viewport_width - 52.;
        x >= 100.
            && y >= 8.
            && x <= viewport_width - 8.
            && !((249. ..=address_right).contains(&x) && (13. ..=43.).contains(&y))
    }

    fn begin_window_drag(&self, window: &Window, cx: &mut App) {
        super::window_chrome::begin_drag(window, cx);
    }

    fn detach_session(&mut self) {
        self.native_address.take();
        for tab in &self.tabs {
            Self::set_tab_visible(tab, false);
        }
    }
}
