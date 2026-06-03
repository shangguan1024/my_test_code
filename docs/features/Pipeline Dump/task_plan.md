# Pipeline Dump Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `dump()` method to `Pipeline` that returns a human-readable debug summary of execution history.

**Architecture:** Single method on existing `Pipeline` struct that iterates `history` and `processors` to format a debug string. No new types, no new dependencies.

**Tech Stack:** Rust, std only

---

### Task 1: Implement `dump()` method on Pipeline

**Files:**
- Modify: `src/pipeline.rs:72-76` (after `history()` method)
- Test: `tests/pipeline_test.rs`

- [ ] **Step 1: Write failing tests for `dump()`**

Add the following tests at the end of `tests/pipeline_test.rs`:

```rust
#[test]
fn test_dump_successful_run() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let mut pipe = pipeline(processor1).chain(processor2);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 5 });
    let _ = pipe.run(data);
    
    let dump = pipe.dump();
    assert!(dump.starts_with("Pipeline[2 processors]:"));
    assert!(dump.contains("Step 1: IncrementProcessor -> Ok (input=Data, output=Data)"));
    assert!(dump.contains("Step 2: DoubleProcessor -> Ok (input=Data, output=Data)"));
}

#[test]
fn test_dump_error_run() {
    let processor1: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let processor2: Arc<dyn Processor> = Arc::new(ErrorProcessor);
    let processor3: Arc<dyn Processor> = Arc::new(DoubleProcessor);
    
    let mut pipe = pipeline(processor1).chain(processor2).chain(processor3);
    let data: Arc<dyn Data> = Arc::new(NumberData { value: 10 });
    let _ = pipe.run(data);
    
    let dump = pipe.dump();
    assert!(dump.starts_with("Pipeline[3 processors]:"));
    assert!(dump.contains("Step 1: IncrementProcessor -> Ok (input=Data, output=Data)"));
    assert!(dump.contains("Step 2: ErrorProcessor -> Err(Processing error: Intentional error)"));
    assert!(dump.contains("Step 3: DoubleProcessor -> [skipped: error short-circuited]"));
}

#[test]
fn test_dump_before_run() {
    let processor: Arc<dyn Processor> = Arc::new(IncrementProcessor);
    let pipe = pipeline(processor);
    
    let dump = pipe.dump();
    assert!(dump.starts_with("Pipeline[1 processors]:"));
    assert!(dump.contains("(not yet executed)"));
}

#[test]
fn test_dump_empty_pipeline() {
    let pipe = Pipeline {
        processors: Vec::new(),
        history: Vec::new(),
    };
    
    let dump = pipe.dump();
    assert!(dump.starts_with("Pipeline[0 processors]:"));
    assert!(dump.contains("(empty)"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_dump`
Expected: FAIL — method `dump` does not exist on `Pipeline`

- [ ] **Step 3: Implement `dump()` method**

Add the following method to `Pipeline` impl block in `src/pipeline.rs`, after the `history()` method (line 76):

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
        for i in self.history.len()..self.processors.len() {
            lines.push(format!("  Step {}: {} -> [skipped: error short-circuited]",
                i + 1, self.processors[i].name()));
        }
    }

    lines.join("\n")
}
```

- [ ] **Step 4: Run all tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/pipeline.rs tests/pipeline_test.rs
git commit -m "feat: add Pipeline::dump() method for debug history summary"
```

---

## Verification Strategy

| Check | Command | Expected Result |
|-------|---------|-----------------|
| All tests pass | `cargo test` | 0 failures, all existing + new tests pass |
| Compiles clean | `cargo build` | No warnings or errors |
| No new deps | Check `Cargo.toml` | `[dependencies]` section empty |
| Dump-specific tests | `cargo test test_dump` | 4 tests pass |