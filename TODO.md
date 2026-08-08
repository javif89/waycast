## Container refactor

1. Kill CONFIG_SINGLETON (unchanged — still first).
3. Merge that loop into WaycastApplication, delete daemon/mod.rs, implement Rescan/Stop, move scanners/ and watcher.rs under core/.

Ending structure is three layers with a clean line between them: Waycast = capabilities, no threads and no loops; WaycastApplication = process lifecycle, lock, threads, the one event loop; ui/cmd/socket = consumers. That's the split that keeps the container from turning into the god object I flagged last time — the loop stays out of it.

## Design

I will switch the design to look like

https://docs.vicinae.com/

Which is a lot nicer. Their website is nicer too. I will also copy

## File watching

https://github.com/notify-rs

- [ ] For projects and apps, watch the directories FLAT and rescan on change
- [ ] For files, we'll just do it on an interval. Not worth it to watch files recursively and the user won't care if there's a delay

## Next Steps

- [ ] Formalize waycast configuration and make sure it gets passed down

## UI

- [ ] Add a little loading indicator
- [ ] Add sequence number to search requests to ensure ordering

## Daemon

- Set up directory watching for projects folder

## Projects Scanning

- [ ] Make search parallel if multiple dirs
- [ ] Switch to using the `ignore` crate to ignore (maybe)

## User features

- [ ] Notify when waycast restarts after a crash

## Cleanups

- [x] The waycast-ui crate should contain the UI code only. Make a new `waycast` crate that ties together
the daemon and ui threads. Essentially transfer over the main.rs from waycast-ui to its own crate

## Waycast cli

- [ ] After transfering over to the new `waycast` crate. I should provide some useful commands like:
    - [ ] diagnose (check for issues)
    - [ ] db reset
    - [ ] rescan (if user wants to force it. I could send an IPC message)