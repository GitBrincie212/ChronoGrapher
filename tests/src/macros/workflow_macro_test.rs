use chronographer::prelude::*;

// ── Helpers ──────────────────────────────────────────────────────────────────

macro_rules! assert_workflow_type {
    ($frame:ty, $contains:literal) => {{
        let w = <$frame>::workflow();
        let name = std::any::type_name_of_val(&w);
        assert!(
            name.contains($contains),
            "expected type name to contain {:?}, got {:?}",
            $contains,
            name
        );
    }};
}

// ── Single primitive: timeout ─────────────────────────────────────────────────

#[taskframe]
#[workflow(timeout(20s))]
pub async fn TimeoutFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_timeout_workflow_type() {
    assert_workflow_type!(TimeoutFrame, "TimeoutTaskFrame");
}

#[tokio::test]
async fn test_timeout_single_is_constructible() {
    let _ = TimeoutFrame::single();
}

// ── Single primitive: delay ───────────────────────────────────────────────────

#[taskframe]
#[workflow(delay(500ms))]
pub async fn DelayFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_delay_workflow_type() {
    assert_workflow_type!(DelayFrame, "DelayTaskFrame");
}

// ── Single primitive: retry (count only) ─────────────────────────────────────

#[taskframe]
#[workflow(retry(3))]
pub async fn RetryFrameSimple(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_retry_simple_workflow_type() {
    assert_workflow_type!(RetryFrameSimple, "RetriableTaskFrame");
}

// ── Single primitive: retry (count + delay) ───────────────────────────────────

#[taskframe]
#[workflow(retry(3, 2s))]
pub async fn RetryFrameWithDelay(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_retry_with_delay_workflow_type() {
    assert_workflow_type!(RetryFrameWithDelay, "RetriableTaskFrame");
}

// ── Single primitive: fallback ────────────────────────────────────────────────
//
// FallbackTaskFrame<T, T2> requires T2: TaskFrame<Args = T::Error>.
// The fallback chain propagates errors as arguments:
//   Primary:        Args = (),      Error = String
//   FallbackLevel1: Args = String,  Error = u32   (receives primary's error)
//   FallbackLevel2: Args = u32,     Error = ()    (receives level1's error)
//
// So each fallback frame's first extra arg receives the previous frame's error.

#[taskframe]
pub async fn FallbackLevel2(_ctx: &TaskFrameContext, _err: u32) -> Result<(), String> {
    Ok(())
}

#[taskframe]
pub async fn FallbackLevel1(_ctx: &TaskFrameContext, _err: String) -> Result<(), u32> {
    Ok(())
}

#[taskframe]
#[workflow(fallback(FallbackLevel1, FallbackLevel2))]
pub async fn PrimaryWithFallback(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_fallback_workflow_type() {
    assert_workflow_type!(PrimaryWithFallback, "FallbackTaskFrame");
}

#[tokio::test]
async fn test_fallback_workflow_is_constructible() {
    let _ = PrimaryWithFallback::workflow();
}

// ── Single fallback ───────────────────────────────────────────────────────────

#[taskframe]
pub async fn SingleFallbackHandler(_ctx: &TaskFrameContext, _err: String) -> Result<(), String> {
    Ok(())
}

#[taskframe]
#[workflow(fallback(SingleFallbackHandler))]
pub async fn PrimaryWithSingleFallback(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_single_fallback_workflow_type() {
    assert_workflow_type!(PrimaryWithSingleFallback, "FallbackTaskFrame");
}

// ── Chained primitives ────────────────────────────────────────────────────────

#[taskframe]
#[workflow(
    timeout(20s),
    delay(500ms),
    retry(1)
)]
pub async fn ChainedFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_chained_workflow_compiles() {
    let _ = ChainedFrame::workflow();
}

#[tokio::test]
async fn test_chained_outermost_type() {
    // retry is last so it is the outermost wrapper
    assert_workflow_type!(ChainedFrame, "RetriableTaskFrame");
}

// ── Extra args passed through correctly ──────────────────────────────────────

#[taskframe]
#[workflow(timeout(10s))]
pub async fn FrameWithArgs(
    _ctx: &TaskFrameContext,
    value: u32,
    label: String,
) -> Result<(), String> {
    let _ = (value, label);
    Ok(())
}

#[tokio::test]
async fn test_extra_args_compile() {
    let _ = FrameWithArgs::single();
}

// ── Generic taskframe with workflow ──────────────────────────────────────────

#[taskframe]
#[workflow(delay(1s))]
pub async fn GenericFrame<T: Send + Sync + 'static>(
    _ctx: &TaskFrameContext,
) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_generic_frame_compiles() {
    let _ = GenericFrame::<u32>::single();
    let _ = GenericFrame::<String>::workflow();
}

// ── single() and workflow() are both accessible ───────────────────────────────

#[taskframe]
#[workflow(timeout(5s))]
pub async fn BothMethodsFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_both_methods_accessible() {
    let _single = BothMethodsFrame::single();
    let _workflow = BothMethodsFrame::workflow();
}

// ── Plain #[taskframe] without #[workflow] ────────────────────────────────────

#[taskframe]
pub async fn PlainFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_plain_frame_workflow_is_self() {
    // Without #[workflow(...)], workflow() returns Self (the bare struct)
    let w = PlainFrame::workflow();
    let name = std::any::type_name_of_val(&w);
    assert!(
        name.contains("PlainFrame"),
        "expected PlainFrame in type name, got {name:?}"
    );
}

// ── Threshold primitive ───────────────────────────────────────────────────────

#[taskframe]
#[workflow(threshold(5))]
pub async fn ThresholdFrame(_ctx: &TaskFrameContext) -> Result<(), String> {
    Ok(())
}

#[tokio::test]
async fn test_threshold_workflow_type() {
    assert_workflow_type!(ThresholdFrame, "ThresholdTaskFrame");
}

// ── Trybuild: compile-fail cases ─────────────────────────────────────────────

#[test]
fn workflow_macro_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("ui/workflow_errors.rs");
}