#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
exe="${root}/../$exe"

# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"
fixtures="$root/fixtures"

SUCCESSFULLY=0
# WITH_FAILURE=1

function set-static-git-environment() {
  set -a
  export GIT_AUTHOR_DATE="2021-09-09 09:06:03 +0200"
  export GIT_COMMITTER_DATE="${GIT_AUTHOR_DATE}"
  export GIT_AUTHOR_NAME="Sebastian Thiel"
  export GIT_COMMITTER_NAME="${GIT_AUTHOR_NAME}"
  export GIT_AUTHOR_EMAIL="git@example.com"
  export GIT_COMMITTER_EMAIL="${GIT_AUTHOR_EMAIL}"
  set +a
}

function init-git-repo() {
  git init . && git add . && git commit -q -m "initial"
}

title "changelog"
(sandbox
  set-static-git-environment
  export CARGO_HOME=$PWD

  snapshot="$snapshot/triple-depth-workspace-changelog"
  cp -R $fixtures/tri-depth-workspace/* .
  { echo $'target/\n.package-cache' > .gitignore && init-git-repo; } &>/dev/null

  (when "interacting with 'a'"
    (with 'dry-run only'
      it "succeeds" && {
        WITH_SNAPSHOT="$snapshot/a-dry-run-success-multi-crate" \
        expect_run $SUCCESSFULLY "$exe" changelog a --no-preview
      }
    )
    (with '--write'
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" changelog a --write
      }
      (with ".git and target/ directories removed"
        rm -Rf .git/ target/
        it "managed to write a changelog" && {
          expect_snapshot "$snapshot/crate-a-released" .
        }
      )
    )
  )
)

title "smart-release"
(sandbox
  set-static-git-environment
  export CARGO_HOME=$PWD

  snapshot="$snapshot/triple-depth-workspace"
  cp -R $fixtures/tri-depth-workspace/* .
  { echo 'target/' > .gitignore && init-git-repo; } &>/dev/null

  (with "'c'"
    (with '-d minor to bump minor dependencies'
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" smart-release c -d minor
      }
    )
  )
  (when "releasing 'a'"
    (with 'dry-run only'
      (with 'conditional version bumping'
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/a-dry-run-success-multi-crate" \
          expect_run $SUCCESSFULLY "$exe" smart-release a --skip-push --skip-publish -v --allow-dirty -b minor
        }
        (with '--no-multi-crate-release'
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/a-dry-run-success" \
            expect_run $SUCCESSFULLY "$exe" smart-release a --skip-push --skip-publish -v --no-multi-crate-release -b minor
          }
        )
      )
      (with 'unconditional version bumping'
        it "succeeds" && {
          WITH_SNAPSHOT="$snapshot/a-dry-run-success-multi-crate-unconditional" \
          expect_run $SUCCESSFULLY "$exe" smart-release a --skip-push --skip-publish -v --no-bump-on-demand -b minor
        }
        (with '--no-multi-crate-release'
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/a-dry-run-success-unconditional" \
            expect_run $SUCCESSFULLY "$exe" smart-release a --skip-push --skip-publish -v --no-multi-crate-release --no-bump-on-demand -b minor
          }
        )
        (when 'releasing b as well'
          it "succeeds" && {
            WITH_SNAPSHOT="$snapshot/a-b-dry-run-success-multi-crate-unconditional" \
            expect_run $SUCCESSFULLY "$exe" smart-release b a --skip-push --skip-publish -v --no-bump-on-demand -b minor
          }
          (with '--no-multi-crate-release'
            it "succeeds" && {
              WITH_SNAPSHOT="$snapshot/a-b-dry-run-success-unconditional" \
              expect_run $SUCCESSFULLY "$exe" smart-release b a --skip-push --skip-publish -v --no-multi-crate-release --no-bump-on-demand -b minor
            }
          )
        )
      )
    )
    (with '--execute but without side-effects'
      it "succeeds" && {
        expect_run $SUCCESSFULLY "$exe" smart-release a --skip-push --skip-publish --execute --allow-dirty
      }
      (with ".git and target/ directories removed"
        rm -Rf .git/ target/
        it "managed to bump B's minor but left C alone as it's not pre-release anymore" && {
          expect_snapshot "$snapshot/crate-a-released" .
        }
        (with 'unconditional version minor bumping'
          init-git-repo &>/dev/null
          it "succeeds" && {
            expect_run $SUCCESSFULLY "$exe" smart-release -b minor a --skip-push --skip-publish --no-bump-on-demand --execute --allow-dirty
          }
          rm -Rf .git/
          it "managed additionally bumped b but not c as it's not pre-release" && {
            expect_snapshot "$snapshot/crate-a-released-force-bump" .
          }
        )
      )
    )
  )
)

