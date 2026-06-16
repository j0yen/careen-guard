# careen-guard

SLO-triggered sweep of live Rust target dirs — the ballast-guard complement for binary-current repos.

## Overview

`ballast-guard` watches the disk SLO and reaps dead target dirs, but structurally cannot touch live dirs (13 GB `recall/target`, 13 GB `wintermute-brain/target`) because their binaries are current and in use. `careen-guard` is the arm that catches exactly those: when disk breaches a watermark, it sweeps the largest **live** target dirs using `careen-sweep`, emitting the same ballast-guard `Event` schema so the two tools compose into one disk-defense story.

## Acceptance Criteria

- **AC1** — Below advisory threshold → single `Ok` JSON event, zero sweep invocations.
- **AC2** — In the advisory band → `Warn` event with non-zero `reclaimable_bytes` estimate, zero deletions.
- **AC3** — On breach → selects live target dirs in descending-size order, invokes `careen-sweep`, emits `Breach` with `bytes_reclaimed > 0` and swept paths in `candidates`.
- **AC4** — When safe candidates are exhausted but usage is still above high-water → `BreachUnresolved` (not a panic, not a silent exit).
- **AC5** — Schema compatibility: careen-guard events deserialize against ballast-guard's `Event` type field-for-field.
- **AC6** — Never selects a dir whose binary is stale/uninstalled (ballast's domain).
- **AC7** — `--event-sink` appends one JSON line per pass, matching ballast-guard's sink behavior.
- **AC8** — Respects `careen-sweep`'s lock-safety: build-locked targets are skipped, sweep continues to next candidate.

## Usage

```sh
careen-guard run [--mount /] [--config ~/.config/careen/guard.toml] [--event-sink /var/log/careen.jsonl]
```

**Exit codes:** `0` = Ok, `2` = Warn, `3` = Breach (resolved), `4` = BreachUnresolved.

## Configuration (`~/.config/careen/guard.toml`)

```toml
high_water_pct = 90
low_water_pct  = 80
advisory_pct   = 85
roots = ["/home/jsy/wintermute"]
```

## Dependencies

- [`careen-survey`](https://github.com/j0yen/careen-survey) — intra-target reclaimable inventory (subprocess)
- [`careen-sweep`](https://github.com/j0yen/careen-sweep) — lock-aware intra-target reclaimer (subprocess)
- Composes with [`ballast-guard`](https://github.com/j0yen/ballast-guard) via a shared `Event` JSON schema

## License

MIT OR Apache-2.0 — see `LICENSE-MIT` and `LICENSE-APACHE`.
