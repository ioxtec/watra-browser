// The native address field is an Objective-C target-action bridge.
#![allow(unsafe_code)]

use crate::browser_session::NavigationCore;
use crate::diagnostics;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained,
    runtime::NSObject, sel,
};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBox, NSBoxType, NSColor, NSEventType, NSFocusRingType, NSFont,
    NSTextField, NSTitlePosition, NSView,
};
use objc2_foundation::{NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString};
use std::{cell::Cell, rc::Rc};
use watra_engine_api::NavigationRequest;

pub struct AddressState {
    core: NavigationCore,
    editing: Rc<Cell<bool>>,
}
define_class! {
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = AddressState]
    struct AddressTarget;
    unsafe impl NSObjectProtocol for AddressTarget {}
    impl AddressTarget {
        #[unsafe(method(submit:))]
        fn submit(&self, sender: &NSTextField) {
            self.navigate(sender);
        }
    }
}

impl AddressTarget {
    fn navigate(&self, sender: &NSTextField) {
        // AppKit may send the control action when editing ends; only Return submits.
        let mtm = MainThreadMarker::new().expect("Address submission requires the main thread");
        let application = objc2_app_kit::NSApplication::sharedApplication(mtm);
        let Some(event) = application.currentEvent() else {
            diagnostics::event("address.action.ignored event=none");
            return;
        };
        let event_type = event.r#type();
        if event_type != NSEventType::KeyDown {
            diagnostics::event(&format!("address.action.ignored event_type={event_type:?}"));
            return;
        }
        if !matches!(event.keyCode(), 36 | 76) {
            diagnostics::event("address.action.ignored key=other");
            return;
        }
        diagnostics::event("address.submit.enter");
        let Some(destination) = navigation_target(&sender.stringValue().to_string()) else {
            return;
        };
        let Ok(request) = NavigationRequest::new(destination.as_str()) else {
            return;
        };
        if let Some(core) = self.ivars().core.borrow_mut().as_mut()
            && let Err(error) = core.navigate(request)
        {
            diagnostics::event("address.navigation.failed");
            eprintln!("Address navigation failed: {error}");
        }
        self.ivars().editing.set(false);
    }
}

pub(crate) struct NativeAddress {
    field: Retained<NSTextField>,
    panel_extension: Retained<NSBox>,
    _target: Retained<AddressTarget>,
    editing: Rc<Cell<bool>>,
}
impl NativeAddress {
    pub(crate) unsafe fn new(
        parent: *mut std::ffi::c_void,
        core: NavigationCore,
        editing: Rc<Cell<bool>>,
    ) -> Self {
        let mtm = MainThreadMarker::new().expect("Address field requires the main thread");
        let panel_extension = NSBox::initWithFrame(NSBox::alloc(mtm), NSRect::default());
        panel_extension.setBoxType(NSBoxType::Custom);
        panel_extension.setTitlePosition(NSTitlePosition::NoTitle);
        panel_extension.setTransparent(false);
        panel_extension.setBorderWidth(0.);
        panel_extension.setCornerRadius(18.);
        panel_extension.setFillColor(&NSColor::colorWithSRGBRed_green_blue_alpha(
            0x2b as f64 / 255.,
            0x2d as f64 / 255.,
            0x31 as f64 / 255.,
            1.,
        ));
        panel_extension.setHidden(true);
        unsafe {
            (&*parent.cast::<NSView>()).addSubview(&panel_extension);
        }

        let field = NSTextField::textFieldWithString(&NSString::from_str(""), mtm);
        field.setPlaceholderString(Some(&NSString::from_str("Search or enter address")));
        field.setBezeled(false);
        field.setBordered(false);
        field.setFocusRingType(NSFocusRingType::None);
        field.setDrawsBackground(false);
        // Keep the editor aligned with the floating address panel as the window grows.
        field.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewMinYMargin,
        );
        field.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
            0.90, 0.91, 0.92, 1.,
        )));
        field.setFont(Some(&NSFont::systemFontOfSize(16.)));
        field.setHidden(true);
        let target = AddressTarget::alloc(mtm).set_ivars(AddressState {
            core,
            editing: editing.clone(),
        });
        let target: Retained<AddressTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            field.setTarget(Some(&target));
            field.setAction(Some(sel!(submit:)));
            (&*parent.cast::<NSView>()).addSubview(&field);
        }
        Self {
            field,
            panel_extension,
            _target: target,
            editing,
        }
    }
    pub(crate) fn update_frame(&self) {
        // Use live AppKit bounds rather than a potentially stale GPUI viewport during drag.
        let Some(parent) = (unsafe { self.field.superview() }) else {
            return;
        };
        let bounds = parent.bounds();
        let panel_y = if parent.isFlipped() {
            bounds.origin.y + 56.
        } else {
            bounds.origin.y + bounds.size.height - 203.
        };
        let extension_frame = NSRect::new(
            NSPoint::new(bounds.origin.x + 242., panel_y),
            NSSize::new((bounds.size.width - 298.).max(0.), 147.),
        );
        if self.panel_extension.frame() != extension_frame {
            self.panel_extension.setFrame(extension_frame);
        }

        // GPUI's host view may be flipped, so derive the field's top edge from its coordinate system.
        let y = if parent.isFlipped() {
            bounds.origin.y + 20.
        } else {
            bounds.origin.y + bounds.size.height - 54.
        };
        let frame = NSRect::new(
            NSPoint::new(bounds.origin.x + 302., y),
            NSSize::new((bounds.size.width - 590.).max(120.), 34.),
        );
        // Setting an unchanged frame can invalidate AppKit cursor rectangles.
        let current = self.field.frame();
        if current.origin.x != frame.origin.x
            || current.origin.y != frame.origin.y
            || current.size.width != frame.size.width
            || current.size.height != frame.size.height
        {
            self.field.setFrame(frame);
        }
        let hidden = !self.editing.get();
        if self.panel_extension.isHidden() != hidden {
            self.panel_extension.setHidden(hidden);
        }
        if self.field.isHidden() != hidden {
            self.field.setHidden(hidden);
            diagnostics::event(if hidden {
                "address.native_field.hidden"
            } else {
                "address.native_field.visible"
            });
        }
    }
    pub(crate) fn set_url(&self, url: &str) {
        self.field.setStringValue(&NSString::from_str(url));
    }
    pub(crate) fn submit(&self) {
        self.navigate_current_value();
    }

    fn navigate_current_value(&self) {
        let Some(destination) = navigation_target(&self.field.stringValue().to_string()) else {
            return;
        };
        let Ok(request) = NavigationRequest::new(destination.as_str()) else {
            return;
        };
        if let Some(core) = self._target.ivars().core.borrow_mut().as_mut()
            && let Err(error) = core.navigate(request)
        {
            diagnostics::event("address.navigation.failed");
            eprintln!("Address navigation failed: {error}");
        }
        self.editing.set(false);
    }
    pub(crate) fn focus(&self) {
        diagnostics::event("address.focus.begin");
        self.update_frame();
        if let Some(window) = self.field.window() {
            let focused = window.makeFirstResponder(Some(&self.field));
            diagnostics::event(if focused {
                "address.focus.first_responder_set"
            } else {
                "address.focus.first_responder_rejected"
            });
            if focused {
                if let Some(editor) = self.field.currentEditor() {
                    let length = self.field.stringValue().length();
                    editor.setSelectedRange(NSRange::new(0, length));
                    diagnostics::event("address.focus.text_selected");
                } else {
                    diagnostics::event("address.focus.editor_unavailable");
                }
            }
        } else {
            diagnostics::event("address.focus.no_window");
        }
    }
}

fn navigation_target(value: &str) -> Option<url::Url> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("https://") || value.starts_with("http://") {
        return url::Url::parse(value).ok();
    }
    if !value.contains(char::is_whitespace)
        && (value.contains('.') || value.starts_with("localhost"))
    {
        return url::Url::parse(&format!("https://{value}")).ok();
    }
    let mut target = url::Url::parse("https://www.google.com/search").ok()?;
    target.query_pairs_mut().append_pair("q", value);
    Some(target)
}

#[cfg(test)]
mod tests {
    use super::navigation_target;
    #[test]
    fn normalizes_host() {
        assert_eq!(
            navigation_target(" wikipedia.org ").unwrap().as_str(),
            "https://wikipedia.org/"
        );
    }
    #[test]
    fn encodes_search_query() {
        assert_eq!(
            navigation_target("rust & cef")
                .unwrap()
                .query_pairs()
                .find(|(k, _)| k == "q")
                .unwrap()
                .1,
            "rust & cef"
        );
    }
    #[test]
    fn ignores_empty_input() {
        assert!(navigation_target("  ").is_none());
    }
}
