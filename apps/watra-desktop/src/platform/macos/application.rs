// CEF shutdown retains and detaches its native AppKit child view.
#![allow(unsafe_code)]

use crate::{
    browser_shell::{self, BrowserShell},
    cef_application, cef_pump, diagnostics,
    platform::macos::renderer_host::CefRendererHost,
};
use cef::{ImplBrowser, ImplBrowserHost};
use gpui::{prelude::*, *};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

actions!(watra, [Quit, FocusAddress, NewTab, CloseTab]);

pub(crate) fn run() {
    diagnostics::event("cef.loader.begin");
    let loader = cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
    assert!(loader.load(), "Failed to load CEF framework");
    diagnostics::event("cef.loader.loaded");
    cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
    cef_application::install();
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus(vec![Menu {
            name: "Watra".into(),
            items: vec![
                MenuItem::action("Open Location…", FocusAddress),
                MenuItem::action("Quit Watra", Quit),
            ],
        }]);
        let args = cef::args::Args::new();
        // GPUI owns the event loop; CEF is pumped on the same main thread.
        let profile = std::env::temp_dir().join("watra-gpui-cef-profile");
        let settings = cef::Settings {
            external_message_pump: 1,
            root_cache_path: cef::CefString::from(profile.to_str().unwrap()),
            ..Default::default()
        };
        let pump_schedule = cef_pump::PumpSchedule::new();
        let mut cef_app = cef_pump::application(pump_schedule.clone());
        assert_eq!(
            cef::initialize(
                Some(args.as_main_args()),
                Some(&settings),
                Some(&mut cef_app),
                std::ptr::null_mut()
            ),
            1
        );
        diagnostics::event("cef.runtime.initialized");
        let quitting = Rc::new(Cell::new(false));
        let runtime_stopped = Rc::new(Cell::new(false));
        let cef_pump =
            cef_pump::CefPump::new(runtime_stopped.clone(), quitting.clone(), pump_schedule);
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        let runtime_browsers = Rc::new(RefCell::new(Vec::new()));
        let browsers_created = Rc::new(Cell::new(0));
        let browsers_closed = Rc::new(Cell::new(0));
        let navigation_core = Rc::new(RefCell::new(None));
        let shell_core = navigation_core.clone();
        let address_editing = Rc::new(Cell::new(false));
        let renderer = Box::new(CefRendererHost::new(
            runtime_browsers.clone(),
            browsers_created.clone(),
            browsers_closed.clone(),
            navigation_core.clone(),
            address_editing.clone(),
        ));
        let browser_window = cx
            .open_window(
                WindowOptions {
                    // Extend content into the titlebar while keeping native window controls.
                    titlebar: Some(TitlebarOptions {
                        title: Some("Watra".into()),
                        appears_transparent: true,
                        traffic_light_position: None,
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    // Keep the address field and browser viewport usable at the smallest size.
                    window_min_size: Some(size(px(1000.), px(520.))),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| BrowserShell::new(renderer, shell_core, address_editing)),
            )
            .expect("Failed to create Watra window");
        diagnostics::event("window.created");
        cx.bind_keys([
            KeyBinding::new("cmd-l", FocusAddress, None),
            KeyBinding::new("cmd-t", NewTab, None),
            KeyBinding::new("cmd-w", CloseTab, None),
        ]);
        cx.on_action(move |_: &FocusAddress, cx| {
            // Menu dispatch temporarily owns the current window; focus after it returns.
            cx.defer(move |cx| {
                let _ = browser_window.update(cx, |shell, _, cx| {
                    shell.focus_address();
                    cx.notify();
                });
            });
        });
        cx.on_action(move |_: &NewTab, cx| {
            cx.defer(move |cx| {
                let _ = browser_window.update(cx, |shell, window, cx| {
                    shell.create_tab(window);
                    cx.notify();
                });
            });
        });
        cx.on_action(move |_: &CloseTab, cx| {
            cx.defer(move |cx| {
                let _ = browser_window.update(cx, |shell, window, cx| {
                    let active_tab = shell.active_tab;
                    shell.close_tab(active_tab, window);
                    cx.notify();
                });
            });
        });
        let quit_requested = quitting.clone();
        cx.on_action(move |_: &Quit, _| quit_requested.set(true));
        let close_requested = quitting.clone();
        let _ = browser_window.update(cx, |_, window, cx| {
            window.on_window_should_close(cx, move |_, _| {
                // A native close may arrive while GPUI owns this window; only signal here.
                // CEF requests a second native close while destroying its child browser.
                close_requested.replace(true)
            });
        });
        let shutdown_requested = quitting.clone();
        cx.spawn(async move |cx| {
            while !shutdown_requested.get() {
                gpui::Timer::after(std::time::Duration::from_millis(10)).await;
            }
            let _ = browser_window.update(cx, |shell, _, _| {
                shell.take_browser();
            });
            let browsers = runtime_browsers.borrow_mut().drain(..).collect::<Vec<_>>();
            for browser in browsers {
                if let Some(host) = browser.host() {
                    // The embedded NSView must leave the hierarchy before CEF can finish closing.
                    let native_view = unsafe {
                        objc2::rc::Retained::<objc2_app_kit::NSView>::retain(
                            host.window_handle().cast(),
                        )
                    };
                    host.close_browser(1);
                    if let Some(view) = native_view {
                        view.removeFromSuperview();
                    }
                }
                drop(browser);
            }
            // Keep pumping until every tab has reported its native close callback.
            while browsers_closed.get() < browsers_created.get() {
                gpui::Timer::after(std::time::Duration::from_millis(10)).await;
            }
            runtime_stopped.set(true);
            cef_pump.stop();
            cef::shutdown();
            let _ = cx.update(|cx| cx.quit());
        })
        .detach();
        browser_shell::start_pointer_tracking(cx, browser_window, quitting.clone());
        cx.on_window_closed(move |cx| {
            if cx.windows().is_empty() {
                // Let the CEF shutdown task finish before terminating the GPUI runtime.
                quitting.set(true);
            }
        })
        .detach();
        cx.activate(true);
    });
}
