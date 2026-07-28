use chronographer::prelude::*;

#[event(payload = &'b T)]
pub trait MyOwnTHEG<'b, T: Send + Sync + 'static> {}

fn main() {}
