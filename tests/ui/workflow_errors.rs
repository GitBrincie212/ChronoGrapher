use chronographer::prelude::*;

fn main() {
    // ── 1. Standalone #[workflow] outside #[taskframe] / #[task] always errors
    //      The macro unconditionally emits a compile_error! when called directly.
    #[workflow(timeout(20s))]
    async fn standalone(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 2. Empty workflow(...) — at least one primitive is required
    #[taskframe]
    #[workflow()]
    async fn empty_workflow(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 3. Unknown primitive name
    #[taskframe]
    #[workflow(frobnicate(10s))]
    async fn unknown_primitive(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 4. timeout with no arguments
    #[taskframe]
    #[workflow(timeout())]
    async fn timeout_no_args(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 5. retry with zero retries
    #[taskframe]
    #[workflow(retry(0))]
    async fn retry_zero(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 6. retry with no arguments at all
    #[taskframe]
    #[workflow(retry())]
    async fn retry_no_args(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 7. #[workflow] on a non-async fn (taskframe already requires async,
    //      so this error comes from #[taskframe] itself)
    #[taskframe]
    #[workflow(timeout(10s))]
    fn not_async(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 8. Missing TaskFrameContext as first argument
    #[taskframe]
    #[workflow(delay(1s))]
    async fn no_ctx() -> Result<(), String> { Ok(()) }

    // ── 9. Wrong first argument type (not TaskFrameContext)
    #[taskframe]
    #[workflow(timeout(5s))]
    async fn wrong_ctx(_ctx: &u32) -> Result<(), String> { Ok(()) }

    // ── 10. Wrong return type (not Result<(), E>)
    #[taskframe]
    #[workflow(timeout(5s))]
    async fn wrong_return(_ctx: &TaskFrameContext) -> () {}

    // ── 11. Result first generic is not ()
    #[taskframe]
    #[workflow(timeout(5s))]
    async fn wrong_ok_type(_ctx: &TaskFrameContext) -> Result<u32, String> { Ok(0) }

    // ── 12. Duplicate primitive of the same kind (if validated — timeout twice)
    #[taskframe]
    #[workflow(timeout(10s), timeout(20s))]
    async fn duplicate_timeout(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 13. Positional arg after named arg inside a primitive
    #[taskframe]
    #[workflow(retry(tries = 3, 2s))]
    async fn positional_after_named(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 14. fallback with no frames listed
    #[taskframe]
    #[workflow(fallback())]
    async fn fallback_no_frames(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }

    // ── 15. Workflow attribute applied directly (not through taskframe/task),
    //       confirmed by the proc macro's unconditional error path
    #[workflow(delay(500ms), retry(2))]
    async fn workflow_direct_use(_ctx: &TaskFrameContext) -> Result<(), String> { Ok(()) }
}