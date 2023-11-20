# Contributing guide

Contributions are more than welcome!

## Commit messages / PR titles

This project uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
for PR titles, which become the commit messages after a squash merge.

The most important prefixes you should have in mind are:

- `feat:` which represents a new feature, and correlates to a [SemVer](https://semver.org/)
  minor.
- `fix:` which represents bug fixes, and correlates to a SemVer patch.
- `deps:` which represents dependency updates, and correlates to a SemVer patch.
- `feat!:`, or `fix!:`, `refactor!:`, etc., which represent a breaking change
  (indicated by the !).
- `docs:` which represents documentation, and does not correlate to a version bump.
- `chore:` which represents miscellaneous changes,
  and does not correlate to a version bump.

See also the [`release-please`](./.github/workflows/release-please.yml)
for the mapping of commit prefixes to changelog entry.
