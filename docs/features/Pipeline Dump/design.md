# Pipeline Dump Design Document

## Part 1: Overall Architecture

### 1.1 Overview

- **Feature:** Add a `dump()` method to `Pipeline` that generates a human-readable debug summary of pipeline execution history
- **Current behavior:** Users must manually iterate `history()` and downcast each record's fields to see what happened (as shown in `main.rs:98-104`)
- **Target behavior:** `pipeline.dump()` returns a formatted `String` showing all processor steps, their status, and error details
- **Risk level:** LOW — additive API, no existing code changes

### 1.2 Requirements

| REQ-ID | Requirement | Type |
|--------|------------|------|
| REQ-001 | `Pipeline::dump()` method returns `String` summary of execution history | Functional |
| REQ-002 | Each processor step shows: name, status (Ok/Err), and error message if applicable | Functional |
| REQ-003 | Steps after an error are marked `[skipped: error short-circuited]` | Functional |
| REQ-004 | Header line shows total processor count | Functional |
| REQ-005 | Calling `dump()` before `run()` (empty history) produces a valid message | Functional |
| NFR-001 | Zero external dependencies — `std` only | Non-functional |
| NFR-002 | O(n) where n = processor count, not data size | Non-functional |
| NFR-003 | Must not break any existing public API | Non-functional |

**Constraints:**
- `Data` trait has no `Debug` bound — cannot display data values, only presence ("Data")
- No `std::any::type_name` stable API — cannot show concrete type names
- Crate has zero dependencies, must remain zero

### 1.3 Module List

| Module | Responsibility | Owner/Area | Change Type |
|--------|----------------|------------|-------------|
| `pipeline` | Add `dump()` method to `Pipeline` struct | src/pipeline.rs | modified |
| `lib` | Export `dump` if needed (not needed — method on existing type) | src/lib.rs | unchanged |

---

## Part 2: Overall Data Flow and Module Interaction

### 2.1 Data Flow

Single module, single method. No cross-module interaction.

```
Pipeline::dump()
  → iterate self.history (Vec<ProcessorRecord>)
  → format each ProcessorRecord into String line
  → concatenate into final String
  → return String
```

### 2.2 Module Boundary Matrix

Not applicable — single module change with no new module boundaries.

### 2.4 Dependency Constraints

- **Allowed:** `crate::pipeline::ProcessorRecord`, `crate::error::PipelineError` (already imported)
- **Forbidden:** No new external crate dependencies
- **Validation:** `Cargo.toml` `[dependencies]` must remain empty

---

## Part 3: Module Decomposition and Detailed Design

### Module: pipeline (modified)

#### 3.1 Module Overview

- **Business boundary:** Debug/diagnostic output for pipeline execution history
- **Data boundary:** Reads `self.history` and `self.processors.len()`, no data mutation
- **Behavior boundary:** Format history records into human-readable string

#### 3.2 Data Structures

No new data structures. Uses existing:
- `ProcessorRecord` — `processor_name: String`, `input: Arc<dyn Data>`, `output: Option<Arc<dyn Data>>`, `error: Option<PipelineError>`
- `PipelineError` — `TypeError`, `ValidationError`, `ProcessingError`, `EmptyPipeline`

#### 3.3 Public Interfaces

#### Interface: `Pipeline::dump(&self) -> String`

**1. Function Description:**
- Generates a formatted debug string summarizing pipeline execution
- Iterates `self.history` to show each step's processor name, status, and error
- Steps after an error are marked skipped

**2. Use Cases:**

| Scenario | Description | Frequency |
|----------|-------------|-----------|
| Debug after run | Developer calls `dump()` after `run()` fails to diagnose error | High |
| Inspect successful run | Developer calls `dump()` to verify all steps executed correctly | Medium |
| Empty pipeline check | Developer calls `dump()` on new pipeline to see processor list | Low |

**3. Business Logic:**
```
Step 1: Build header line with processor count
        "Pipeline[{count} processors]:"
Step 2: If history is empty and processors exist, show processor names only
Step 3: If history is empty and no processors, show "Pipeline[0 processors]: (empty)"
Step 4: Iterate history, for each record:
        - If output.is_some(): "  Step {i}: {name} -> Ok (input=Data, output=Data)"
        - If error.is_some(): "  Step {i}: {name} -> Err({error})"
Step 5: For processors beyond history length (after error):
        "  Step {i}: {name} -> [skipped: error short-circuited]"
Step 6: Concatenate all lines, return String
Performance: O(n) where n = self.processors.len()
```

**4. Constraints:**

| Constraint | Value | Source | Impact |
|------------|-------|--------|--------|
| No `Debug` on `Data` | N/A | `src/data.rs:4` | Can only show "Data", not values |
| Zero deps | 0 | `Cargo.toml` | Must use `std` only |
| Immutable | `&self` | API design | `dump()` must not mutate state |

**5. Parameters:**

| Parameter | Type | Required | Description | Constraint | Default |
|-----------|------|----------|-------------|------------|---------|
| `&self` | `&Pipeline` | ✅ | Pipeline instance | Must have valid history or processors | N/A |

**6. Return Values:**

| Field | Type | Description | Constraint |
|-------|------|-------------|------------|
| return | `String` | Formatted debug summary | Always non-empty |

**7. Exceptions:**

No exceptions — `dump()` always returns a `String`, even for empty history.

**8. Usage Examples:**

```rust
// Example 1: Dump after successful run
let mut pipe = pipeline(Arc::new(IncrementProcessor)).chain(Arc::new(DoubleProcessor));
let data: Arc<dyn Data> = Arc::new(NumberData { value: 5 });
pipe.run(data)?;
println!("{}", pipe.dump());
// Output:
// Pipeline[2 processors]:
//   Step 1: IncrementProcessor -> Ok (input=Data, output=Data)
//   Step 2: DoubleProcessor -> Ok (input=Data, output=Data)

// Example 2: Dump after error
let mut pipe = pipeline(Arc::new(IncrementProcessor)).chain(Arc::new(ErrorProcessor)).chain(Arc::new(DoubleProcessor));
pipe.run(data)?;
println!("{}", pipe.dump());
// Output:
// Pipeline[3 processors]:
//   Step 1: IncrementProcessor -> Ok (input=Data, output=Data)
//   Step 2: ErrorProcessor -> Err(Processing error: Intentional error)
//   Step 3: DoubleProcessor -> [skipped: error short-circuited]

// Example 3: Dump empty history (before run)
let pipe = pipeline(Arc::new(IncrementProcessor));
println!("{}", pipe.dump());
// Output:
// Pipeline[1 processors]: (not yet executed)
```

#### 3.4.4 Implementation Logic

```rust
pub fn dump(&self) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Pipeline[{} processors]:", self.processors.len()));

    if self.history.is_empty() {
        if self.processors.is_empty() {
            lines.push("  (empty)".to_string());
        } else {
            lines.push("  (not yet executed)".to_string());
        }
    } else {
        for (i, record) in self.history.iter().enumerate() {
            let step = format!("  Step {}: {}", i + 1, record.processor_name);
            if let Some(error) = &record.error {
                lines.push(format!("{} -> Err({})", step, error));
            } else if record.output.is_some() {
                lines.push(format!("{} -> Ok (input=Data, output=Data)", step));
            } else {
                lines.push(format!("{} -> (running)", step));
            }
        }
        let remaining = self.processors.len() - self.history.len();
        for i in self.history.len()..self.processors.len() {
            lines.push(format!("  Step {}: {} -> [skipped: error short-circuited]",
                i + 1, self.processors[i].name()));
        }
    }

    lines.join("\n")
}
```

#### 3.4.5 Test Strategy

| REQ-ID | Test Type | Test File | Expected Evidence |
|--------|-----------|-----------|-------------------|
| REQ-001 | Unit | tests/pipeline_test.rs | `dump()` returns non-empty String |
| REQ-002 | Unit | tests/pipeline_test.rs | Each step shows name + status |
| REQ-003 | Unit | tests/pipeline_test.rs | Skipped steps after error marked correctly |
| REQ-004 | Unit | tests/pipeline_test.rs | Header shows correct processor count |
| REQ-005 | Unit | tests/pipeline_test.rs | Empty history produces valid output |
| NFR-001 | Manual | Cargo.toml | No new deps added |
| NFR-003 | Compile | `cargo build` | Existing API unchanged |

---

## Part 4: Integration and Verification

### 4.1 Integration Points

- `ProcessorRecord` (existing) — `processor_name`, `output`, `error` fields read by `dump()`
- `Processor::name()` (existing) — used to show names of skipped processors
- `PipelineError::Display` (existing) — used to format error messages

### 4.2 Implementation Plan

Single task: Add `dump()` method to `Pipeline` struct in `src/pipeline.rs`, add tests in `tests/pipeline_test.rs`.

### 4.3 Verification Checklist

| REQ-ID | Verification Item | Test Method | Verification Criteria |
|--------|-------------------|-------------|----------------------|
| REQ-001 | `dump()` exists on `Pipeline` | `cargo test` | Method compiles and returns String |
| REQ-002 | Step format includes name + status | `cargo test` | Output contains "Step N: Name -> Ok/Err" |
| REQ-003 | Skipped steps marked | `cargo test` | Output contains "[skipped: error short-circuited]" |
| REQ-004 | Header shows count | `cargo test` | Output starts with "Pipeline[N processors]:" |
| REQ-005 | Empty history handled | `cargo test` | Returns "(not yet executed)" or "(empty)" |
| NFR-001 | No new deps | Check Cargo.toml | [dependencies] empty |
| NFR-002 | O(n) performance | Code review | Iterates processors only, no data cloning |
| NFR-003 | API unchanged | `cargo build` | All existing tests pass |

### 4.4 Change Impact Analysis

- **Impact:** Minimal — additive method only, no existing code touched
- **Risk:** None — no breaking changes
- **Rollback:** Remove `dump()` method and tests