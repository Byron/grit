#!/bin/bash

set -eu -o pipefail

git init repo

(cd repo
  touch a b c
  git add .
  git commit -m initial

  git worktree add ../wt-1
  git worktree add ../wt-2

  git config extensions.worktreeConfig true
  git config --worktree worktree.setting "set in the main worktree"
)

(cd wt-1
  git config --worktree worktree.setting "set in wt-1"
)

(cd wt-2
  git config --worktree worktree.setting "set in wt-2"
)