use std::rc::Rc;

use uuid::Uuid;
use yew::{html::Scope, prelude::*};

pub const AUTOMATIC_CLOSE_DELAY: u32 = 2500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastVariant {
    Info,
    Success,
    Warn,
    Error,
}

impl From<ToastVariant> for Classes {
    fn from(variant: ToastVariant) -> Self {
        match variant {
            ToastVariant::Info => classes!("info"),
            ToastVariant::Success => classes!("success"),
            ToastVariant::Warn => classes!("warn"),
            ToastVariant::Error => classes!("error"),
        }
    }
}

#[derive(Clone)]
pub struct CloseCallback(Rc<dyn Fn()>);

impl CloseCallback {
    pub fn call(self) {
        self.0();
    }
}

impl std::fmt::Debug for CloseCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CloseCallback").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastAutomaticallyClosing {
    No,
    WithDefaultDelay,
    WithDelay { millis: u32 }
}

/// Create a new toast with the `Default`implementation, then overwrite desired fields.
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: Uuid,
    pub created_at: time::OffsetDateTime,
    pub variant: ToastVariant,
    pub heading: String,
    pub message: String,
    pub dismissible: bool,
    pub automatically_closing: ToastAutomaticallyClosing,
    pub close_callback: Option<CloseCallback>,
}

impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
            variant: ToastVariant::Info,
            heading: String::new(),
            message: String::new(),
            dismissible: false,
            automatically_closing: ToastAutomaticallyClosing::WithDefaultDelay,
            close_callback: None,
        }
    }
}

impl CloseCallback {
    //pub fn new<COMP, MSG, F>(toast: Toast, link: Scope<COMP>, message_provider: F) -> Self
    //where
    //    COMP: Component,
    //    MSG: Into<COMP::Message>,
    //    F: Fn(Toast) -> MSG + 'static,
    //{
    //    let toast = toast.clone();
    //    let timeout = Rc::new(Timeout::new(toast.automatic_close_delay, move || {
    //        let message = message_provider(toast).into();
    //        link.send_message(message)
    //    }));
    //    Self(timeout)
    //}
    pub fn new<COMP, MSG, F>(toast: Toast, link: Scope<COMP>, message_provider: F) -> Self
    where
        COMP: Component,
        MSG: Into<COMP::Message>,
        F: Fn(Toast) -> MSG + 'static,
    {
        Self(Rc::new(move || {
            let message = message_provider(toast.clone()).into();
            link.send_message(message)
        }))
    }
}
