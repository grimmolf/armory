#!/bin/bash
# Cross-platform binary build script for Armory Rust
# Builds binaries for macOS, Ubuntu, and Fedora

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARMORY_RUST_DIR="$PROJECT_ROOT/armory-rust"
BUILD_OUTPUT_DIR="$PROJECT_ROOT/build-output"

# Check if we're in the right directory
if [[ ! -f "$ARMORY_RUST_DIR/Cargo.toml" ]]; then
    echo -e "${RED}Error: Could not find armory-rust/Cargo.toml${NC}"
    echo "Please run this script from the project root directory"
    exit 1
fi

# Platform targets
declare -A TARGETS=(
    ["macos-intel"]="x86_64-apple-darwin"
    ["macos-apple-silicon"]="aarch64-apple-darwin"
    ["ubuntu-x86_64"]="x86_64-unknown-linux-gnu"
    ["ubuntu-aarch64"]="aarch64-unknown-linux-gnu"
    ["fedora-x86_64"]="x86_64-unknown-linux-musl"
)

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if target is installed
check_target() {
    local target=$1
    if rustup target list --installed | grep -q "$target"; then
        return 0
    else
        return 1
    fi
}

# Function to install target
install_target() {
    local target=$1
    print_status "Installing Rust target: $target"
    rustup target add "$target"
}

# Function to check cross-compilation dependencies
check_dependencies() {
    local current_os=$(uname -s)
    
    if [[ "$current_os" == "Linux" ]]; then
        # Check for cross-compilation tools on Linux
        if ! command -v gcc-aarch64-linux-gnu &> /dev/null; then
            print_warning "gcc-aarch64-linux-gnu not found. ARM64 builds may fail."
            echo "Install with: sudo apt-get install gcc-aarch64-linux-gnu"
        fi
        
        if ! command -v musl-gcc &> /dev/null; then
            print_warning "musl-gcc not found. musl builds may fail."
            echo "Install with: sudo apt-get install musl-tools"
        fi
    fi
}

# Function to build for a specific target
build_target() {
    local platform=$1
    local target=$2
    
    print_status "Building for $platform ($target)..."
    
    # Check if target is installed
    if ! check_target "$target"; then
        install_target "$target"
    fi
    
    # Set up environment for cross-compilation
    local env_vars=""
    case "$target" in
        "aarch64-unknown-linux-gnu")
            env_vars="CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc"
            ;;
        "x86_64-unknown-linux-musl")
            env_vars="CC=musl-gcc"
            ;;
    esac
    
    # Build the binary
    cd "$ARMORY_RUST_DIR"
    if [[ -n "$env_vars" ]]; then
        env $env_vars cargo build --release --target "$target"
    else
        cargo build --release --target "$target"
    fi
    
    # Create output directory for this platform
    local output_dir="$BUILD_OUTPUT_DIR/armory-rust-$platform"
    mkdir -p "$output_dir"
    
    # Copy binary
    local binary_name="armory-rust"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="armory-rust.exe"
    fi
    
    cp "target/$target/release/$binary_name" "$output_dir/"
    
    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$output_dir/"
    cp "$PROJECT_ROOT/LICENSE" "$output_dir/"
    cp "$ARMORY_RUST_DIR/README.md" "$output_dir/RUST_README.md"
    
    # Create platform-specific installation script
    create_install_script "$output_dir" "$platform"
    
    # Create compressed archive
    cd "$BUILD_OUTPUT_DIR"
    tar -czf "armory-rust-$platform.tar.gz" "armory-rust-$platform/"
    
    print_success "Built $platform binary: $BUILD_OUTPUT_DIR/armory-rust-$platform.tar.gz"
}

# Function to create installation script
create_install_script() {
    local output_dir=$1
    local platform=$2
    
    cat > "$output_dir/install.sh" << 'EOF'
#!/bin/bash
# Armory Rust Installation Script

set -e

echo "🦀 Installing Armory Rust Bitcoin Wallet..."

# Default installation directory
INSTALL_DIR="${HOME}/.local/bin"

# Allow custom installation directory
if [[ -n "$1" ]]; then
    INSTALL_DIR="$1"
fi

# Create directory if it doesn't exist
mkdir -p "${INSTALL_DIR}"

# Copy binary
cp armory-rust "${INSTALL_DIR}/"
chmod +x "${INSTALL_DIR}/armory-rust"

echo "✅ Armory Rust installed to ${INSTALL_DIR}/armory-rust"
echo ""
echo "Add ${INSTALL_DIR} to your PATH if not already done:"
echo "export PATH=\"\$PATH:${INSTALL_DIR}\""
echo ""
echo "You can also add this to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
echo ""
echo "Run with: armory-rust --help"
echo ""
echo "🔧 Available commands:"
echo "  armory-rust wallet create --name my-wallet"
echo "  armory-rust wallet list"
echo "  armory-rust wallet balance --name my-wallet"
echo "  armory-rust transaction create --help"
EOF
    
    chmod +x "$output_dir/install.sh"
}

# Function to build all targets
build_all() {
    local current_os=$(uname -s)
    local available_targets=()
    
    # Determine which targets can be built on the current platform
    case "$current_os" in
        "Darwin")
            available_targets+=("macos-intel" "macos-apple-silicon")
            # Cross-compilation to Linux requires additional setup
            if command -v docker &> /dev/null; then
                print_status "Docker detected - Linux targets available via cross-compilation"
                available_targets+=("ubuntu-x86_64" "fedora-x86_64")
            else
                print_warning "Docker not found - skipping Linux targets"
                print_warning "Install Docker to enable Linux cross-compilation"
            fi
            ;;
        "Linux")
            available_targets+=("ubuntu-x86_64" "ubuntu-aarch64" "fedora-x86_64")
            print_warning "macOS targets not available on Linux"
            ;;
        *)
            print_error "Unsupported OS: $current_os"
            exit 1
            ;;
    esac
    
    print_status "Building for platforms: ${available_targets[*]}"
    
    for platform in "${available_targets[@]}"; do
        local target="${TARGETS[$platform]}"
        build_target "$platform" "$target"
    done
}

# Function to show usage
show_usage() {
    echo "Armory Rust Cross-Platform Build Script"
    echo ""
    echo "Usage: $0 [OPTIONS] [PLATFORM]"
    echo ""
    echo "PLATFORMS:"
    echo "  macos-intel       - macOS Intel (x86_64)"
    echo "  macos-apple-silicon - macOS Apple Silicon (aarch64)"
    echo "  ubuntu-x86_64     - Ubuntu/Debian x86_64"
    echo "  ubuntu-aarch64    - Ubuntu/Debian ARM64"
    echo "  fedora-x86_64     - Fedora/RHEL/CentOS x86_64 (musl)"
    echo "  all               - Build all available platforms"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help        - Show this help"
    echo "  -c, --clean       - Clean build artifacts first"
    echo "  --check-deps      - Check cross-compilation dependencies"
    echo ""
    echo "EXAMPLES:"
    echo "  $0 all                    # Build all available platforms"
    echo "  $0 macos-intel            # Build only for macOS Intel"
    echo "  $0 ubuntu-x86_64          # Build only for Ubuntu x86_64"
    echo "  $0 --clean all            # Clean and build all"
}

# Parse command line arguments
CLEAN_BUILD=false
CHECK_DEPS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -c|--clean)
            CLEAN_BUILD=true
            shift
            ;;
        --check-deps)
            CHECK_DEPS=true
            shift
            ;;
        -*)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            PLATFORM=$1
            shift
            ;;
    esac
done

# Main execution
main() {
    echo "🦀 Armory Rust Cross-Platform Build Script"
    echo "==========================================="
    
    # Check dependencies if requested
    if [[ "$CHECK_DEPS" == true ]]; then
        check_dependencies
        exit 0
    fi
    
    # Clean build artifacts if requested
    if [[ "$CLEAN_BUILD" == true ]]; then
        print_status "Cleaning build artifacts..."
        cd "$ARMORY_RUST_DIR"
        cargo clean
        rm -rf "$BUILD_OUTPUT_DIR"
    fi
    
    # Create build output directory
    mkdir -p "$BUILD_OUTPUT_DIR"
    
    # Check dependencies
    check_dependencies
    
    # Build based on platform argument
    if [[ -z "$PLATFORM" ]] || [[ "$PLATFORM" == "all" ]]; then
        build_all
    elif [[ -n "${TARGETS[$PLATFORM]}" ]]; then
        build_target "$PLATFORM" "${TARGETS[$PLATFORM]}"
    else
        print_error "Unknown platform: $PLATFORM"
        echo ""
        show_usage
        exit 1
    fi
    
    echo ""
    print_success "Build completed! Binaries available in: $BUILD_OUTPUT_DIR"
    echo ""
    echo "📦 Generated files:"
    ls -la "$BUILD_OUTPUT_DIR"/*.tar.gz 2>/dev/null || true
}

# Run main function
main