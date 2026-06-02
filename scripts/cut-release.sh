#!/usr/bin/env bash
# Two-phase release: --pr bumps the version + opens a release PR; --tag pushes
# the tag (after the PR merges) which triggers .github/workflows/release.yml.
#
#   scripts/cut-release.sh --pr 0.2.0    # bump Cargo.toml, commit on a branch, open PR
#   scripts/cut-release.sh --tag 0.2.0   # tag v0.2.0 on main + push (run after merge)
set -euo pipefail

usage() {
  echo "usage: $0 --pr <version> | --tag <version>" >&2
  exit 2
}

[ $# -eq 2 ] || usage
mode="$1"
version="$2"

case "$mode" in
  --pr)
    branch="release/v${version}"
    git switch -c "$branch"
    # Bump the [package] version (first `version = ` line in Cargo.toml).
    sed -i -E "0,/^version = \"[0-9.]+\"/s//version = \"${version}\"/" Cargo.toml
    cargo build --quiet # refresh Cargo.lock
    git add Cargo.toml Cargo.lock
    git commit -m "release: v${version}"
    git push -u origin "$branch"
    gh pr create --fill --title "release: v${version}" \
      --body "Version bump to v${version}. Merge, then run \`scripts/cut-release.sh --tag ${version}\`."
    ;;
  --tag)
    git switch main
    git pull --ff-only
    git tag -a "v${version}" -m "v${version}"
    git push origin "v${version}"
    echo "Pushed tag v${version} — release.yml will build + publish the GitHub Release."
    ;;
  *)
    usage
    ;;
esac
