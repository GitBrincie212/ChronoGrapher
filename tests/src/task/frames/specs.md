Let me explore the test files in the `frames/` directory.

Searched for files: *
Viewed collectionframe_test.rs:1-201
Viewed condition_taskframe_test.rs:1-124
Viewed delay_taskframe_test.rs:1-142
Viewed dependency_taskframe_test.rs:1-73
Viewed dynamic_taskframe_test.rs:1-64
Viewed fallback_taskframe_test.rs:1-70
Viewed noop_operation_taskframe_test.rs:1-61
Viewed threshold_taskframe_test.rs:1-121
Viewed timeout_taskframe_test.rs:1-62

Here's a comprehensive analysis of edge case gaps across all [frames/](cci:1://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/threshold_taskframe_test.rs:66:0-91:1) test files:

---

## Edge Case Analysis: `tests/src/task/frames/`

### 🔴 Most Missing Edge Cases

#### [fallback_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/fallback_taskframe_test.rs:0:0-0:0) — 1 test only

This is the most under-tested file. It only covers the case where the primary frame fails and the fallback succeeds. Missing:

- **Primary succeeds → fallback is NOT called** (no test for the happy path where fallback is bypassed)
- **Primary fails AND fallback also fails**
- **Fallback increments/side-effects** when primary panics or errors

#### [dependency_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dependency_taskframe_test.rs:0:0-0:0) — 2 tests only

- **Mixed dependencies** (some ok, some failing) → Fails on first error
- **Zero dependencies** (empty [deps](cci:1://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dependency_taskframe_test.rs:26:0-30:1) list) → should always pass through to the inner frame
- **Inner frame failing** when all deps resolve → the counter + error path is never tested together

#### [dynamic_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dynamic_taskframe_test.rs:0:0-0:0) — 3 tests only

- **Closure captures mutated state across multiple executions** → multi-run behavior
- **Error type inspection** → no test that checks the actual error content/value, just `is_err()`

---

### 🟡 Moderate Gaps

#### [condition_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/condition_taskframe_test.rs:0:0-0:0) — 3 tests

- **Truthy condition with a failing inner frame** → only tests the frame succeeds on true; no test for truthy + error
- **Falsey condition + fallback that also fails** → fallback error propagation is untested
- **Truthy condition with no fallback set, but `error_on_false = false`** → the default "skip silently" behavior on false isn't explicitly tested alongside the builder (only [error_on_false(true)](cci:1://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/condition_taskframe_test.rs:96:0-122:1) is checked)

#### [timeout_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/timeout_taskframe_test.rs:0:0-0:0) — 2 tests

- **Task that finishes exactly at the timeout boundary** → boundary/race condition
- **Task errors before timeout** → only "ok before timeout" and "ok after timeout" are tested; a frame that returns [Err](cci:2://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/condition_taskframe_test.rs:16:0-16:32) before the deadline is not covered
- **Zero-duration timeout** → immediate timeout behavior
- **Counter assertion after timeout** — the test checks [counter](cci:1://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dynamic_taskframe_test.rs:35:0-51:1) isn't verified after the timeout test (it's created but `.load()` is never asserted)

#### [collectionframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/collectionframe_test.rs:0:0-0:0) — 8 tests (best covered)

- **`ParallelExecStrategy` with all frames failing** → only the QuitOnSuccess case with one success is tested; all-fail parallel behavior is missing
- **`ParallelExecStrategy::QuitOnFailure`** → not tested at all
- **`SelectionExecStrategy` selecting a failing frame** → only selecting a succeeding frame is tested
- **Empty collection with `SequentialExecStrategy`** → only `SelectionExecStrategy` is tested with an empty list

---

### 🟢 Acceptable Coverage (but minor gaps)

#### [threshold_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/threshold_taskframe_test.rs:0:0-0:0) — 2 active tests

- Has a commented-out `ThresholdErrorReachBehaviour` test (acknowledges the gap with a TODO) DO NOT TOUCH
- Missing: **threshold with count = 1** (boundary), **threshold never reached** (sub-threshold runs), **mixed success/fail frames** with `ThresholdSuccessesCountLogic`

#### [delay_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/delay_taskframe_test.rs:0:0-0:0) — 4 tests

- **Inner frame that fails** with a delay → only `should_fail: false` is used; a delayed failing frame is untested
- **Zero-duration delay** → borderline edge case

---

### Summary Table

| File | Tests | Key Missing Edge Cases |
|---|---|---|
| [fallback_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/fallback_taskframe_test.rs:0:0-0:0) | 1 | Primary succeeds, both fail |
| [dependency_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dependency_taskframe_test.rs:0:0-0:0) | 2 | Mixed deps, zero deps, inner frame failing |
| [dynamic_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dynamic_taskframe_test.rs:0:0-0:0) | 3 | Error content, multi-run state |
| [condition_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/condition_taskframe_test.rs:0:0-0:0) | 3 | Truthy + failing frame, false + failing fallback |
| [timeout_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/timeout_taskframe_test.rs:0:0-0:0) | 2 | Frame errors before timeout, zero-duration, boundary |
| [threshold_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/threshold_taskframe_test.rs:0:0-0:0) | 2 | Threshold=1, sub-threshold, mixed frames |
| [collectionframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/collectionframe_test.rs:0:0-0:0) | 8 | All-fail parallel, selection on failing frame |

**Most urgent to address:** [fallback_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/fallback_taskframe_test.rs:0:0-0:0) and [dependency_taskframe_test.rs](cci:7://file:///home/mepasta/Desktop/github_contrib/ChronoGrapher/tests/src/task/frames/dependency_taskframe_test.rs:0:0-0:0) — both have very thin coverage with obvious untested behaviors.

