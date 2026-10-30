use cef::{ImplBrowser, ImplBrowserHost};
use cocoa::{
    appkit::{NSCursor, NSEvent, NSView, NSWindow},
    base::id,
    foundation::{NSPoint, NSSize},
};
use gpui::{prelude::*, *};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

#[derive(Clone, Copy)]
enum ToolbarIcon {
    Back,
    Forward,
    Reload,
    Menu,
}

fn toolbar_icon(icon: ToolbarIcon) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let mut path = PathBuilder::stroke(px(1.2));
            let p = |x: f32, y: f32| bounds.origin + point(px(x), px(y));
            match icon {
                ToolbarIcon::Back | ToolbarIcon::Forward => {
                    let x = |v| {
                        if matches!(icon, ToolbarIcon::Forward) {
                            16. - v
                        } else {
                            v
                        }
                    };
                    path.move_to(p(x(9.5), 3.));
                    path.line_to(p(x(4.5), 8.));
                    path.line_to(p(x(9.5), 13.));
                    path.move_to(p(x(4.5), 8.));
                    path.line_to(p(x(14.), 8.));
                }
                ToolbarIcon::Reload => {
                    for i in 0..=28 {
                        let angle = -0.65 + i as f32 / 28. * 5.3;
                        let point = p(8. + 5.5 * angle.cos(), 8. + 5.5 * angle.sin());
                        if i == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                    path.move_to(p(12.4, 1.5));
                    path.line_to(p(12.4, 5.5));
                    path.line_to(p(8.4, 5.5));
                }
                ToolbarIcon::Menu => {
                    for x in [3., 8., 13.] {
                        path.move_to(p(x, 7.));
                        path.line_to(p(x, 9.));
                    }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(0xb8bec8));
            }
        },
    )
    .size(px(16.))
}

struct SettingsPreview;
impl Render for SettingsPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .bg(rgb(0x202328))
            .text_color(rgb(0xe6e7e9))
            .font_family("Helvetica")
            .text_size(px(15.))
            .child("Watra settings — coming soon")
    }
}

struct BrowserShell {
    browser: Option<cef::Browser>,
    sidebar_open: bool,
    sidebar_width: f32,
    resizing_sidebar: bool,
    resize_offset: f32,
    native_frame: Option<(f64, f64, f64)>,
    divider_hovered: bool,
}

impl Render for BrowserShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        let maximum_width = (f32::from(viewport.width) - 320.).clamp(180., 360.);
        let sidebar_width = if self.sidebar_open {
            self.sidebar_width.clamp(180., maximum_width)
        } else {
            0.
        };
        let content_left = if self.sidebar_open {
            sidebar_width + 1.
        } else {
            0.
        };
        if self.browser.is_none()
            && let Ok(handle) = HasWindowHandle::window_handle(window)
            && let RawWindowHandle::AppKit(handle) = handle.as_raw()
        {
            let info = cef::WindowInfo {
                runtime_style: cef::RuntimeStyle::ALLOY,
                ..Default::default()
            }
            .set_as_child(
                handle.ns_view.as_ptr().cast(),
                &cef::Rect {
                    // Cocoa child-view coordinates start at the bottom left.
                    x: content_left as i32,
                    y: 0,
                    width: (f32::from(viewport.width) - content_left).max(1.) as i32,
                    height: (f32::from(viewport.height) - 54.).max(1.) as i32,
                },
            );
            self.browser = cef::browser_host_create_browser_sync(
                Some(&info),
                None,
                Some(&cef::CefString::from(
                    "data:text/html,<h1>Watra CEF embedded</h1><input placeholder='Keyboard test'>",
                )),
                Some(&cef::BrowserSettings::default()),
                None,
                None,
            );
            eprintln!("CEF browser created: {}", self.browser.is_some());
        }
        if let Some(host) = self.browser.as_ref().and_then(|browser| browser.host()) {
            let view = host.window_handle() as id;
            if !view.is_null() {
                // Keep Chromium visible and resize only when geometry changes.
                let frame = (
                    content_left as f64,
                    (f32::from(viewport.width) - content_left).max(1.) as f64,
                    (f32::from(viewport.height) - 54.).max(1.) as f64,
                );
                if self.native_frame != Some(frame) {
                    unsafe {
                        view.setFrameOrigin(NSPoint::new(frame.0, 0.));
                        view.setFrameSize(NSSize::new(frame.1, frame.2));
                    }
                    self.native_frame = Some(frame);
                }
            }
        }
        // Reserve a separate content region for the native CEF child view.
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x17191c))
            // Use an explicit system-installed family while validating text rendering.
            .font_family("Helvetica")
            .text_size(px(15.))
            .text_color(rgb(0xe6e7e9))
            .child(
                div()
                    // Native buttons and toolbar share a center at 27 points.
                    .h(px(54.))
                    .pl(px(100.))
                    .pr_4()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("toggle-sidebar")
                            .w(px(28.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x363b42)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.sidebar_open = !this.sidebar_open;
                                    cx.notify();
                                }),
                            )
                            .child(
                                div()
                                    .w(px(15.))
                                    .h(px(12.))
                                    .border_1()
                                    .border_color(rgb(0xb8bec8))
                                    .rounded(px(2.))
                                    .child(
                                        div()
                                            .w(px(5.))
                                            .h_full()
                                            .border_r_1()
                                            .border_color(rgb(0xb8bec8)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("back")
                            .size(px(28.))
                            .text_size(px(26.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    if let Some(browser) = &this.browser {
                                        browser.go_back();
                                    }
                                }),
                            )
                            .child(toolbar_icon(ToolbarIcon::Back)),
                    )
                    .child(
                        div()
                            .id("forward")
                            .size(px(28.))
                            .text_size(px(26.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    if let Some(browser) = &this.browser {
                                        browser.go_forward();
                                    }
                                }),
                            )
                            .child(toolbar_icon(ToolbarIcon::Forward)),
                    )
                    .child(
                        div()
                            .id("reload")
                            .size(px(28.))
                            .text_size(px(22.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, _| {
                                    if let Some(browser) = &this.browser {
                                        browser.reload();
                                    }
                                }),
                            )
                            .child(toolbar_icon(ToolbarIcon::Reload)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .rounded_md()
                            .bg(rgb(0x292d32))
                            .px_3()
                            .h(px(30.))
                            .flex()
                            .items_center()
                            .window_control_area(WindowControlArea::Drag)
                            .child("Watra · Local test page"),
                    )
                    .child(
                        div()
                            .id("settings-menu")
                            .size(px(28.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .rounded_md()
                            .hover(|style| style.bg(rgb(0x363b42)))
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                let _ = cx.open_window(
                                    WindowOptions {
                                        titlebar: Some(TitlebarOptions {
                                            title: Some("Watra settings".into()),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    },
                                    |_, cx| cx.new(|_| SettingsPreview),
                                );
                            })
                            .child(toolbar_icon(ToolbarIcon::Menu)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .child(
                        div()
                            .w(px(sidebar_width))
                            .h_full()
                            .overflow_hidden()
                            .bg(rgb(0x202328))
                            .when(self.sidebar_open, |sidebar| {
                                sidebar
                                    .p_3()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(div().text_lg().child("Watra"))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x9da5b0))
                                            .child("WORKSPACE"),
                                    )
                                    .child(
                                        div()
                                            .p_2()
                                            .rounded_md()
                                            .bg(rgb(0x343a43))
                                            .child("Local test page"),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x9da5b0))
                                            .child("Tab management coming next"),
                                    )
                            }),
                    )
                    .when(self.sidebar_open, |row| {
                        row.child(
                            div()
                                .id("sidebar-divider")
                                .w(px(1.))
                                .h_full()
                                .relative()
                                .child(
                                    div()
                                        .absolute()
                                        .h_full()
                                        .left(px(if self.divider_hovered { -1. } else { 0. }))
                                        .w(px(if self.divider_hovered { 3. } else { 1. }))
                                        .bg(rgb(if self.divider_hovered {
                                            0x596474
                                        } else {
                                            0x30353d
                                        })),
                                )
                                .cursor(CursorStyle::ResizeLeftRight)
                                .hover(|style| style.bg(rgb(0x596474)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                        this.resize_offset =
                                            f32::from(event.position.x) - this.sidebar_width;
                                        this.resizing_sidebar = true;
                                        cx.notify();
                                    }),
                                ),
                        )
                    })
                    .child(div().flex_1().h_full()),
            )
    }
}

fn main() {
    let loader = cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
    assert!(loader.load(), "Failed to load CEF framework");
    cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
    Application::new().run(|cx: &mut App| {
        let args = cef::args::Args::new();
        // GPUI owns the event loop; CEF is pumped on the same main thread.
        let profile = std::env::temp_dir().join("watra-gpui-cef-profile");
        let settings = cef::Settings {
            external_message_pump: 1,
            root_cache_path: cef::CefString::from(profile.to_str().unwrap()),
            ..Default::default()
        };
        assert_eq!(
            cef::initialize(
                Some(args.as_main_args()),
                Some(&settings),
                None,
                std::ptr::null_mut()
            ),
            1
        );
        cx.spawn(async move |cx| {
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(10)).await;
                if cx.update(|_| cef::do_message_loop_work()).is_err() {
                    break;
                }
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        let browser_window = cx
            .open_window(
                WindowOptions {
                    // Extend content into the titlebar while keeping native window controls.
                    titlebar: Some(TitlebarOptions {
                        title: Some("Watra".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.), px(20.))),
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| BrowserShell {
                        browser: None,
                        sidebar_open: true,
                        sidebar_width: 240.,
                        resizing_sidebar: false,
                        resize_offset: 0.,
                        native_frame: None,
                        divider_hovered: false,
                    })
                },
            )
            .expect("Failed to create Watra window");
        // Track the native mouse while dragging across the CEF child view.
        // GPUI alone cannot receive every movement once the pointer enters CEF.
        cx.spawn(async move |cx| {
            loop {
                gpui::Timer::after(std::time::Duration::from_millis(16)).await;
                if browser_window
                    .update(cx, |shell, window, cx| {
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
                            let mouse = native_window.mouseLocationOutsideOfEventStream();
                            let left_pressed =
                                NSEvent::pressedMouseButtons(cocoa::base::nil) & 1 != 0;
                            let inside_height = mouse.y >= 0.
                                && mouse.y
                                    < (f32::from(window.viewport_size().height) - 54.) as f64;
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
                                let _: () = objc2::msg_send![
                                    cursor.cast::<objc2::runtime::AnyObject>(),
                                    set
                                ];
                            } else if was_resize_cursor {
                                let cursor = NSCursor::arrow_cursor(cocoa::base::nil);
                                let _: () = objc2::msg_send![
                                    cursor.cast::<objc2::runtime::AnyObject>(),
                                    set
                                ];
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
                                    let maximum = (f32::from(window.viewport_size().width) - 320.)
                                        .clamp(180., 360.);
                                    shell.sidebar_width =
                                        (mouse.x as f32 - shell.resize_offset).clamp(180., maximum);
                                } else {
                                    shell.resizing_sidebar = false;
                                }
                                cx.notify();
                            }
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}
