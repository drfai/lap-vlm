# Contributing to Lap

Thank you for your interest in contributing to **Lap**!
We welcome all kinds of contributions — bug reports, feature ideas, and code.

This document helps you get started and ensures contributions are efficient and aligned with the project.

---

## Project Philosophy

Lap follows a few core principles:

- **Consistency** – predictable and coherent UX
- **Simplicity** – avoid unnecessary complexity
- **Elegance** – clean and minimal design
- **Performance first** – must scale to large libraries (10k–100k+ files)

If a proposal conflicts with these principles, it is unlikely to be accepted.

---

## How to Contribute

### Reporting Bugs

Before opening an issue:

- Check existing issues
- Try the latest version

Please include:

- OS and version (macOS / Windows / Linux)
- Lap version
- Steps to reproduce
- Expected vs actual behavior
- Sample files (if relevant)

---

### Suggesting Features

- Clearly describe the **problem**, not just the solution
- Explain the **use case**
- Keep proposals aligned with project philosophy

---

### Submitting Code

#### Workflow

1. Fork the repository
2. Create a branch:
   ```bash
   git checkout -b feature/xxx
   ```
3. Make changes
4. Commit:
   ```bash
   git commit -m "feat: xxx"
   ```
5. Open a Pull Request

---

## Pull Request Guidelines

- Keep PRs **small and focused**
- Explain **what** and **why**
- Link related issues (e.g., `Fixes #123`)
- Add screenshots for UI changes
- Ensure the app builds and runs locally

---

## Coding Guidelines

### General

- Prefer clarity over cleverness
- Avoid unnecessary abstractions
- Keep changes minimal and focused

### Rust

- Use idiomatic Rust
- Avoid `panic!` in production paths
- Handle errors explicitly (`Result` / `Option`)

### Frontend (Vue)

- Use `<script setup>`
- Keep components small and composable
- Follow existing Tailwind conventions

---

## Performance & Stability

This is critical for Lap:

- Avoid performance regressions
- Test with large libraries if possible
- Be mindful of memory usage
- File scanning / thumbnail / DB changes require extra care

---

## What Not to Do

- Large, unfocused PRs
- Breaking existing workflows without discussion
- Adding heavy dependencies without strong justification

---

## 💬 Communication

- Be respectful and constructive
- Discuss before large changes
- Feedback is always welcome

---

## Recognition

All contributions are appreciated.  
Active contributors may be invited to become long-term collaborators.

---

Thanks for helping make Lap better ❤️
