// CEF requires a custom NSApplication subclass at the Objective-C boundary.
#![allow(unsafe_code)]

use objc2::{
    MainThreadMarker, msg_send,
    runtime::{AnyClass, AnyObject, AnyProtocol, Bool, ClassBuilder, Sel},
    sel,
};
use objc2_app_kit::NSEvent;
use std::cell::Cell;

thread_local! {
    static HANDLING_SEND_EVENT: Cell<bool> = const { Cell::new(false) };
}

extern "C" fn is_handling_event(_: *mut AnyObject, _: Sel) -> Bool {
    Bool::new(HANDLING_SEND_EVENT.with(Cell::get))
}
extern "C" fn set_handling_event(_: *mut AnyObject, _: Sel, value: Bool) {
    HANDLING_SEND_EVENT.with(|state| state.set(value.as_bool()));
}
extern "C" fn send_event(app: *mut AnyObject, _: Sel, event: *mut NSEvent) {
    let previous = HANDLING_SEND_EVENT.with(|state| state.replace(true));
    let superclass =
        AnyClass::get(c"GPUIApplication").expect("GPUI application class is registered");
    // CEF needs the send-event scope to cooperate with AppKit's native tracking loops.
    unsafe {
        let _: () = msg_send![super(app, superclass), sendEvent: event];
    }
    HANDLING_SEND_EVENT.with(|state| state.set(previous));
}

pub(crate) fn install() {
    MainThreadMarker::new().expect("Application setup requires the main thread");
    let superclass =
        AnyClass::get(c"GPUIApplication").expect("GPUI application class is registered");
    let mut builder = ClassBuilder::new(c"WatraApplication", superclass)
        .expect("Watra application class must be registered once");
    unsafe {
        builder.add_method(
            sel!(isHandlingSendEvent),
            is_handling_event as extern "C" fn(*mut AnyObject, Sel) -> Bool,
        );
        builder.add_method(
            sel!(setHandlingSendEvent:),
            set_handling_event as extern "C" fn(*mut AnyObject, Sel, Bool),
        );
        builder.add_method(
            sel!(sendEvent:),
            send_event as extern "C" fn(*mut AnyObject, Sel, *mut NSEvent),
        );
    }
    for name in [c"CrAppProtocol", c"CrAppControlProtocol", c"CefAppProtocol"] {
        if let Some(protocol) = AnyProtocol::get(name) {
            builder.add_protocol(protocol);
        }
    }
    let class = builder.register();
    // Inherit GPUI's platform ivar while providing the application hooks required by CEF.
    let app: *mut AnyObject = unsafe { msg_send![class, sharedApplication] };
    assert!(
        !app.is_null() && std::ptr::eq(unsafe { &*app }.class(), class),
        "NSApplication was initialized before the Watra application bridge"
    );
}
