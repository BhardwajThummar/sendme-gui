#!/bin/bash

# ============================================
# SendMe GUI - Version Management Script
# ============================================
# Industry-standard semantic versioning tool
# Usage: ./scripts/version.sh [major|minor|patch|<version>] [options]
# ============================================

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PACKAGE_JSON="package.json"
CARGO_TOML="src-tauri/Cargo.toml"
TAURI_CONFIG="src-tauri/tauri.conf.json"
CHANGELOG="CHANGELOG.md"

# Help message
show_help() {
    cat << EOF
${BLUE}SendMe GUI - Version Management${NC}

${YELLOW}Usage:${NC}
  ./scripts/version.sh [VERSION_TYPE|VERSION] [OPTIONS]

${YELLOW}Version Types:${NC}
  major       Bump major version (1.2.3 -> 2.0.0)
  minor       Bump minor version (1.2.3 -> 1.3.0)
  patch       Bump patch version (1.2.3 -> 1.2.4)
  X.Y.Z       Set specific version (e.g., 1.5.0)

${YELLOW}Options:${NC}
  --no-git    Skip git operations (tag and commit)
  --dry-run   Show changes without applying them
  -h, --help  Show this help message

${YELLOW}Examples:${NC}
  ./scripts/version.sh patch              # Bump patch version
  ./scripts/version.sh minor              # Bump minor version
  ./scripts/version.sh 2.0.0              # Set to version 2.0.0
  ./scripts/version.sh patch --no-git     # Bump without git operations
  ./scripts/version.sh major --dry-run    # Preview major version bump

EOF
}

# Parse arguments
VERSION_TYPE=""
DRY_RUN=false
NO_GIT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-git)
            NO_GIT=true
            shift
            ;;
        major|minor|patch)
            VERSION_TYPE="$1"
            shift
            ;;
        [0-9]*.[0-9]*.[0-9]*)
            VERSION_TYPE="$1"
            shift
            ;;
        *)
            echo -e "${RED}Error: Unknown argument '$1'${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Validate input
if [ -z "$VERSION_TYPE" ]; then
    echo -e "${RED}Error: Version type or version number required${NC}"
    show_help
    exit 1
fi

# Get current version from package.json
get_current_version() {
    if [ ! -f "$PACKAGE_JSON" ]; then
        echo -e "${RED}Error: $PACKAGE_JSON not found${NC}"
        exit 1
    fi
    node -p "require('./$PACKAGE_JSON').version"
}

# Calculate new version
calculate_version() {
    local current="$1"
    local type="$2"

    IFS='.' read -r major minor patch <<< "$current"

    case "$type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            # Validate semver format
            if [[ ! "$type" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                echo -e "${RED}Error: Invalid version format. Use X.Y.Z${NC}" >&2
                exit 1
            fi
            echo "$type"
            ;;
    esac
}

# Update version in file
update_file_version() {
    local file="$1"
    local new_version="$2"

    case "$file" in
        *.json)
            # Update JSON files
            if [ "$DRY_RUN" = true ]; then
                echo -e "${BLUE}[DRY RUN]${NC} Would update $file to version $new_version"
            else
                # Use node to properly update JSON
                node -e "
                    const fs = require('fs');
                    const data = JSON.parse(fs.readFileSync('$file', 'utf8'));
                    data.version = '$new_version';
                    fs.writeFileSync('$file', JSON.stringify(data, null, 2) + '\n');
                "
                echo -e "${GREEN}✓${NC} Updated $file"
            fi
            ;;
        *.toml)
            # Update TOML files
            if [ "$DRY_RUN" = true ]; then
                echo -e "${BLUE}[DRY RUN]${NC} Would update $file to version $new_version"
            else
                sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$file"
                rm -f "${file}.bak"
                echo -e "${GREEN}✓${NC} Updated $file"
            fi
            ;;
    esac
}

# Update changelog
update_changelog() {
    local new_version="$1"
    local date=$(date +%Y-%m-%d)

    if [ ! -f "$CHANGELOG" ]; then
        echo -e "${YELLOW}Warning: $CHANGELOG not found, skipping${NC}"
        return
    fi

    if [ "$DRY_RUN" = true ]; then
        echo -e "${BLUE}[DRY RUN]${NC} Would update $CHANGELOG for version $new_version"
        return
    fi

    # Check if there's an Unreleased section
    if grep -q "## \[Unreleased\]" "$CHANGELOG"; then
        # Replace Unreleased with new version
        sed -i.bak "s/## \[Unreleased\]/## [${new_version}] - ${date}/" "$CHANGELOG"
        rm -f "${CHANGELOG}.bak"
        echo -e "${GREEN}✓${NC} Updated $CHANGELOG"
    else
        echo -e "${YELLOW}Warning: No [Unreleased] section found in $CHANGELOG${NC}"
    fi
}

# Create git tag
create_git_tag() {
    local version="$1"

    if [ "$NO_GIT" = true ]; then
        echo -e "${YELLOW}Skipping git operations (--no-git flag)${NC}"
        return
    fi

    if [ "$DRY_RUN" = true ]; then
        echo -e "${BLUE}[DRY RUN]${NC} Would create git commit and tag v$version"
        return
    fi

    # Check if git repo exists
    if [ ! -d ".git" ]; then
        echo -e "${YELLOW}Warning: Not a git repository, skipping git operations${NC}"
        return
    fi

    # Stage version files
    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONFIG" "$CHANGELOG" 2>/dev/null || true

    # Create commit
    if git diff --cached --quiet; then
        echo -e "${YELLOW}No changes to commit${NC}"
    else
        git commit -m "chore(release): bump version to ${version}"
        echo -e "${GREEN}✓${NC} Created commit"
    fi

    # Create tag
    git tag -a "v${version}" -m "Release version ${version}"
    echo -e "${GREEN}✓${NC} Created tag v${version}"

    echo ""
    echo -e "${BLUE}To push the release:${NC}"
    echo -e "  git push origin tauri-v2"
    echo -e "  git push origin v${version}"
}

# Main execution
main() {
    echo -e "${BLUE}=== SendMe GUI Version Manager ===${NC}"
    echo ""

    # Get current version
    CURRENT_VERSION=$(get_current_version)
    echo -e "Current version: ${YELLOW}${CURRENT_VERSION}${NC}"

    # Calculate new version
    NEW_VERSION=$(calculate_version "$CURRENT_VERSION" "$VERSION_TYPE")
    echo -e "New version:     ${GREEN}${NEW_VERSION}${NC}"
    echo ""

    # Confirm if not dry run
    if [ "$DRY_RUN" = false ]; then
        read -p "Proceed with version bump? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}Aborted${NC}"
            exit 0
        fi
    fi

    # Update all version files
    echo -e "${BLUE}Updating version files...${NC}"
    update_file_version "$PACKAGE_JSON" "$NEW_VERSION"
    update_file_version "$CARGO_TOML" "$NEW_VERSION"
    update_file_version "$TAURI_CONFIG" "$NEW_VERSION"

    # Update changelog
    update_changelog "$NEW_VERSION"

    # Git operations
    echo ""
    create_git_tag "$NEW_VERSION"

    echo ""
    echo -e "${GREEN}✓ Version bump complete!${NC}"

    if [ "$DRY_RUN" = false ]; then
        echo ""
        echo -e "${BLUE}Next steps:${NC}"
        echo -e "  1. Review the changes"
        echo -e "  2. Run builds: ${YELLOW}yarn build${NC}"
        echo -e "  3. Test the application"
        echo -e "  4. Push to remote: ${YELLOW}git push && git push --tags${NC}"
    fi
}

# Run main function
main
