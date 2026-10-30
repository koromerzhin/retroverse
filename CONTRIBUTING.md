# Contributing to RETROVERSE

Thanks for your interest. RETROVERSE follows Git Flow.

## Branches

- `main`: stable, publishable releases.
- `develop`: integration branch.
- `feature/*`: new features (branch from `develop`).
- `bugfix/*`: bug fixes on develop.
- `release/*`: release preparation.
- `hotfix/*`: urgent fixes on stable.

## Workflow

1. Fork / branch from `develop`: `git checkout -b feature/my-feature develop`
2. Implement + tests.
3. Run checks: `cargo fmt`, `cargo clippy`, `cargo test`, `npm run build`.
4. Commit with clear messages.
5. Open a PR against `develop`.

## Code style

- Rust: `rustfmt`, clippy warnings treated as errors where reasonable.
- TypeScript: follow existing patterns.
- SQL migrations must never destroy existing data.

## License

By contributing, you agree your contributions are licensed under GPL-3.0-or-later.