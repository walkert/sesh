# sesh

A simple utility to create/select tmux sessions based on git project directories.

# Usage

`sesh` takes a single argument which is a path which will be searched for `.git` directories. A max depth of 6 seems to capture the right amount of hits without going too deep. Intended to be used in conjunction with a tmux bind like so:

```
bind e display-popup -E "sesh ~/GIT/work"
```
