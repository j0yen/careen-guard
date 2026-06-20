# careen-guard

When the disk fills past a watermark, careen-guard reclaims space from the Rust `target/` dirs that are too alive for the usual cleaner to touch — by shrinking them in place rather than deleting them.

## Why it exists

A disk-defense tool reaps `target/` dirs whose binaries are stale or uninstalled. But the dirs that grow largest are usually the ones still in use — a 13 GB `recall/target`, a 13 GB `wintermute-brain/target` — and those are exactly the ones a delete-based cleaner can't safely touch. That leaves a gap: the biggest reclaimable space sits behind binaries that are current.

careen-guard fills that gap. On a breach it picks the largest *live* target dirs and runs `careen-sweep` to reclaim inside them (incremental build artifacts, not the current binaries), pairing with `ballast-guard` rather than duplicating it: ballast handles dead dirs, careen handles live ones. Both emit the same `Event` JSON schema, so they read as one disk-defense story.

## How a pass works

`careen-guard run` makes a single SLO evaluation and exits. It reads disk usage on the mount and acts by band:

| Disk usage | Level | Action |
|---|---|---|
| below advisory | `Ok` | nothing swept |
| advisory band | `Warn` | report the reclaimable estimate; delete nothing |
| above high-water | `Breach` | sweep the largest live targets (descending) via `careen-sweep` until usage clears |
| breach, no safe candidates left | `BreachUnresolved` | stop and say so — no panic, no silent exit |

Two safety properties carry from the tools it drives. It never selects a dir whose binary is stale or uninstalled — that is ballast-guard's domain. And it respects `careen-sweep`'s build-lock: a build-locked target is skipped, and the sweep moves to the next candidate. Candidate inventory comes from `careen-survey`; both are invoked as subprocesses (override the binary names with `CAREEN_SWEEP_BIN` / `CAREEN_SURVEY_BIN`).

## Usage

```sh
careen-guard run [--mount /] [--config ~/.config/careen/guard.toml] [--event-sink /var/log/careen.jsonl]
```

Every pass writes one JSON event to stdout; `--event-sink` appends that same line to a file, matching ballast-guard's sink behavior, so a tail of the sink is the full history of passes.

**Exit codes:** `0` = Ok, `2` = Warn, `3` = Breach (resolved), `4` = BreachUnresolved.

## Configuration (`~/.config/careen/guard.toml`)

```toml
high_water_pct = 90
low_water_pct  = 80
advisory_pct   = 85
roots = ["/home/jsy/wintermute"]
```

## Install

```sh
cargo install --path .
# also needs careen-survey and careen-sweep on PATH (see below)
```

## Where it fits

careen-guard is the trigger layer; it drives two siblings and pairs with a third:

- [`careen-survey`](https://github.com/j0yen/careen-survey) — inventories live target dirs and their reclaimable bytes (invoked as a subprocess).
- [`careen-sweep`](https://github.com/j0yen/careen-sweep) — the lock-aware reclaimer that shrinks a target in place (invoked as a subprocess).
- [`ballast-guard`](https://github.com/j0yen/ballast-guard) — handles the dead/stale dirs careen-guard won't touch; the two share one `Event` JSON schema, so their output composes.

## License

MIT OR Apache-2.0 — see `LICENSE-MIT` and `LICENSE-APACHE`.
