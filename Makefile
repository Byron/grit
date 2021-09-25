
help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

always:

##@ Publishing & Versioning

try-publish-all: ## Dry-run publish all crates in the currently set version if they are not published yet.
	cargo run --package cargo-smart-release --bin cargo-smart-release -- smart-release gitoxide

try-bump-minor-version: ## Show how updating the minor version of PACKAGE=<name> would look like.
	cargo run --package cargo-smart-release --bin cargo-smart-release -- smart-release --update-crates-index --bump minor --no-dependencies --skip-publish --skip-tag --skip-push -v $(PACKAGE)

bump-minor-version: ## Similar to try-bump-minor-version, but actually performs the operation on PACKAGE=<name>
	cargo run --package cargo-smart-release --bin cargo-smart-release -- smart-release --update-crates-index --bump minor --no-dependencies --skip-publish --skip-tag --skip-push -v $(PACKAGE) --execute

##@ Release Builds

release-all: release release-lean release-small ## all release builds

release: always ## the default build, big but pretty (builds in ~2min 35s)
	cargo build --release

release-unix: always ## the default build, big but pretty, unix only (builds in ~2min 35s)
	cargo build --release --no-default-features --features max-termion

release-lean: always ## lean and fast, with line renderer (builds in ~1min 30s)
	cargo build --release --no-default-features --features lean

release-light: always ## lean and fast, log only (builds in ~1min 14s)
	cargo build --release --no-default-features --features light

release-small: always ## minimal dependencies, at cost of performance (builds in ~46s)
	cargo build --release --no-default-features --features small

##@ Debug Builds

debug: always ## the default build, big but pretty
	cargo build

debug-unix: always ## the default build, big but pretty, unix only
	cargo build --no-default-features --features max-termion

debug-lean: always ## lean and fast, with line renderer
	cargo build --no-default-features --features lean

debug-light: always ## lean and fast
	cargo build --no-default-features --features light

debug-small: always ## minimal dependencies, at cost of performance
	cargo build --no-default-features --features small

##@ Development

target/release/gix: always
	cargo build --release --no-default-features --features small

nix-shell-macos: ## Enter a nix-shell able to build on macos
	nix-shell -p pkg-config openssl libiconv darwin.apple_sdk.frameworks.Security darwin.apple_sdk.frameworks.SystemConfiguration

##@ Testing

tests: clippy check doc unit-tests journey-tests-small journey-tests-async journey-tests journey-tests-smart-release ## run all tests, including journey tests, try building docs

audit: ## run various auditing tools to assure we are legal and safe
	cargo deny check advisories bans licenses sources


doc: ## Run cargo doc on all crates
	cargo doc
	cargo doc --features=max,lean,light,small

clippy: ## Run cargo clippy on all crates
	cargo clippy --all --tests
	cargo clippy --all --no-default-features --features small
	cargo clippy --all --no-default-features --features light-async --tests

check: ## Build all code in suitable configurations
	cargo check --all
	cargo check --no-default-features --features small
	cargo check --no-default-features --features light
	cargo check --no-default-features --features light-async
	if cargo check --features light-async 2>/dev/null; then false; else true; fi
	cargo check --no-default-features --features lean
	cargo check --no-default-features --features lean-termion
	cargo check --no-default-features --features max
	cargo check --no-default-features --features max-termion
	cd git-actor && cargo check \
				 && cargo check --features local-time-support
	cd gitoxide-core && cargo check \
                     && cargo check --features blocking-client \
                     && cargo check --features async-client \
                     && cargo check --features local-time-support
	cd gitoxide-core && if cargo check --all-features 2>/dev/null; then false; else true; fi
	cd git-hash && cargo check --all-features \
				&& cargo check
	cd git-object && cargo check --all-features \
                  && cargo check --features verbose-object-parsing-errors
	cd git-actor && cargo check --features serde1
	cd git-pack && cargo check --features serde1 \
			   && cargo check --features pack-cache-lru-static \
			   && cargo check --features pack-cache-lru-dynamic \
			   && cargo check --features object-cache-dynamic \
			   && cargo check
	cd git-packetline && cargo check \
					   && cargo check --features blocking-io \
					   && cargo check --features async-io
	cd git-packetline && if cargo check --all-features 2>/dev/null; then false; else true; fi
	cd git-url && cargo check --all-features \
			   && cargo check
	cd git-features && cargo check --all-features \
			   && cargo check --features parallel \
			   && cargo check --features rustsha1 \
			   && cargo check --features fast-sha1 \
			   && cargo check --features progress \
			   && cargo check --features time \
			   && cargo check --features io-pipe \
			   && cargo check --features crc32 \
			   && cargo check --features zlib \
			   && cargo check --features zlib,zlib-ng-compat \
			   && cargo check --features cache-efficiency-debug
	cd git-commitgraph && cargo check --all-features \
			   && cargo check
	cd git-config && cargo check --all-features \
				 && cargo check
	cd git-transport && cargo check \
					 && cargo check --features blocking-client \
					 && cargo check --features async-client \
					 && cargo check --features http-client-curl
	cd git-transport && if cargo check --all-features 2>/dev/null; then false; else true; fi
	cd git-protocol && cargo check \
					&& cargo check --features blocking-client \
					&& cargo check --features async-client
	cd git-protocol && if cargo check --all-features 2>/dev/null; then false; else true; fi
	cd git-repository && cargo check --no-default-features --features local \
					  && cargo check --no-default-features --features async-network-client \
					  && cargo check --no-default-features --features blocking-network-client \
					  && cargo check --no-default-features --features blocking-network-client,blocking-http-transport \
					  && cargo check --no-default-features --features one-stop-shop \
					  && cargo check --no-default-features --features max-performance \
					  && cargo check --no-default-features
	cd cargo-smart-release && cargo check --all
	cd experiments/object-access && cargo check
	cd experiments/diffing && cargo check
	cd experiments/traversal && cargo check

unit-tests: ## run all unit tests
	cargo test --all
	cd git-features && cargo test && cargo test --all-features
	cd git-ref && cargo test --all-features
	cd git-odb && cargo test && cargo test --all-features
	cd git-object && cargo test && cargo test --features verbose-object-parsing-errors
	cd git-pack && cargo test --features internal-testing-to-avoid-being-run-by-cargo-test-all \
				&& cargo test --features "internal-testing-git-features-parallel"
	cd git-packetline && cargo test \
					  && cargo test --features blocking-io,maybe-async/is_sync --test blocking-packetline \
					  && cargo test --features "async-io" --test async-packetline
	cd git-transport && cargo test \
					 && cargo test --features http-client-curl,maybe-async/is_sync \
					 && cargo test --features async-client
	cd git-protocol && cargo test --features blocking-client \
					&& cargo test --features async-client \
					&& cargo test
	cd gitoxide-core && cargo test --lib

continuous-unit-tests: ## run all unit tests whenever something changes
	watchexec -w src $(MAKE) unit-tests

jtt = target/debug/jtt
journey-tests: always  ## run journey tests (max)
	cargo build
	cargo build --package git-testtools --bin jtt
	./tests/journey.sh target/debug/gix target/debug/gixp $(jtt) max

journey-tests-small: always ## run journey tests (lean-cli)
	cargo build --no-default-features --features small
	cd tests/tools && cargo build
	./tests/journey.sh target/debug/gix target/debug/gixp $(jtt) small

journey-tests-async: always ## run journey tests (light-async)
	cargo build --no-default-features --features light-async
	cd tests/tools && cargo build
	./tests/journey.sh target/debug/gix target/debug/gixp $(jtt) async

journey-tests-smart-release:
	cargo build --package cargo-smart-release
	cd cargo-smart-release && ./tests/journey.sh ../target/debug/cargo-smart-release

continuous-journey-tests: ## run stateless journey tests whenever something changes
	watchexec $(MAKE) journey-tests

clear-cache: ## Remove persisted results of test-repositories, they regenerate automatically
	-find . -path "*fixtures/generated" -type d -exec rm -Rf \{\} \;

rust_repo = tests/fixtures/repos/rust.git
$(rust_repo):
	mkdir -p $@
	cd $@ && git init --bare && git remote add origin https://github.com/rust-lang/rust && git fetch

linux_repo = tests/fixtures/repos/linux.git
$(linux_repo):
	mkdir -p $@
	cd $@ && git init --bare && git remote add origin https://github.com/torvalds/linux && git fetch

test_many_commits_1m_repo = tests/fixtures/repos/test-many-commits-1m.git
$(test_many_commits_1m_repo):
	mkdir -p $@
	cd $@ && git init --bare && git remote add origin https://github.com/cirosantilli/test-many-commits-1m.git && git fetch

## get all non-rc tags up to v5.8, oldest tag first (should have 78 tags)
## -> convert to commit ids
## -> write a new incremental commit-graph file for each commit id
tests/fixtures/commit-graphs/linux/long-chain: $(linux_repo)
	mkdir -p $@
	rm -rf $(linux_repo)/objects/info/*graph*
	set -x && cd $(linux_repo) && \
		for tag in $$(git tag --list --merged v5.8 --sort=version:refname | grep -Fv -- -rc); do \
			git show-ref -s "$$tag" | git commit-graph write --split=no-merge --stdin-commits; \
		done
	mv -f $(linux_repo)/objects/info/*graphs* $@
	actual=$$(ls -1 $@/commit-graphs/*.graph | wc -l); \
		if [ $$actual -ne 78 ]; then echo expected 78 commit-graph files, got $$actual; exit 1; fi

tests/fixtures/commit-graphs/linux/single-file: $(linux_repo)
	mkdir -p $@
	rm -rf $(linux_repo)/objects/info/*graph*
	cd $(linux_repo) && git show-ref -s v5.8 | git commit-graph write --stdin-commits
	mv -f $(linux_repo)/objects/info/*graph* $@

tests/fixtures/commit-graphs/rust/single-file: $(rust_repo)
	mkdir -p $@
	rm -rf $(rust_repo)/objects/info/*graph*
	cd $(rust_repo) && git fetch --tags && git show-ref -s 1.47.0 | git commit-graph write --stdin-commits
	mv -f $(rust_repo)/objects/info/*graph* $@

tests/fixtures/commit-graphs/test-many-commits-1m/single-file: $(test_many_commits_1m_repo)
	mkdir -p $@
	rm -rf $(test_many_commits_1m_repo)/objects/info/*graph*
	cd $(test_many_commits_1m_repo) \
		&& echo f4d21576c13d917e1464d9bc1323a560a5b8595d | git commit-graph write --stdin-commits
	mv -f $(test_many_commits_1m_repo)/objects/info/*graph* $@

commit_graphs = \
	tests/fixtures/commit-graphs/linux/long-chain \
	tests/fixtures/commit-graphs/linux/single-file \
	tests/fixtures/commit-graphs/rust/single-file \
	tests/fixtures/commit-graphs/test-many-commits-1m/single-file

##@ on CI

stress: ## Run various algorithms on big repositories
	$(MAKE) -j3 $(linux_repo) $(rust_repo) release-lean
	time ./target/release/gixp --verbose pack-verify --re-encode $(linux_repo)/objects/pack/*.idx
	rm -Rf out; mkdir out && time ./target/release/gixp --verbose pack-index-from-data -p $(linux_repo)/objects/pack/*.pack out/
	time ./target/release/gixp --verbose pack-verify out/*.idx

	time ./target/release/gixp --verbose pack-verify --statistics $(rust_repo)/objects/pack/*.idx
	time ./target/release/gixp --verbose pack-verify --algorithm less-memory $(rust_repo)/objects/pack/*.idx
	time ./target/release/gixp --verbose pack-verify --re-encode $(rust_repo)/objects/pack/*.idx
	# We must ensure there is exactly one pack file for the pack-explode *.idx globs to work.
	git repack -Ad
	time ./target/release/gixp --verbose pack-explode .git/objects/pack/*.idx

	rm -Rf delme; mkdir delme && time ./target/release/gixp --verbose pack-explode .git/objects/pack/*.idx delme/

	$(MAKE) stress-commitgraph
	$(MAKE) bench-git-config

.PHONY: stress-commitgraph
stress-commitgraph: release-lean $(commit_graphs)
	set -x; for path in $(wordlist 2, 999, $^); do \
		time ./target/release/gixp --verbose commit-graph-verify $$path; \
	done

.PHONY: bench-git-config
bench-git-config:
	cd git-config && cargo bench

##@ Maintenance

baseline_asset_dir = git-repository/src/assets/baseline-init
baseline_asset_fixture = tests/fixtures/baseline-init

$(baseline_asset_fixture):
	mkdir -p $@
	cd "$@" && git init --bare && \
		sed -i '' -E '/bare = true|ignorecase = true|precomposeunicode = true|filemode = true/d' config && \
		sed -i '' 's/master/main/g' $$(find . -type f)

transport_fixtures = git-transport/tests/fixtures
base_url = https://github.com/Byron/gitoxide.git
update-curl-fixtures: ## use curl to fetch raw fixtures for use in unit test. Changes there might break them
	curl -D - -L "$(base_url)/info/refs?service=git-upload-pack"  > $(transport_fixtures)/v1/http-handshake.response
	curl -D - -H 'Git-Protocol: version=2' -L "$(base_url)/info/refs?service=git-upload-pack"  > $(transport_fixtures)/v2/http-handshake.response
	curl -H 'User-Agent: git/oxide-0.1.0' -D - -H 'Git-Protocol: version=1' -L "https://github.com/Byron/foo/info/refs?service=git-upload-pack"  > $(transport_fixtures)/http-401.response
	curl -D - -H 'Git-Protocol: version=1' -L "https://github.com/Byron/gitoxide/info/refs?service=git-upload-pack"  > $(transport_fixtures)/http-404.response

update-assets: $(baseline_asset_fixture) ## refresh assets compiled into the binaries from their source
	-rm -Rf $(baseline_asset_dir)
	mkdir -p $(dir $(baseline_asset_dir))
	cp -R $< $(baseline_asset_dir)

force-update-assets: ## As update-assets, but will run git to update the baseline as well
	-rm -Rf $(baseline_asset_fixture)
	$(MAKE) update-assets

check-size: ## Run cargo-diet on all crates to see that they are still in bound
	./etc/check-package-size.sh

fmt: ## run nightly rustfmt for its extra features, but check that it won't upset stable rustfmt
	cargo +nightly fmt --all -- --config-path rustfmt-nightly.toml
	cargo +stable fmt --all -- --check
