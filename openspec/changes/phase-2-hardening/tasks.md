# Tasks

## 1. Design

- [x] 1.1 Write `proposal.md`, the delta specs and `design.md`, and verify `openspec validate phase-2-hardening --strict` passes

## 2. Small fixes (16.9, 16.5, 16.8 sonnet, in parallel)

- [x] 2.1 (16.9) Change the dry run text everywhere (D1), rebuild `docs/spec`, mark S40 accepted. Verify no "change nothing" is left outside archives
- [x] 2.2 (16.5) Draw `⇂` in the drawer (D2). Verify with a test at the layer that owns it, and a deviation row
- [x] 2.3 (16.8) Show the batch refusal as "The command did not start" with its reason (D3). Verify with a unit test that runs on every platform, and a deviation row

## 3. Fixes (16.4, 16.6, 16.7, 16.12 opus, in parallel)

- [x] 3.1 (16.4) Fix the four layout bugs (D4). Verify each with an assertion or image in the test that renders that screen
- [x] 3.2 (16.6) Add recent actions and generators to the empty palette (D5). Verify with the palette test
- [x] 3.3 (16.7) Stop every run on close (D6). Verify with a core test against the fake artisan, and a deviation row for Windows
- [x] 3.4 (16.12) Fix the cause of the flaky settings test (D7). Verify by running the test many times

## 4. Real PHP (16.3 opus)

- [x] 4.1 Add `tests/real_php.rs` and the CI job (D8). Verify by running it once on macOS with PHP 8.4

## 5. Accessibility and images (16.2, 16.1 sonnet)

- [x] 5.1 (16.2) Cover the phase 2 screens in `tests/accessibility.rs`, add the four keys to Settings, Keyboard (D9)
- [ ] 5.2 (16.1) Add the missing images, light and dark, and look at each new or changed one (D10)

## 6. Docs (16.10 sonnet)

- [ ] 6.1 Describe phase 2 in `README.md`; check S32 to S50 against the code, each with how it was checked

## 7. Linux look (16.11 sonnet, only when the screen is unlocked)

- [ ] 7.1 Look at the phase 2 screens on the thinkcentre, or record Linux as looked at only through CI. Record Windows as not run

## 8. Integration

- [ ] 8.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on macOS; all pass
- [ ] 8.2 Run `openspec validate phase-2-hardening --strict`
