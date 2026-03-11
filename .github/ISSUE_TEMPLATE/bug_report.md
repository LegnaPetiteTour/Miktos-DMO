---
name: Bug Report
about: Report incorrect behavior, false positives, crashes, or unexpected output
title: "fix(<scope>): <short description>"
labels: ["bug", "needs-triage"]
assignees: []
---

## Description

A clear description of the bug. What happened, and what did you expect to happen?

## Steps to Reproduce

1. Run command / open app
2. Perform action
3. Observe result

## Expected Behavior

What should have happened?

## Actual Behavior

What actually happened? Include the full output or error message.

```
[paste output or error here]
```

## False Positive (if applicable)

If a file or directory was incorrectly classified as waste:

- **Path**: (relative path, do not share full home directory structure if sensitive)
- **Category assigned**: 
- **Expected category**: (e.g., `Protected / UserDocument`)

## Environment

- **macOS version**: (e.g., macOS 15.3 Sequoia)
- **DMO version / git commit**: (run `git rev-parse --short HEAD`)
- **Rust version**: (run `rustc --version`)
- **App or CLI**: [ ] CLI (`dmo`) [ ] Desktop app (`dmo-app`)

## Additional Context

Any relevant logs, screenshots, or information. For the desktop app, check the terminal where `npm run tauri dev` is running for Rust-side errors.
