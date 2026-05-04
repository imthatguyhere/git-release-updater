---
description: Test Driven Development for Rust source
applyTo: "src/**/*.rs"
---

# Test Driven Development Instructions

- When adding new functionality, first write a test that fails due to the missing functionality. Then implement the functionality to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- When fixing a bug, first write a test that reproduces the bug. Then implement the fix to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- When modifying existing functionality, first write a test that captures the expected behavior of the existing functionality. Then implement the modification to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- **Public API tests** go in the `tests/` directory as integration tests. Each file tests a single module named `<module>_test.rs` and imports via `use git_release_updater::<module>`.
- **Private function tests** go inline in the source file under `#[cfg(test)] mod tests { use super::*; ... }` in the same file.
- All tests should be run and passing before any change is considered complete.