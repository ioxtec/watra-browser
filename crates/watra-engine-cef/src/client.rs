use cef::rc::Rc as _;
use cef::{
    Client, ImplClient, ImplLifeSpanHandler, LifeSpanHandler, WrapClient, WrapLifeSpanHandler,
};
use cef::{ImplFrame, ImplLoadHandler, LoadHandler, WrapLoadHandler, wrap_load_handler};
use std::{cell::Cell, rc::Rc};

cef::wrap_life_span_handler! {
    struct BrowserLifetime { closed: Rc<Cell<usize>> }
    impl LifeSpanHandler {
        fn on_before_close(&self, _browser: Option<&mut cef::Browser>) {
            self.closed.set(self.closed.get() + 1);
        }
    }
}

cef::wrap_client! {
    struct BrowserClient { closed: Rc<Cell<usize>> }
    impl Client {
        fn life_span_handler(&self) -> Option<cef::LifeSpanHandler> {
            Some(BrowserLifetime::new(self.closed.clone()))
        }
        fn load_handler(&self) -> Option<LoadHandler> { Some(PageAppearance::new()) }
    }
}

/// Creates a CEF client with lifetime tracking and Watra's scrollbar styling.
pub fn create_client(closed: Rc<Cell<usize>>) -> cef::Client {
    BrowserClient::new(closed)
}

cef::wrap_load_handler! {
    struct PageAppearance;
    impl LoadHandler {
        fn on_load_end(&self, _browser: Option<&mut cef::Browser>, frame: Option<&mut cef::Frame>, _status: i32) {
            if let Some(frame) = frame && frame.is_main() != 0 {
                // Only style scrollbar chrome; page layout and scrolling remain native.
                let script = r#"(() => {
                    if (document.getElementById('watra-scrollbar-style')) return;
                    const style = document.createElement('style');
                    style.id = 'watra-scrollbar-style';
                    style.textContent = '::-webkit-scrollbar{background:transparent}::-webkit-scrollbar-track{background:transparent}html{scrollbar-color:rgba(130,130,130,.65) transparent}';
                    document.documentElement.appendChild(style);
                })()"#;
                frame.execute_java_script(Some(&cef::CefString::from(script)), None, 0);
            }
        }
    }
}
