// Native window hit testing and cursor updates use AppKit FFI.
#![allow(unsafe_code)]

use super::{BrowserShell, maximum_sidebar_width};
use crate::diagnostics;
use cocoa::{
    appkit::{NSCursor, NSEvent, NSWindow},
    base::id,
};
use gpui::{App, Timer, WindowHandle};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::Cell, rc::Rc, time::Duration};

pub(crate) fn spawn(
    cx: &mut App,
    browser_window: WindowHandle<BrowserShell>,
    quitting: Rc<Cell<bool>>,
) {
    // Track the native mouse while dragging across the CEF child view.
    // GPUI alone cannot receive every movement once the pointer enters CEF.
    cx.spawn(async move |cx| {
        loop {
            Timer::after(Duration::from_millis(16)).await;
            if quitting.get() {
                break;
            }
            let focus_requested = Cell::new(false);
            let update_result = browser_window.update(cx, |shell, window, cx| {
                let focus_address = shell.address_focus_pending;
                shell.address_focus_pending = false;
                focus_requested.set(focus_address);
                if focus_address {
                    diagnostics::event("address.focus.deferred");
                }
                shell.initialize_renderer(window);
                shell.sync_renderer_layout(window);
                if shell.rendered_address_editing != shell.address_editing.get() {
                    shell.rendered_address_editing = shell.address_editing.get();
                    shell.initialize_renderer(window);
                    shell.sync_renderer_layout(window);
                    cx.notify();
                }
                if let Some(raw_url) = shell.current_page_url() {
                    let is_new_tab = raw_url.starts_with("data:") || raw_url == "about:blank";
                    let display_url = if is_new_tab { "New Tab" } else { &raw_url };
                    let title = if is_new_tab {
                        "New Tab".to_string()
                    } else {
                        raw_url
                            .split_once("://")
                            .map(|(_, remainder)| remainder.split('/').next().unwrap_or(remainder))
                            .unwrap_or(&raw_url)
                            .to_string()
                    };
                    let title_changed = shell.update_active_tab_title(title);
                    if shell.address_url != display_url {
                        shell.address_url = display_url.to_string();
                        shell.set_renderer_address(display_url);
                        cx.notify();
                    } else if title_changed {
                        cx.notify();
                    }
                }
                let Ok(handle) = HasWindowHandle::window_handle(window) else {
                    return;
                };
                let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
                    return;
                };
                unsafe {
                    let native_window = (handle.ns_view.as_ptr() as id).window();
                    if native_window.is_null() {
                        return;
                    }
                    let mtm = objc2::MainThreadMarker::new()
                        .expect("Sidebar tracking requires the main thread");
                    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
                    if !app.isActive() || !native_window.isKeyWindow() {
                        shell.resizing_sidebar = false;
                        shell.address_focus_pending = false;
                        focus_requested.set(false);
                        let had_address_editor = shell.address_editing.get();
                        if had_address_editor {
                            diagnostics::event("address.editor.closed.window_inactive");
                            shell.address_editing.set(false);
                            shell.rendered_address_editing = false;
                            shell.sync_renderer_layout(window);
                        }
                        let had_hover = shell.address_hovered || shell.divider_hovered;
                        if shell.address_hovered {
                            shell.address_hovered = false;
                            shell.address_hover_generation += 1;
                        }
                        if had_hover || had_address_editor {
                            shell.divider_hovered = false;
                            cx.notify();
                        }
                        return;
                    }
                    let mouse = native_window.mouseLocationOutsideOfEventStream();
                    let window_size = window.viewport_size();
                    let viewport_width = f32::from(window_size.width) as f64;
                    let viewport_height = f32::from(window_size.height) as f64;
                    let in_address_bar = mouse.x >= 249.
                        && mouse.x <= viewport_width - 52.
                        && mouse.y >= viewport_height - 43.
                        && mouse.y <= viewport_height - 13.;
                    let in_address_editor = in_address_bar
                        || (mouse.x >= 242.
                            && mouse.x <= viewport_width - 56.
                            && mouse.y >= viewport_height - 203.
                            && mouse.y <= viewport_height - 13.);
                    let left_pressed = NSEvent::pressedMouseButtons(cocoa::base::nil) & 1 != 0;
                    if shell.address_editing.get() && left_pressed && !in_address_editor {
                        diagnostics::event("address.editor.closed.outside_click");
                        shell.address_editing.set(false);
                        shell.rendered_address_editing = false;
                        shell.sync_renderer_layout(window);
                        cx.notify();
                    }
                    let in_address_bar = !shell.address_editing.get() && in_address_bar;
                    if shell.address_hovered != in_address_bar {
                        shell.address_hovered = in_address_bar;
                        shell.address_hover_generation += 1;
                        cx.notify();
                    }
                    if left_pressed && in_address_bar {
                        diagnostics::event("address.click.detected_by_tracking");
                        shell.address_editing.set(true);
                        shell.address_focus_pending = true;
                        cx.notify();
                    }
                    let inside_height = mouse.y >= 0.
                        && mouse.y < (f32::from(window.viewport_size().height) - 54.) as f64;
                    let was_resize_cursor = shell.divider_hovered || shell.resizing_sidebar;
                    let hovered = shell.sidebar_open
                        && inside_height
                        && (mouse.x - shell.sidebar_width as f64).abs() <= 6.;
                    if shell.divider_hovered != hovered {
                        shell.divider_hovered = hovered;
                        cx.notify();
                    }
                    if hovered || shell.resizing_sidebar {
                        let cursor = NSCursor::resize_left_right_cursor(cocoa::base::nil);
                        let _: () =
                            objc2::msg_send![cursor.cast::<objc2::runtime::AnyObject>(), set];
                    } else if was_resize_cursor {
                        let cursor = NSCursor::arrow_cursor(cocoa::base::nil);
                        let _: () =
                            objc2::msg_send![cursor.cast::<objc2::runtime::AnyObject>(), set];
                    }
                    // A twelve-point hit area surrounds the one-point separator.
                    if !shell.resizing_sidebar
                        && shell.sidebar_open
                        && left_pressed
                        && inside_height
                        && (mouse.x - shell.sidebar_width as f64).abs() <= 6.
                    {
                        shell.resizing_sidebar = true;
                        shell.resize_offset = mouse.x as f32 - shell.sidebar_width;
                    }
                    if shell.resizing_sidebar {
                        if left_pressed {
                            let maximum =
                                maximum_sidebar_width(f32::from(window.viewport_size().width));
                            shell.sidebar_width =
                                (mouse.x as f32 - shell.resize_offset).clamp(180., maximum);
                        } else {
                            shell.resizing_sidebar = false;
                        }
                        cx.notify();
                    }
                }
            });
            match update_result {
                Ok(()) if focus_requested.get() => {
                    let window_handle = browser_window;
                    let _ = cx.update(|app| {
                        app.defer(move |app| {
                            let _ = window_handle.update(app, |shell, _, _| {
                                shell.focus_renderer_address();
                            });
                        });
                    });
                }
                Ok(()) => {}
                Err(_) => break,
            }
        }
    })
    .detach();
}
