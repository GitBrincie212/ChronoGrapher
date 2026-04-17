use chronographer::task::TaskFrame;
use chronographer::task::TaskFrameContext;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::impl_counting_frame;

#[allow(dead_code)]
#[derive(Debug)]
struct DummyError(&'static str);

impl Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "threshold error")
    }
}

impl_counting_frame!(DummyError);
