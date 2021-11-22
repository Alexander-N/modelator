#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

# create a new branch `[release] vX.Y.Z` with ${RELEASE_VERSION}

git checkout -b "release/v${RELEASE_VERSION}"

# update the versions on Rust crate
sed -i "s|^version = \"[^\"]\+\"|version = \"${RELEASE_VERSION}\"|g" "$RUST_DIR/modelator/Cargo.toml"

# update the version on Python module
sed -i "s|^version = \"[^\"]\+\"|version = \"${RELEASE_VERSION}\"|g" "$PYTHON_DIR/pyproject.toml"

# nothing to do for Go lang for now

BODY_FILE="current_changelog"

unclog build -u | sed "s/## Unreleased/## v${RELEASE_VERSION}/g" > "$BODY_FILE"

# unclog hack until https://github.com/informalsystems/unclog/issues/22 closes

echo "release v${RELEASE_VERSION}" > summary.txt

cat > fake_editor <<EOF
#!/bin/sh
cat summary.txt > \$1
EOF

chmod u+x fake_editor

unclog release --editor "./fake_editor" --version "v${RELEASE_VERSION}"

unclog build > CHANGELOG.md

git add ".changelog/v${RELEASE_VERSION}"
git add --update

COMMIT_MSG="[RELEASE] v${RELEASE_VERSION}"

git commit -m "$COMMIT_MSG"
git push origin "release/v${RELEASE_VERSION}"

# create the pull request from `[release] vX.Y.Z` to `main`

gh pr create \
    --title "$COMMIT_MSG" \
    --body-file "$BODY_FILE" \
    --assignee "@me"