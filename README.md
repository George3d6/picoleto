# Picoleto

*Version: 0.1.1*

Used to keep directories synchronized based on the contents of a "source" directory.

Most of the heavy lifting is done via rsync and ssh.

Designed as a replacement for lsyncd which was randomly going crazy and rsyncing whole directories for no reason.

## State

Seems to be working but I still don't trust it enough to use it in production myself, I want to test it a bit more first, so I would
not recommend anyone use it just yet for anything critical. Currently I just use it as part of my personal files backup routine.

Actively being developed.

## Dependencies

This program is currently linux only since the only library that would have helped me make it corss-comptaible in a not-so-painful way is
https://github.com/passcod/notify and no longer being maintained and doesn't support BSD, as such going with just a thin and actively maintained wrapper over inotify seemed like the more reasonable choice.

You must have: rsync, ssh, mkdir and rm in your $PATH.

## Roadmap

Add alternative config layout to specify either a single target sync dir or an array (alternative is having array all the time, but that's going to look ugly to the user so I'd rather find a way to let the user chose).

Manually test a few more usecases.

Benchmark to make sure there are no leaks or undue usage of resources.

Add some extra configuration (e.g. max number of threads) & change the threading model.

Automated testing for this is quite annoying... so that's a possible todo if I ever reach a stable version
