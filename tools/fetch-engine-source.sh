#!/bin/sh

set -eu

usage() {
    echo "Usage: $0 <engine-name>" >&2
}

if [ "$#" -ne 1 ]; then
    usage
    exit 64
fi

engine_name=$1
case "$engine_name" in
    *[!a-z0-9-]*|'')
        echo "Engine name may contain only lowercase letters, digits and hyphens." >&2
        exit 64
        ;;
esac

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest_dir="$project_root/engines/$engine_name/source"
repository_file="$manifest_dir/repository"
revision_file="$manifest_dir/revision"
paths_file="$manifest_dir/paths"

for required_file in "$repository_file" "$revision_file" "$paths_file"; do
    if [ ! -f "$required_file" ]; then
        echo "Missing manifest file: $required_file" >&2
        exit 66
    fi
done

IFS= read -r repository < "$repository_file"
IFS= read -r revision < "$revision_file"

case "$repository" in
    https://*) ;;
    *)
        echo "Repository must use an official HTTPS URL." >&2
        exit 65
        ;;
esac

case "$revision" in
    *[!0-9a-f]*|'')
        echo "Revision must be a full lowercase Git commit ID." >&2
        exit 65
        ;;
esac

if [ "${#revision}" -ne 40 ] || [ "$revision" = "0000000000000000000000000000000000000000" ]; then
    echo "Revision must be a non-placeholder, 40-character Git commit ID." >&2
    exit 65
fi

if [ ! -s "$paths_file" ]; then
    echo "The paths manifest cannot be empty." >&2
    exit 65
fi

while IFS= read -r source_path || [ -n "$source_path" ]; do
    case "$source_path" in
        ''|/*|..|../*|*/../*|*/..)
            echo "Invalid sparse path: $source_path" >&2
            exit 65
            ;;
    esac
done < "$paths_file"

checkout_root="$project_root/.watra/engine-sources"
checkout_dir="$checkout_root/$engine_name"
mkdir -p "$checkout_root"

if [ ! -d "$checkout_dir/.git" ]; then
    if [ -e "$checkout_dir" ]; then
        echo "Checkout path exists but is not a Git repository: $checkout_dir" >&2
        exit 73
    fi

    git init "$checkout_dir"
    git -C "$checkout_dir" remote add origin "$repository"
    git -C "$checkout_dir" sparse-checkout init --cone
else
    configured_repository=$(git -C "$checkout_dir" remote get-url origin)
    if [ "$configured_repository" != "$repository" ]; then
        echo "Configured origin differs from the manifest; refusing to replace it." >&2
        exit 65
    fi

    if [ -n "$(git -C "$checkout_dir" status --porcelain)" ]; then
        echo "Checkout contains local changes; refusing to overwrite them." >&2
        exit 65
    fi
fi

git -C "$checkout_dir" sparse-checkout set --stdin < "$paths_file"
git -C "$checkout_dir" fetch --depth=1 --filter=blob:none origin "$revision"
git -C "$checkout_dir" checkout --detach FETCH_HEAD

echo "Fetched $engine_name at $revision into $checkout_dir"

