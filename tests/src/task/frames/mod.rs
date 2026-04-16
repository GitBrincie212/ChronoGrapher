mod collectionframe_test;
mod dynamic_taskframe_test;
mod noop_operation_taskframe_test;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! impl_counting_frame {
        ($err:ident) => {
            #[allow(dead_code)]
            fn ok_frame(
                counter: &Arc<AtomicUsize>,
            ) -> Arc<dyn chronographer::task::ErasedTaskFrame<()>> {
                Arc::new(CountingFrame {
                    counter: counter.clone(),
                    should_fail: false,
                })
            }

            #[allow(dead_code)]
            fn failing_frame(
                counter: &Arc<AtomicUsize>,
            ) -> Arc<dyn chronographer::task::ErasedTaskFrame<()>> {
                Arc::new(CountingFrame {
                    counter: counter.clone(),
                    should_fail: true,
                })
            }

            #[allow(dead_code)]
            struct CountingFrame {
                counter: Arc<AtomicUsize>,
                should_fail: bool,
            }

            impl TaskFrame for CountingFrame {
                type Error = $err;
                type Args = ();

                fn execute(
                    &self,
                    _ctx: &TaskFrameContext,
                    _args: &Self::Args,
                ) -> impl Future<Output = Result<(), Self::Error>> + Send {
                    async move {
                        self.counter.fetch_add(1, Ordering::SeqCst);
                        if self.should_fail {
                            Err($err("frame failed"))
                        } else {
                            Ok(())
                        }
                    }
                }
            }
        };
    }
}
