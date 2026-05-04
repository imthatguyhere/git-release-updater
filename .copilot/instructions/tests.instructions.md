---
description: Test Driven Development for Rust source
applyTo: "src/**/*.rs"
---

# Test Driven Development Instructions

- When adding new functionality, first write a test that fails due to the missing functionality. Then implement the functionality to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- When fixing a bug, first write a test that reproduces the bug. Then implement the fix to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- When modifying existing functionality, first write a test that captures the expected behavior of the existing functionality. Then implement the modification to make the test pass. Finally, refactor the code as needed while ensuring all tests still pass.
- Tests should be in `.test.rs` files in the same directory as the source code they are testing, and should be organized into modules that mirror the structure of the source code.
- All tests should be run and passing before any change is considered complete.