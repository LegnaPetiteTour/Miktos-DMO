# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 0.2.x (Phase 1) | ✅ Active development |
| 0.1.x (Phase 0) | ✅ Security patches only |

---

## Scope: What DMO Does (and Does Not Do)

Understanding DMO's threat model helps narrow the scope of relevant vulnerabilities:

**What DMO does:**
- Reads **filesystem metadata only**: path, size, mtime, atime, inode type
- Writes scan results to a **local SQLite database** (`dmo_scan.db`)
- Runs entirely **offline** — no network calls, no telemetry, no cloud sync
- In Phase 0–1: **read-only** — no filesystem modifications of any kind

**What DMO does NOT do:**
- Read file contents
- Make network requests
- Store credentials or personal data
- Execute system commands
- Modify, move, or delete files (until Phase 3 with explicit user approval)

---

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security vulnerabilities privately via GitHub's built-in security advisory system:

1. Go to the [Security tab](https://github.com/LegnaPetiteTour/Miktos-DMO/security) of this repository
2. Click **"Report a vulnerability"**
3. Fill in the advisory form with as much detail as possible

You will receive a response within **72 hours**. If the issue is confirmed, we will coordinate a fix and disclosure timeline with you.

---

## What to Include in a Report

A useful vulnerability report includes:

- **Description**: What is the vulnerability and what impact does it have?
- **Reproduction steps**: Step-by-step instructions to trigger the issue
- **Environment**: macOS version, Rust version, DMO version or git commit
- **Evidence**: Logs, screenshots, or a proof-of-concept (without weaponizing it)
- **Suggested fix**: Optional, but appreciated

---

## Vulnerability Classes of Interest

Given DMO's read-only, offline-first nature, relevant vulnerability classes include:

| Class | Example |
|---|---|
| **Path traversal** | Scanner following symlinks outside the intended scan root |
| **Classifier bypass** | Crafted filename/path that causes a protected file to be classified as waste |
| **SQLite injection** | Malicious filename/path injected into database queries |
| **Privilege escalation** | DMO accessing files outside the user's permission scope |
| **Denial of service** | Crafted directory structure causing infinite recursion or OOM |

Out-of-scope (Phase 0–1): network-based attacks (no network surface), remote code execution via file contents (contents never read).

---

## Disclosure Policy

- We follow **Coordinated Vulnerability Disclosure (CVD)**
- Critical vulnerabilities targeting the classifier's protected-category logic will be treated as P0 and patched within 7 days
- A security advisory will be published on GitHub after the fix is released
- Credit will be given to the reporter unless they prefer to remain anonymous
