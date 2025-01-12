#!/usr/bin/env bash
set -eu -o pipefail

git init -q module1
(cd module1
  touch this
  git add . && git commit -q -m c1
)

git init submodule-with-extra-worktree-host
(cd submodule-with-extra-worktree-host
  git submodule add ../module1 m1
  (cd m1
    git worktree add ../../worktree-of-submodule
  )
)
