# Run `just --list` for the recipes. The release workflow is in .github/workflows/release.yml.

# The same checks CI runs.
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
    cargo test --all-features
    cargo build --no-default-features

# Cut a release: bump the version, run the checks, then commit, tag and push.
# Pushing the tag publishes to crates.io. Usage: just release 0.2.0
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    tag="v{{version}}"
    if ! [[ "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
        echo "'{{version}}' is not a semver version like 0.2.0" >&2; exit 1
    fi
    if [ "$(git branch --show-current)" != main ]; then
        echo "releases are cut from main" >&2; exit 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        echo "the working tree is not clean" >&2; exit 1
    fi
    git fetch --quiet origin main --tags
    if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
        echo "main is not at origin/main; pull or push first" >&2; exit 1
    fi
    if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
        echo "tag $tag already exists" >&2; exit 1
    fi
    current="$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
    if [ "$current" = "{{version}}" ]; then
        echo "Cargo.toml is already at {{version}}" >&2; exit 1
    fi
    perl -pi -e 's/^version = ".*"/version = "{{version}}"/ if $. < 10' Cargo.toml
    cargo check --quiet --all-features
    just check
    git --no-pager diff --stat
    read -r -p "Commit, tag $tag and push main and the tag (publishes to crates.io)? [y/N] " answer
    if [ "$answer" != y ]; then
        git checkout Cargo.toml
        echo "aborted; nothing was committed or pushed" >&2; exit 1
    fi
    git commit --quiet -am "chore: release $tag"
    git tag -a "$tag" -m "mtgcolor {{version}}"
    git push origin main "$tag"
    echo "pushed $tag; watch https://github.com/jbowes/mtgcolor/actions"
