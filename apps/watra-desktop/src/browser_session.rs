use std::{cell::RefCell, rc::Rc};
use watra_core::BrowserCore;
use watra_engine_api::BrowserEngine;

/// Shared active session; concrete renderers are selected by the composition root.
pub(crate) type NavigationCore = Rc<RefCell<Option<BrowserCore<Box<dyn BrowserEngine>>>>>;
