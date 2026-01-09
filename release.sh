#!/bin/bash
set -e

# Usage:
#   ./release.sh patch   # v0.1.0 -> v0.1.1
#   ./release.sh minor   # v0.1.1 -> v0.2.0
#   ./release.sh major   # v0.2.0 -> v1.0.0

# Get latest semver tag, default to v0.0.0 if none exist
current=$(git tag -l 'v*' --sort=-v:refname | head -n1)
if [ -z "$current" ]; then
    current="v0.0.0"
fi

# Strip 'v' prefix for parsing
version="${current#v}"
IFS='.' read -r major minor patch <<< "$version"

case "$1" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch)
        patch=$((patch + 1))
        ;;
    *)
        echo "Usage: $0 <major|minor|patch>"
        echo "Current version: $current"
        exit 1
        ;;
esac

new_tag="v${major}.${minor}.${patch}"

echo "Current: $current"
echo "New:     $new_tag"
read -p "Create and push tag? [y/N] " confirm

if [[ "$confirm" =~ ^[Yy]$ ]]; then
    git tag "$new_tag"
    git push origin "$new_tag"
    echo "Released $new_tag"
else
    echo "Aborted"
fi
