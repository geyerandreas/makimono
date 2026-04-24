#!/usr/bin/env bash

# This script updates the version of the makimono project across its components.
# It synchronizes the version in the workspace Cargo.toml and the makimono-node package.json.

set -Eeuo pipefail

usage() {
	echo "Usage: $0 <version>" >&2
	exit 1
}

[ "$#" -eq 1 ] || usage

VERSION=$1

case "$VERSION" in
	*[!0-9A-Za-z.+-]* | "")
		echo "Error: invalid version '$VERSION'" >&2
		exit 1
		;;
esac

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

CARGO_FILE="$ROOT_DIR/Cargo.toml"
PACKAGE_FILE="$ROOT_DIR/crates/makimono-node/package.json"

[ -f "$CARGO_FILE" ] || {
	echo "Error: missing file $CARGO_FILE" >&2
	exit 1
}

[ -f "$PACKAGE_FILE" ] || {
	echo "Error: missing file $PACKAGE_FILE" >&2
	exit 1
}

tmp_cargo=$(mktemp)
tmp_package=$(mktemp)

cleanup() {
	rm -f "$tmp_cargo" "$tmp_package"
}
trap cleanup EXIT INT TERM

awk -v v="$VERSION" '
	BEGIN {
		in_workspace_package = 0
		replaced = 0
	}

	/^\[[^]]+\]/ {
		in_workspace_package = ($0 == "[workspace.package]")
	}

	in_workspace_package && !replaced && /^[[:space:]]*version[[:space:]]*=/ {
		print "version = \"" v "\""
		replaced = 1
		next
	}

	{ print }

	END {
		if (!replaced) {
			print "Error: could not find [workspace.package] version in Cargo.toml" > "/dev/stderr"
			exit 1
		}
	}
' "$CARGO_FILE" > "$tmp_cargo"

awk -v v="$VERSION" '
	BEGIN {
		replaced = 0
	}

	!replaced && /^[[:space:]]*"version"[[:space:]]*:/ {
		sub(/"version"[[:space:]]*:[[:space:]]*"[^"]*"/, "\"version\": \"" v "\"")
		replaced = 1
	}

	{ print }

	END {
		if (!replaced) {
			print "Error: could not find version field in package.json" > "/dev/stderr"
			exit 1
		}
	}
' "$PACKAGE_FILE" > "$tmp_package"

mv "$tmp_cargo" "$CARGO_FILE"
mv "$tmp_package" "$PACKAGE_FILE"

echo "Updated versions to $VERSION"
