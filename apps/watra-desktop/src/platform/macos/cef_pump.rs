// The external CEF message pump is bridged to an AppKit NSTimer.
#![allow(unsafe_code)]

use cef::rc::Rc as _;
use cef::{
    App, BrowserProcessHandler, ImplApp, ImplBrowserProcessHandler, WrapApp,
    WrapBrowserProcessHandler, wrap_app, wrap_browser_process_handler,
};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained,
    runtime::NSObject, sel,
};
use objc2_app_kit::{NSApplication, NSCursor, NSCursorFrameResizePosition, NSEvent};
use objc2_foundation::{NSObjectProtocol, NSRunLoop, NSRunLoopCommonModes, NSTimer};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

pub(crate) struct PumpSchedule {
    epoch: Instant,
    next_work: AtomicU64,
}
impl PumpSchedule {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            epoch: Instant::now(),
            next_work: AtomicU64::new(0),
        })
    }
    fn now(&self) -> u64 {
        self.epoch.elapsed().as_millis().min((u64::MAX - 1) as u128) as u64
    }
    fn request(&self, delay_ms: i64) {
        self.next_work.store(
            self.now()
                .saturating_add(delay_ms.max(0) as u64)
                .min(u64::MAX - 1),
            Ordering::Release,
        );
    }
    fn take_due(&self) -> bool {
        let deadline = self.next_work.load(Ordering::Acquire);
        deadline <= self.now()
            && self
                .next_work
                .compare_exchange(deadline, u64::MAX, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
    }
    fn ensure_idle_work(&self) {
        // Match CEF's external-pump fallback without replacing a new engine request.
        let _ = self.next_work.compare_exchange(
            u64::MAX,
            self.now().saturating_add(33),
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}
wrap_browser_process_handler! {
    struct PumpBrowserProcess { schedule: Arc<PumpSchedule> }
    impl BrowserProcessHandler {
        fn on_schedule_message_pump_work(&self, delay_ms: i64) {
            // CEF may request work from any thread; native dispatch stays on the main thread.
            self.schedule.request(delay_ms);
        }
    }
}
wrap_app! {
    struct PumpApplication { schedule: Arc<PumpSchedule> }
    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(PumpBrowserProcess::new(self.schedule.clone()))
        }
    }
}
pub(crate) fn application(schedule: Arc<PumpSchedule>) -> App {
    PumpApplication::new(schedule)
}

pub struct PumpState {
    stopped: Rc<Cell<bool>>,
    quitting: Rc<Cell<bool>>,
    schedule: Arc<PumpSchedule>,
    pumping: Cell<bool>,
    native_mouse_sequence: Cell<bool>,
    was_live_resize: Cell<bool>,
    resize_cursor: Cell<Option<NSCursorFrameResizePosition>>,
}
impl PumpState {
    fn native_interaction_active(&self) -> bool {
        let pressed = NSEvent::pressedMouseButtons() & 1 != 0;
        if !pressed {
            self.native_mouse_sequence.set(false);
        }
        let mtm = MainThreadMarker::new().expect("CEF pump requires the main thread");
        let app = NSApplication::sharedApplication(mtm);
        let Some(window) = app.keyWindow().or_else(|| app.mainWindow()) else {
            return false;
        };
        let mouse = NSEvent::mouseLocation();
        let frame = window.frame();
        let x = mouse.x - frame.origin.x;
        let y = mouse.y - frame.origin.y;
        let live_resize = window.inLiveResize();
        let was_live_resize = self.was_live_resize.replace(live_resize);
        let cursor = if pressed && self.resize_cursor.get().is_some() {
            self.resize_cursor.get()
        } else {
            crate::window_chrome::resize_position(x, y, frame.size.width, frame.size.height)
        };
        if let Some(position) = cursor {
            crate::window_chrome::set_resize_cursor(position);
        } else if self.resize_cursor.get().is_some() {
            NSCursor::arrowCursor().set();
        }
        self.resize_cursor.set(cursor);
        // Once AppKit owns live resize, keep CEF painting and flush the final size on release.
        if live_resize || was_live_resize {
            self.schedule.request(0);
            return false;
        }
        let native_target = is_native_target(x, y, frame.size.width, frame.size.height);
        if pressed && native_target {
            self.native_mouse_sequence.set(true);
        }
        native_target
            || self.native_mouse_sequence.get()
            || crate::window_chrome::native_tracking_active()
    }
}
fn is_native_target(x: f64, y: f64, width: f64, height: f64) -> bool {
    let inside = x >= -12. && x <= width + 12. && y >= -12. && y <= height + 12.;
    let border = inside && (x <= 12. || x >= width - 12. || y <= 12. || y >= height - 12.);
    let controls = (0. ..=100.).contains(&x) && y >= height - 54. && y <= height;
    border || controls
}

#[cfg(test)]
mod tests {
    use super::is_native_target;
    #[test]
    fn reserves_all_edges_and_corners_including_outer_hit_area() {
        for (x, y) in [
            (-4., 300.),
            (904., 300.),
            (450., -4.),
            (450., 604.),
            (-4., -4.),
            (904., -4.),
            (-4., 604.),
            (904., 604.),
        ] {
            assert!(is_native_target(x, y, 900., 600.));
        }
    }
    #[test]
    fn leaves_browser_content_active_and_reserves_window_controls() {
        assert!(!is_native_target(400., 300., 900., 600.));
        assert!(!is_native_target(1000., 300., 900., 600.));
        assert!(is_native_target(35., 575., 900., 600.));
    }
}

define_class! {
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = PumpState]
    struct PumpTarget;
    unsafe impl NSObjectProtocol for PumpTarget {}
    impl PumpTarget {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            let state = self.ivars();
            if state.stopped.get() || state.pumping.get() { return; }
            // AppKit owns frame/button tracking. Re-entering CEF here can consume its mouse events.
            if !state.quitting.get() && state.native_interaction_active() { return; }
            if !state.schedule.take_due() { return; }
            state.pumping.set(true);
            cef::do_message_loop_work();
            state.pumping.set(false);
            state.schedule.ensure_idle_work();
        }
    }
}
pub(crate) struct CefPump {
    timer: Retained<NSTimer>,
    _target: Retained<PumpTarget>,
}
impl CefPump {
    pub(crate) fn new(
        stopped: Rc<Cell<bool>>,
        quitting: Rc<Cell<bool>>,
        schedule: Arc<PumpSchedule>,
    ) -> Self {
        let mtm = MainThreadMarker::new().expect("CEF pump requires the main thread");
        let target = PumpTarget::alloc(mtm).set_ivars(PumpState {
            stopped,
            quitting,
            schedule,
            pumping: Cell::new(false),
            native_mouse_sequence: Cell::new(false),
            was_live_resize: Cell::new(false),
            resize_cursor: Cell::new(None),
        });
        let target: Retained<PumpTarget> = unsafe { msg_send![super(target), init] };
        let timer = unsafe {
            NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
                0.01,
                &target,
                sel!(tick:),
                None,
                true,
            )
        };
        unsafe {
            NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes);
        }
        Self {
            timer,
            _target: target,
        }
    }
    pub(crate) fn stop(&self) {
        self.timer.invalidate();
    }
}
impl Drop for CefPump {
    fn drop(&mut self) {
        self.stop();
    }
}
