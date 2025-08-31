#!/bin/bash

# Local Dependency Update Script
# This script provides Dependabot-like functionality for local development
# Supports Rust (Cargo) and Go dependencies

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
INTERACTIVE=true
UPDATE_CARGO=true
UPDATE_GO=true
CREATE_COMMIT=false
BRANCH_PREFIX="deps"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-interactive)
            INTERACTIVE=false
            shift
            ;;
        --cargo-only)
            UPDATE_GO=false
            shift
            ;;
        --go-only)
            UPDATE_CARGO=false
            shift
            ;;
        --commit)
            CREATE_COMMIT=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --dry-run         Show what would be updated without making changes"
            echo "  --no-interactive  Run without prompting for confirmation"
            echo "  --cargo-only      Only update Rust dependencies"
            echo "  --go-only         Only update Go dependencies"
            echo "  --commit          Create git commits for updates"
            echo "  --help            Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if required tools are installed
check_prerequisites() {
    log "Checking prerequisites..."
    
    local missing_tools=()
    
    if $UPDATE_CARGO && ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if $UPDATE_CARGO && ! command -v cargo-outdated &> /dev/null; then
        warn "cargo-outdated not found. Installing..."
        if ! $DRY_RUN; then
            cargo install cargo-outdated
        fi
    fi
    
    if $UPDATE_GO && ! command -v go &> /dev/null; then
        missing_tools+=("go")
    fi
    
    if $CREATE_COMMIT && ! command -v git &> /dev/null; then
        missing_tools+=("git")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    success "All prerequisites satisfied"
}

# Update Rust dependencies
update_cargo_deps() {
    if ! $UPDATE_CARGO; then
        return 0
    fi
    
    log "Checking Rust dependencies for updates..."
    
    # Check for outdated dependencies
    local outdated_output
    if ! outdated_output=$(cargo outdated --workspace --format json 2>/dev/null); then
        warn "Failed to check outdated Cargo dependencies. Falling back to cargo update."
        if $DRY_RUN; then
            log "Would run: cargo update --workspace --dry-run"
        else
            log "Running cargo update..."
            cargo update --workspace
            success "Cargo dependencies updated"
        fi
        return 0
    fi
    
    # Parse outdated dependencies
    local has_updates=false
    if echo "$outdated_output" | grep -q '"dependencies"'; then
        has_updates=true
        log "Found outdated Rust dependencies:"
        echo "$outdated_output" | jq -r '.dependencies[] | "  \(.name): \(.project) -> \(.latest)"' 2>/dev/null || {
            log "Outdated dependencies found (unable to parse JSON output)"
        }
    fi
    
    if $has_updates; then
        if $INTERACTIVE && ! $DRY_RUN; then
            read -p "Update Rust dependencies? [y/N]: " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log "Skipping Rust dependency updates"
                return 0
            fi
        fi
        
        if $DRY_RUN; then
            log "Would update Rust dependencies"
        else
            log "Updating Rust dependencies..."
            cargo update --workspace
            
            if $CREATE_COMMIT; then
                git add Cargo.lock
                git commit -m "chore(deps): update Rust dependencies

Updated by local dependency update script" || warn "Failed to commit Cargo updates"
            fi
            
            success "Rust dependencies updated"
        fi
    else
        success "All Rust dependencies are up to date"
    fi
}

# Update Go dependencies in a specific directory
update_go_deps_in_dir() {
    local dir="$1"
    local name="$2"
    
    if [ ! -f "$dir/go.mod" ]; then
        return 0
    fi
    
    log "Checking Go dependencies in $name ($dir)..."
    
    pushd "$dir" > /dev/null
    
    # Get list of dependencies that can be updated
    local updates_available=false
    local update_output
    if update_output=$(go list -u -m all 2>/dev/null | grep -E '\[.*\]$'); then
        updates_available=true
        log "Found outdated Go dependencies in $name:"
        echo "$update_output" | while read -r line; do
            echo "  $line"
        done
    fi
    
    if $updates_available; then
        if $INTERACTIVE && ! $DRY_RUN; then
            read -p "Update Go dependencies in $name? [y/N]: " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log "Skipping Go dependency updates in $name"
                popd > /dev/null
                return 0
            fi
        fi
        
        if $DRY_RUN; then
            log "Would update Go dependencies in $name"
        else
            log "Updating Go dependencies in $name..."
            go get -u ./...
            go mod tidy
            
            if $CREATE_COMMIT; then
                git add go.mod go.sum
                git commit -m "chore(deps): update Go dependencies in $name

Updated by local dependency update script" || warn "Failed to commit Go updates for $name"
            fi
            
            success "Go dependencies updated in $name"
        fi
    else
        success "All Go dependencies are up to date in $name"
    fi
    
    popd > /dev/null
}

# Update Go dependencies
update_go_deps() {
    if ! $UPDATE_GO; then
        return 0
    fi
    
    # Update Go dependencies in bindings/go
    update_go_deps_in_dir "bindings/go" "Go bindings"
    
    # Update Go dependencies in examples/goclient
    update_go_deps_in_dir "examples/goclient" "Go client example"
}

# Create summary report
create_summary() {
    log "Dependency update summary:"
    
    if $UPDATE_CARGO; then
        if command -v cargo &> /dev/null; then
            echo "  Rust workspace: Updated"
        else
            echo "  Rust workspace: Skipped (cargo not available)"
        fi
    else
        echo "  Rust workspace: Skipped (--go-only specified)"
    fi
    
    if $UPDATE_GO; then
        if command -v go &> /dev/null; then
            echo "  Go bindings: Checked"
            echo "  Go examples: Checked"
        else
            echo "  Go modules: Skipped (go not available)"
        fi
    else
        echo "  Go modules: Skipped (--cargo-only specified)"
    fi
    
    if $DRY_RUN; then
        warn "This was a dry run. No changes were made."
    fi
}

# Main execution
main() {
    echo "🔄 Local Dependency Updater"
    echo "=============================="
    
    check_prerequisites
    
    log "Starting dependency update process..."
    
    # Navigate to project root
    cd "$(dirname "$0")/.."
    
    update_cargo_deps
    update_go_deps
    
    create_summary
    
    success "Dependency update process completed!"
}

# Run main function
main "$@"
