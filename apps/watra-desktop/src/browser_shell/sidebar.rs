use super::{BrowserShell, effective_sidebar_width};
use gpui::{prelude::*, *};

pub(super) fn render(
    shell: &mut BrowserShell,
    window: &Window,
    cx: &mut Context<BrowserShell>,
) -> impl IntoElement {
    let sidebar_width = effective_sidebar_width(
        shell.sidebar_open,
        shell.sidebar_width,
        f32::from(window.viewport_size().width),
    );
    div()
        .flex_1()
        .flex()
        .child(
            div()
                .w(px(sidebar_width))
                .h_full()
                .overflow_hidden()
                .bg(rgb(0x202328))
                .when(shell.sidebar_open, |sidebar| {
                    sidebar
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().px_1().py_1().text_lg().child("Watra"))
                        .child(
                            div()
                                .id("new-tab")
                                .h(px(36.))
                                .px_2()
                                .flex()
                                .items_center()
                                .gap_2()
                                .rounded_md()
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0x2b3037)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, window, cx| {
                                        cx.stop_propagation();
                                        this.create_tab(window);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    div()
                                        .text_size(px(20.))
                                        .text_color(rgb(0x9da5b0))
                                        .child("+"),
                                )
                                .child(div().text_color(rgb(0xc4c8cf)).child("New Tab")),
                        )
                        .child(
                            div()
                                .mt_2()
                                .mb_1()
                                .text_sm()
                                .text_color(rgb(0x9da5b0))
                                .child("TABS"),
                        )
                        .children(shell.tabs.iter().map(|tab| {
                            let id = tab.id;
                            let title = tab.title.clone();
                            let active = id == shell.active_tab;
                            div()
                                .id(ElementId::Name(format!("tab-{id}").into()))
                                .h(px(38.))
                                .px_2()
                                .flex()
                                .items_center()
                                .gap_2()
                                .rounded_md()
                                .when(active, |row| row.bg(rgb(0x343a43)))
                                .when(!active, |row| row.hover(|style| style.bg(rgb(0x292d33))))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.activate_tab(id);
                                        cx.notify();
                                    }),
                                )
                                .child(div().size(px(8.)).rounded_full().bg(rgb(if active {
                                    0xe7a957
                                } else {
                                    0x727985
                                })))
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .id(ElementId::Name(format!("close-tab-{id}").into()))
                                        .size(px(24.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_sm()
                                        .text_color(rgb(0x9da5b0))
                                        .hover(|style| {
                                            style.bg(rgb(0x464c55)).text_color(rgb(0xffffff))
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, window, cx| {
                                                cx.stop_propagation();
                                                this.close_tab(id, window);
                                                cx.notify();
                                            }),
                                        )
                                        .child("×"),
                                )
                        }))
                        .child(div().flex_1())
                }),
        )
        .when(shell.sidebar_open, |row| {
            row.child(
                div()
                    .id("sidebar-divider")
                    .w(px(1.))
                    .h_full()
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .h(px(40.))
                            .top(relative(0.5))
                            .mt(px(-20.))
                            .rounded(px(2.))
                            .left(px(if shell.divider_hovered { -1. } else { 0. }))
                            .w(px(if shell.divider_hovered { 3. } else { 1. }))
                            .bg(if shell.divider_hovered || shell.resizing_sidebar {
                                rgba(0x707b8bcc)
                            } else {
                                rgba(0x00000000)
                            }),
                    )
                    .cursor(CursorStyle::ResizeLeftRight)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _, cx| {
                            this.resize_offset = f32::from(event.position.x) - this.sidebar_width;
                            this.resizing_sidebar = true;
                            cx.notify();
                        }),
                    ),
            )
        })
        .child(
            div().flex_1().h_full().p_2().child(
                div()
                    .size_full()
                    .rounded(px(8.))
                    .bg(rgb(0x202328))
                    .shadow_md(),
            ),
        )
}
