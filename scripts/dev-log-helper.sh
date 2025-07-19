#!/bin/bash
# Development Log Helper Script for Armory Project
# Provides utilities for managing automated development logging

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEV_LOG_PATH="$PROJECT_ROOT/armory-rust/docs/DEVELOPMENT_LOG.md"

show_help() {
    cat << EOF
Development Log Helper for Armory Project

Usage: $0 [COMMAND] [OPTIONS]

Commands:
  setup       Set up git hooks for automated logging
  status      Show current logging configuration
  update      Manually update development log with current session
  clean       Remove auto-generated placeholder entries
  test        Test the git hooks without committing

Options:
  -h, --help  Show this help message

Examples:
  $0 setup                    # Set up automated logging
  $0 status                   # Check current configuration
  $0 update "Feature work"    # Manually add log entry
  $0 clean                    # Remove placeholder entries

The git hooks will automatically capture commit information and update
the development log. Claude Code sessions should fill in the details.
EOF
}

setup_hooks() {
    echo "🔧 Setting up development log automation..."
    
    # Check if hooks directory exists
    HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
    if [ ! -d "$HOOKS_DIR" ]; then
        echo "❌ Git hooks directory not found: $HOOKS_DIR"
        exit 1
    fi
    
    # Check if hooks are already installed
    if [ -f "$HOOKS_DIR/post-commit" ] && [ -f "$HOOKS_DIR/pre-commit" ]; then
        echo "✅ Git hooks are already installed"
    else
        echo "❌ Git hooks are missing - please run this script from the project root"
        exit 1
    fi
    
    # Test hooks
    if [ -x "$HOOKS_DIR/post-commit" ] && [ -x "$HOOKS_DIR/pre-commit" ]; then
        echo "✅ Git hooks are executable and ready"
    else
        echo "❌ Git hooks are not executable"
        exit 1
    fi
    
    echo "📝 Development log automation is configured!"
    echo "   - Pre-commit: Code quality checks and reminders"
    echo "   - Post-commit: Automatic log entry creation"
    echo "   - Log file: $DEV_LOG_PATH"
}

show_status() {
    echo "📊 Development Log Automation Status"
    echo ""
    
    # Check hooks
    HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
    echo "Git Hooks:"
    if [ -x "$HOOKS_DIR/pre-commit" ]; then
        echo "  ✅ pre-commit hook: Active"
    else
        echo "  ❌ pre-commit hook: Missing or not executable"
    fi
    
    if [ -x "$HOOKS_DIR/post-commit" ]; then
        echo "  ✅ post-commit hook: Active"
    else
        echo "  ❌ post-commit hook: Missing or not executable"
    fi
    
    echo ""
    
    # Check development log
    echo "Development Log:"
    if [ -f "$DEV_LOG_PATH" ]; then
        echo "  ✅ Log file exists: $DEV_LOG_PATH"
        RECENT_ENTRIES=$(grep -c "### \[" "$DEV_LOG_PATH" 2>/dev/null || echo "0")
        echo "  📝 Total entries: $RECENT_ENTRIES"
        
        # Show recent auto-logged entries
        AUTO_ENTRIES=$(grep -c "Auto-logged" "$DEV_LOG_PATH" 2>/dev/null || echo "0")
        echo "  🤖 Auto-logged entries: $AUTO_ENTRIES"
    else
        echo "  ❌ Log file not found"
    fi
    
    echo ""
    
    # Show recent commits
    echo "Recent Commits:"
    git log --oneline -5 2>/dev/null || echo "  No git history available"
}

manual_update() {
    local DESCRIPTION="$1"
    if [ -z "$DESCRIPTION" ]; then
        read -p "Enter description for this log entry: " DESCRIPTION
    fi
    
    if [ -z "$DESCRIPTION" ]; then
        echo "❌ Description is required"
        exit 1
    fi
    
    # Get current info
    BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M')
    
    # Create manual entry
    TEMP_LOG="/tmp/armory_manual_log.tmp"
    cat > "$TEMP_LOG" << EOF

### [$TIMESTAMP] - [$BRANCH] - [Manual Entry]

**Context:** $DESCRIPTION
**Files Modified:** [To be filled]
**Key Changes:** [To be filled]

**Cross-References:** 
- Manual entry created at $TIMESTAMP
- Branch: $BRANCH

**Implementation Notes:** [To be filled]

---
EOF
    
    # Insert into development log
    if [ -f "$DEV_LOG_PATH" ] && grep -q "## Development Entries" "$DEV_LOG_PATH"; then
        awk '
        /^## Development Entries/ {
            print $0
            print ""
            system("cat '"$TEMP_LOG"'")
            next
        }
        {print}
        ' "$DEV_LOG_PATH" > "${DEV_LOG_PATH}.tmp" && mv "${DEV_LOG_PATH}.tmp" "$DEV_LOG_PATH"
        
        echo "✅ Manual entry added to development log"
        rm -f "$TEMP_LOG"
    else
        echo "❌ Could not update development log"
        rm -f "$TEMP_LOG"
        exit 1
    fi
}

clean_placeholders() {
    echo "🧹 Cleaning placeholder entries from development log..."
    
    if [ ! -f "$DEV_LOG_PATH" ]; then
        echo "❌ Development log not found"
        exit 1
    fi
    
    # Count placeholder entries
    PLACEHOLDER_COUNT=$(grep -c "\[To be filled" "$DEV_LOG_PATH" 2>/dev/null || echo "0")
    
    if [ "$PLACEHOLDER_COUNT" -eq 0 ]; then
        echo "✅ No placeholder entries found"
        exit 0
    fi
    
    echo "📝 Found $PLACEHOLDER_COUNT placeholder markers"
    read -p "Do you want to remove incomplete auto-logged entries? (y/N): " CONFIRM
    
    if [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
        # This is a simple approach - in practice, you might want more sophisticated cleaning
        echo "⚠️  Manual review recommended for cleaning incomplete entries"
        echo "   File: $DEV_LOG_PATH"
        echo "   Search for: '[To be filled' to find placeholder entries"
    else
        echo "ℹ️  Keeping placeholder entries for manual completion"
    fi
}

test_hooks() {
    echo "🧪 Testing git hooks..."
    
    HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
    
    # Test pre-commit hook
    if [ -x "$HOOKS_DIR/pre-commit" ]; then
        echo "Testing pre-commit hook..."
        bash "$HOOKS_DIR/pre-commit"
        echo "✅ Pre-commit hook test completed"
    else
        echo "❌ Pre-commit hook not found or not executable"
    fi
    
    echo ""
    echo "Note: Post-commit hook will run automatically after your next commit"
}

# Main script logic
case "${1:-}" in
    setup)
        setup_hooks
        ;;
    status)
        show_status
        ;;
    update)
        manual_update "$2"
        ;;
    clean)
        clean_placeholders
        ;;
    test)
        test_hooks
        ;;
    -h|--help|help)
        show_help
        ;;
    "")
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use '$0 --help' for usage information"
        exit 1
        ;;
esac