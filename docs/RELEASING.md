# Repeatable release procedure

1. Implement and test actual behavior; unsupported modes must fail clearly.
2. Run the exact test suite on the release commit and record results.
3. Update README.md, docs/FEATURES.md, CHANGELOG.md and docs/ROADMAP.md.
4. Check LICENSE, AI_DISCLOSURE.md, third-party dependencies and provenance.
5. Publish complete source and working CI, then create an immutable version tag.
6. Create a GitHub Release using docs/RELEASE_TEMPLATE.md.
7. Include changes, working settings, experimental settings, limitations, setup commands and checksum-verified artifacts.
8. Keep previous release tags intact. Publish a new patch release for corrections.

No claims of native compilation, full self-hosting, security guarantees or passing tests without verifiable evidence.
