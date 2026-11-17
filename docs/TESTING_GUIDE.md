# Testing Guide

This guide covers running automated and manual test suites for the Rust MCP Client.

---

## 1. Automated Test Execution

All test suites can be executed using `cargo test` in `rust-client/`:

```bash
cd rust-client
cargo test
```

### Test Suite Breakdown:

| Test File | Total Tests | Focus Area |
|---|---|---|
| `tests/regex_tests.rs` | 57 | Core intent detection, parameters, projection, analytics, batch, undo |
| `tests/context_tests.rs` | 9 | Pronoun resolution, follow-up queries, session isolation |
| `tests/planner_tests.rs` | 8 | Multi-step plans, salary comparison, field projection, validation |
| `tests/suggestion_tests.rs` | 9 | Error classification, entity-preserving recovery, templates |
| `src/intent/fuzzy.rs` | 2 | Levenshtein distance & typo autocorrection |
| **Total Unit Tests** | **85** | **100% Passing** |

---

## 2. Formatting & Linting Verification

Run the following checks to ensure code health and zero clippy warnings:

```bash
cd rust-client

# Check formatting
cargo fmt --check

# Check compilation
cargo check

# Clippy linter with warnings treated as errors
cargo clippy -- -D warnings
```

---

## 3. Manual Acceptance Testing Scenarios

To run the interactive CLI:
```bash
cd rust-client
cargo run
```

Or run in server mode for the full web application:
```bash
cd rust-client
cargo run -- --server --port 5001
```

### Scenario Test Checklist:

1. **Test 1: Prompt Suggestions (Missing Parameter)**
   - **Input**: `Find employee`
   - **Expected**: Explains ID/email is needed, suggests `Find employee 1001`, `Find employee using email...`, `Show all employees`.

2. **Test 2: Unsupported Operation (Delete)**
   - **Input**: `Delete employee 1001`
   - **Expected**: Explains deletion is unsupported, suggests `Find employee 1001`, `Show employee 1001 details`, `Update employee 1001 status to Inactive`.

3. **Test 3: Multi-Step Planning (Multi-Condition)**
   - **Input**: `Find all employees in IT earning more than 150000 and show their names and salaries`
   - **Expected**: Generates multi-step plan, invokes `search_employees`, applies salary post-filter `> 150000`, projects `firstName`, `lastName`, and `salary`.

4. **Test 4: Conversational Context (Pronoun Resolution)**
   - **Turn 1**: `Find employee 1001`
   - **Turn 2**: `What is his salary?`
   - **Expected**: Identifies "his" refers to 1001, retrieves salary for employee 1001.

5. **Test 5: Multiple Follow-Ups**
   - **Turn 1**: `Find employee 1001`
   - **Turn 2**: `What is his department?`
   - **Turn 3**: `Show his email too`
   - **Expected**: Both follow-ups resolve to employee 1001.

6. **Test 6: Context-Aware Planning (In-Memory Filter)**
   - **Turn 1**: `Search employees in IT`
   - **Turn 2**: `Only show their names`
   - **Expected**: Projects the previous search results in memory without a redundant database query.

7. **Test 7: Write Confirmation**
   - **Input**: `Update employee 1001 salary to 180000`
   - **Expected**: Shows the exact parameters and requires explicit confirmation before invoking tool.

8. **Test 8: Error Recovery (Invalid Parameter)**
   - **Input**: `Find employee abc`
   - **Expected**: Explains employee ID must be numeric and suggests valid format.
