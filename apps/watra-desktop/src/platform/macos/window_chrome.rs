// Window dragging and resize cursors use public AppKit FFI APIs.
#![allow(unsafe_code)]

use gpui::{App, Window};
use std::cell::Cell;
thread_local! { static NATIVE_TRACKING: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn native_tracking_active() -> bool {
    NATIVE_TRACKING.with(Cell::get)
}
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSView};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub(crate) fn begin_drag(window: &Window, cx: &mut App) {
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };
    let view = unsafe { &*handle.ns_view.as_ptr().cast::<NSView>() };
    let Some(native_window) = view.window() else {
        return;
    };
    let mtm = MainThreadMarker::new().expect("Window dragging requires the main thread");
    let Some(event) = NSApplication::sharedApplication(mtm).currentEvent() else {
        return;
    };
    // AppKit runs a nested tracking loop; enter it after GPUI has released its context.
    cx.spawn(async move |_| {
        NATIVE_TRACKING.with(|state| state.set(true));
        native_window.performWindowDragWithEvent(&event);
        NATIVE_TRACKING.with(|state| state.set(false));
    })
    .detach();
}

pub(crate) fn resize_position(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<objc2_app_kit::NSCursorFrameResizePosition> {
    use objc2_app_kit::NSCursorFrameResizePosition as Position;
    if !(-6. ..=width + 6.).contains(&x)
        || !(-6. ..=height + 6.).contains(&y)
        || (x > 6. && x < width - 6. && y > 6. && y < height - 6.)
    {
        return None;
    }
    Some(
        match (x <= 12., x >= width - 12., y <= 12., y >= height - 12.) {
            (true, _, true, _) => Position::BottomLeft,
            (true, _, _, true) => Position::TopLeft,
            (_, true, true, _) => Position::BottomRight,
            (_, true, _, true) => Position::TopRight,
            (true, _, _, _) => Position::Left,
            (_, true, _, _) => Position::Right,
            (_, _, true, _) => Position::Bottom,
            _ => Position::Top,
        },
    )
}

#[allow(deprecated)]
pub(crate) fn set_resize_cursor(position: objc2_app_kit::NSCursorFrameResizePosition) {
    use objc2::{ClassType, msg_send, sel};
    use objc2_app_kit::{
        NSCursor, NSCursorFrameResizeDirections, NSCursorFrameResizePosition as Position,
    };
    let modern: bool = unsafe {
        msg_send![NSCursor::class(), respondsToSelector: sel!(frameResizeCursorFromPosition:inDirections:)]
    };
    let cursor = if modern {
        NSCursor::frameResizeCursorFromPosition_inDirections(
            position,
            NSCursorFrameResizeDirections::All,
        )
    } else {
        // Older macOS releases have only the basic public resize cursors.
        match position {
            Position::Left | Position::Right => NSCursor::resizeLeftRightCursor(),
            Position::Top | Position::Bottom => NSCursor::resizeUpDownCursor(),
            _ => NSCursor::crosshairCursor(),
        }
    };
    cursor.set();
}

#[cfg(test)]
mod tests {
    use super::resize_position;
    use objc2_app_kit::NSCursorFrameResizePosition as Position;
    #[test]
    fn maps_each_edge_and_corner_to_its_native_cursor() {
        for (x, y, expected) in [
            (1., 300., Position::Left),
            (899., 300., Position::Right),
            (450., 599., Position::Top),
            (450., 1., Position::Bottom),
            (1., 1., Position::BottomLeft),
            (899., 1., Position::BottomRight),
            (1., 599., Position::TopLeft),
            (899., 599., Position::TopRight),
        ] {
            assert_eq!(resize_position(x, y, 900., 600.), Some(expected));
        }
        assert_eq!(resize_position(200., 300., 900., 600.), None);
    }
}
