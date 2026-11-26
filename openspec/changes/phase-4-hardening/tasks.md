# Tasks

One branch, `change/phase-4-hardening`, in its own worktree. One pull request to `main`.

## 1. Paths

- [x] 1.1 Make `shown` in `app/mod.rs` write a path outside the app whole, with `/` (design D1); verify with a unit test that fails on the old function
- [x] 1.2 Make `findings.rs` use `shown`; verify `cargo test -p mortise-cli` still compares the exact output of shop-api
- [x] 1.3 Resolve `..` by name in `shown`, write UNC and verbatim prefixes as paths that open, and compare components in the push document's `relative`; verify with the unit tests of `app/mod.rs` and `push/mod.rs` (the prefix tests run on Windows only)

## 2. CI

- [x] 2.1 Extend the job "Command line tool" of `ci.yml` (design D2); verify by running its steps in bash on the Mac, and that the path checks fail on a path with a backslash
- [ ] 2.2 Verify the job passes on Linux, macOS and Windows on the pull request

## 3. Screens

- [x] 3.1 Check that `tests/images.rs` stores S5.8 and S5.9 in both themes; verify `cargo test -p mortise-app --test images` passes with no new image
- [x] 3.2 Find the switch of S5.3 in the audit and audit the Accept dialog (design D3); verify `cargo test -p mortise-app --test accessibility` passes

## 4. Docs

- [x] 4.1 Add a Windows runner and a pinned version to the how-to for CI, and the rule for a path outside the app to the reference of the command line tool; verify `npm run build` in `website`
- [x] 4.2 Review the rows of phase 4 in `docs/deviations.md` and add S76; verify each fact against the code or a test
- [x] 4.3 Validate this change with `openspec validate phase-4-hardening --strict`
