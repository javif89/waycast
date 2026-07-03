## 2026-02-07

Got the app entry rescan working on file change. Directory watching is running on its own thread.

Need to figure out how I'm going to architect this thing in an extensible way. The next step
is to also watch the project dirs based on config and rescan those on a new project.

Need to figure out what the correct, efficient, and idiomatic way is to have all these
"watcher" type threads since we have the IPC thread, watcher threads, and daemon threads.

Am I about to rediscover "scheduling"? lol.