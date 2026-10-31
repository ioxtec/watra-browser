use super::BrowserShell;
use gpui::{prelude::*, *};
use std::time::Duration;

#[derive(Clone, Copy)]
enum ToolbarIcon {
    Back,
    Forward,
    Reload,
    Menu,
    Site,
    Lock,
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
                ToolbarIcon::Site => {
                    for i in 0..=32 {
                        let angle = i as f32 / 32. * std::f32::consts::TAU;
                        let point = p(8. + 6. * angle.cos(), 8. + 6. * angle.sin());
                        if i == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                    path.move_to(p(2., 8.));
                    path.line_to(p(14., 8.));
                    for i in 0..=16 {
                        let angle = i as f32 / 16. * std::f32::consts::PI;
                        let point = p(8. + 3. * (angle * 2.).cos(), 8. + 6. * angle.cos());
                        if i == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                }
                ToolbarIcon::Lock => {
                    for i in 0..=12 {
                        let angle = std::f32::consts::PI + i as f32 / 12. * std::f32::consts::PI;
                        let point = p(8. + 3.5 * angle.cos(), 6. + 3.5 * angle.sin());
                        if i == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                    path.move_to(p(4.5, 6.));
                    path.line_to(p(4.5, 3.));
                    path.line_to(p(11.5, 3.));
                    path.line_to(p(11.5, 6.));
                    path.move_to(p(3., 6.));
                    path.line_to(p(13., 6.));
                    path.line_to(p(13., 13.));
                    path.line_to(p(3., 13.));
                    path.line_to(p(3., 6.));
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
            .text_size(px(15.))
            .child("Watra settings — coming soon")
    }
}

pub(super) fn render(shell: &mut BrowserShell, cx: &mut Context<BrowserShell>) -> impl IntoElement {
    let editing = shell.address_editing.get();
    let hovered = shell.address_hovered;
    let address_url = shell.address_url.clone();
    let hover_generation = shell.address_hover_generation;
    let site_icon = toolbar_icon(ToolbarIcon::Site);
    let secure = address_url.starts_with("https://");
    div()
        // Native buttons and toolbar share a center at 27 points.
        .h(px(54.))
        .relative()
        .pt(px(13.))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|shell, event: &MouseDownEvent, window, cx| {
                let x = f32::from(event.position.x);
                let y = f32::from(event.position.y);
                if !shell.can_start_window_drag(x, y, f32::from(window.viewport_size().width)) {
                    return;
                }
                if event.click_count == 2 {
                    window.zoom_window();
                } else {
                    shell.begin_window_drag(window, cx);
                }
            }),
        )
        .pl(px(100.))
        .pr_4()
        .flex()
        .items_start()
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
                        crate::diagnostics::event("address.click");
                        cx.stop_propagation();
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
                    cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.go_back();
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
                    cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.go_forward();
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
                    cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.reload();
                    }),
                )
                .child(toolbar_icon(ToolbarIcon::Reload)),
        )
        .child(
            div()
                .id("address-bar")
                .flex_1()
                .ml(px(5.))
                .rounded_md()
                .h(px(30.))
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.address_editing.set(true);
                        this.rendered_address_editing = true;
                        this.address_focus_pending = true;
                        cx.notify();
                    }),
                )
                .child(site_icon)
                .when(secure, |bar| bar.child(toolbar_icon(ToolbarIcon::Lock)))
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .text_size(px(13.))
                        .text_color(rgb(if address_url == "Search or enter address" {
                            0x9da5b0
                        } else {
                            0xc4c8cf
                        }))
                        .child(address_url),
                )
                .with_animation(
                    ("address-hover", hover_generation),
                    Animation::new(Duration::from_millis(150)).with_easing(|delta| {
                        if delta < 0.5 {
                            2. * delta * delta
                        } else {
                            1. - (-2. * delta + 2.).powi(2) / 2.
                        }
                    }),
                    move |bar, progress| {
                        let opacity = if hover_generation == 0 {
                            0.
                        } else if hovered {
                            progress
                        } else {
                            1. - progress
                        };
                        let alpha = (opacity.clamp(0., 1.) * 0x55 as f32) as u32;
                        bar.bg(rgba(0x292d3200 | alpha))
                    },
                ),
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
                    cx.stop_propagation();
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
        )
        .when(editing, |toolbar| {
            toolbar.child(
                div()
                    .absolute()
                    .left(px(242.))
                    .right(px(56.))
                    .top(px(13.))
                    .h(px(190.))
                    .px_3()
                    .flex()
                    .flex_col()
                    .rounded(px(18.))
                    .bg(rgb(0x2b2d31))
                    .shadow_lg()
                    .child(
                        div()
                            .h(px(42.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_size(px(15.))
                                    .text_color(rgb(0x9da5b0))
                                    .child(toolbar_icon(ToolbarIcon::Site))
                                    .when(secure, |row| row.child(toolbar_icon(ToolbarIcon::Lock))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .text_size(px(15.))
                                    .text_color(rgb(0x9da5b0))
                                    .child("Enter to navigate")
                                    .child(
                                        div()
                                            .id("go-address")
                                            .h(px(34.))
                                            .px_3()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_size(px(15.))
                                            .rounded_full()
                                            .bg(rgb(0xe3e5e8))
                                            .text_color(rgb(0x25282d))
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.submit_renderer_address();
                                                    cx.notify();
                                                }),
                                            )
                                            .child("Go ↵"),
                                    )
                                    .child(
                                        div()
                                            .id("close-address-editor")
                                            .size(px(24.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded_full()
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(0x41454b)))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.address_editing.set(false);
                                                    this.rendered_address_editing = false;
                                                    cx.notify();
                                                }),
                                            )
                                            .child("×"),
                                    ),
                            ),
                    )
                    .child(div().mx(px(2.)).h(px(1.)).bg(rgb(0x3b3e43)))
                    .child(div().flex_1()),
            )
        })
}
