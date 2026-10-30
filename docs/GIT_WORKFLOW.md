# Git workflow

RETROVERSE uses Git Flow.

- `main`: stable, publishable.
- `develop`: integration branch (permanent).
- `feature/*`, `bugfix/*`, `release/*`, `hotfix/*`.

Rules:
- Never commit directly to `main`.
- Feature branches from `develop`.
- Merge only after tests pass.
- Remote: https://github.com/koromerzhin/retroverse