## Design

I will switch the design to look like

https://docs.vicinae.com/

Which is a lot nicer. Their website is nicer too. I will also copy

## Other

- [ ] Upgrade to iced 0.14

## File watching

https://github.com/notify-rs

- [ ] For projects and apps, watch the directories FLAT and rescan on change
- [ ] For files, we'll just do it on an interval. Not worth it to watch files recursively and the user won't care if there's a delay

## Next Steps

- [ ] Formalize waycast configuration and make sure it gets passed down
- [ ] Create an icon resolver so I can stop worrying about that shit everywhere
    - [ ] It should have a method that always returns the path to an icon. Pick some sort of default icon that is guaranteed to exist so on a failure we still have something

## Important

Once I figure out how to load the iced app, and hide and show the
window over IPC, I can go back to the "show" command sending an ipc
message to the daemon to show the window.

## Setting up for daemon

- [x] Refactor all plugins to return a standard Item struct instead of the impl LauncherListItem
A standard item struct will also save us from having to use `dyn` everywhere. So no vtable lookups.

## Waycast Facade

- [ ] Getting default list
- [ ] Get all items of a kind
- [ ] Hybrid search function. Just transfer the one from the UI to the facade
- [ ] Icon fetching functionality including cache and all.
    - [ ] Make an icon fetching facade that this will use
    - [ ] Handle failures and everything in there so I don't have to deal with it in the UI
    - [ ] I can also have some better fallbacks. For example if we don't have `image-webp` we could do `image-generic`
- [x] Execute an item based on kind. This will consolidate execute logic instead of it being in the plugins
- [ ] Hold the waycast data reference
- [x] Add actual errors using thiserror
- [ ] Resolve all the configs for all the scanners and bullshit here too

## Reorganizing

- [x] Get rid of the macros crate. No longer needed
- [x] Move scanners to a different crate. Either its own (current) or to a different crate to house similar functionality
- [x] Remove the cache crate since we're using sqlite now
- [ ] Figure out the clusterfuck I have going on with icon resolution. It really shouldn't be that hard

## Database

- [x] Create search index like chat jipity suggested
- [x] Create `search` function in data crate that uses the DB index and then possibly also the nucleo matcher after
- [ ] Make all the waycast-data query methods return Vec<LauncherItem> instead of ItemRow to save wasted effort
- [ ] With configuration, put the database on the xdg data path or local when on dev mode

## UI

- [x] Filtering
- [x] Do more things async like filtering
- [ ] Add a little loading indicator
- [ ] Add sequence number to search requests to ensure ordering

## General

- [x] Figure out how to only do db open once in app.rs
- [x] Improve memory usage. Currently using 1G. Seems like a leak somewhere

## Daemon

- [x] Daemon facade with a run function that contains the main loop.
- [x] Builder pattern for the facade for config purposes
- [ ] Set up directory watching for apps & projects so we can have real time updates instead of relying on the full scan
- [x] Save resolved icon paths to db for quick access from UI. Resolution will be done in daemon for speed
- [x] Figure out a ttl mechanism for icon cache

## Config

- [x] Make sure file search plugin takes a map of paths instead of the "default list" so it's easier to control

## Projects Scanning

- [ ] Make search parallel if multiple dirs
- [ ] Switch to using the `ignore` crate to ignore (maybe)

## User features

- [x] Notify when waycast is ready on boot
- [ ] Notify when waycast restarts after a crash

## Local Dev

- [ ] Formalize local dev environment
    - [ ] Make sure that when on development, we use the `xdg` directory in current dir for everything
    - [ ] Add an init command to the justfile that will
        - Set up devicons
        - Run reset db

## Cleanups

- [x] The waycast-ui crate should contain the UI code only. Make a new `waycast` crate that ties together
the daemon and ui threads. Essentially transfer over the main.rs from waycast-ui to its own crate

## Waycast cli

- [ ] After transfering over to the new `waycast` crate. I should provide some useful commands like:
    - [ ] cache clear
    - [x] version
    - [ ] diagnose (check for issues)
    - [ ] db reset
    - [ ] rescan (if user wants to force it. I could send an IPC message)
    - [ ] show (show the UI. Could just get rid of the ipc show thing)
    - [ ] status (check if the daemon is running. Probably through an IPC hello)

## Searching

I think the way I'll do search is the following:

To take advantage of both nucleo matcher AND sqlite fts

1. Keep both apps and project candidates in memory
2. Files will not be in memory since there can be a lot
3. On search, run nucleo over the app and project entries, then run fts for Files
4. Order results in priority of apps -> projects -> files

## Nix install flake

- [ ] Set up systemd service to start up waycast on boot and relaunch on crash

## Caching

- [x] I can just implement a cache with the database the same way laravel does and use it through the WaycastData facade.
    - I'll have a key, value and ttl column