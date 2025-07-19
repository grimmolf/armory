#!/bin/bash
# Docker-based cross-platform build script

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
OUTPUT_DIR="$PROJECT_ROOT/docker-output"

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
)

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to build Docker image
build_docker_image() {
    print_status "Building Docker cross-compilation image..."
    cd "$PROJECT_ROOT"
    docker build -f Dockerfile.cross -t armory-rust-cross .
    print_success "Docker image built successfully"
}

# Function to build for a specific target
build_target() {
    local target=$1
    print_status "Building for target: $target"
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Run Docker build
    docker run --rm \
        -v "$OUTPUT_DIR:/workspace/output" \
        armory-rust-cross \
        ./docker-build.sh "$target"
    
    print_success "Built $target"
}

# Function to build all targets
build_all() {
    for target in "${TARGETS[@]}"; do
        build_target "$target"
    done
}

# Function to create release packages
create_packages() {
    print_status "Creating release packages..."
    
    cd "$OUTPUT_DIR"
    
    for target in "${TARGETS[@]}"; do
        local binary_name="armory-rust-$target"
        
        if [[ -f "$binary_name" ]]; then
            # Create package directory
            local package_dir="armory-rust-linux-${target#*-unknown-linux-}"
            mkdir -p "$package_dir"
            
            # Copy binary and rename
            cp "$binary_name" "$package_dir/armory-rust"
            chmod +x "$package_dir/armory-rust"
            
            # Copy documentation
            cp "$PROJECT_ROOT/README.md" "$package_dir/"
            cp "$PROJECT_ROOT/LICENSE" "$package_dir/"
            cp "$PROJECT_ROOT/armory-rust/README.md" "$package_dir/RUST_README.md"
            
            # Create installation script
            cat > "$package_dir/install.sh" << 'EOF'
#!/bin/bash
# Armory Rust Installation Script

set -e

echo "🦀 Installing Armory Rust Bitcoin Wallet..."

INSTALL_DIR="${1:-${HOME}/.local/bin}"
mkdir -p "${INSTALL_DIR}"

cp armory-rust "${INSTALL_DIR}/"
chmod +x "${INSTALL_DIR}/armory-rust"

echo "✅ Armory Rust installed to ${INSTALL_DIR}/armory-rust"
echo "Add ${INSTALL_DIR} to your PATH if needed"
echo "Run with: armory-rust --help"
EOF
            
            chmod +x "$package_dir/install.sh"
            
            # Create tar.gz package
            tar -czf "$package_dir.tar.gz" "$package_dir/"
            rm -rf "$package_dir"
            
            print_success "Created package: $package_dir.tar.gz"
        fi
    done
}

# Function to show usage
show_usage() {
    echo "Docker-based Cross-Platform Build Script"
    echo ""
    echo "Usage: $0 [OPTIONS] [TARGET]"
    echo ""
    echo "TARGETS:"
    echo "  x86_64-unknown-linux-gnu    - Standard Linux x86_64"
    echo "  x86_64-unknown-linux-musl   - Static Linux x86_64 (musl)"
    echo "  aarch64-unknown-linux-gnu   - Linux ARM64"
    echo "  all                         - Build all targets"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help          - Show this help"
    echo "  -b, --build-image   - Build Docker image first"
    echo "  -p, --package       - Create release packages"
    echo "  -c, --clean         - Clean output directory"
    echo ""
    echo "EXAMPLES:"
    echo "  $0 --build-image all              # Build image and all targets"
    echo "  $0 x86_64-unknown-linux-gnu       # Build specific target"
    echo "  $0 --package all                  # Build all and create packages"
}

# Parse command line arguments
BUILD_IMAGE=false
CREATE_PACKAGES=false
CLEAN=false
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -b|--build-image)
            BUILD_IMAGE=true
            shift
            ;;
        -p|--package)
            CREATE_PACKAGES=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -*)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            TARGET=$1
            shift
            ;;
    esac
done

# Main execution
main() {
    echo "🐳 Armory Rust Docker Cross-Platform Build"
    echo "========================================="
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        print_error "Docker is required but not installed"
        exit 1
    fi
    
    # Clean output directory if requested
    if [[ "$CLEAN" == true ]]; then
        print_status "Cleaning output directory..."
        rm -rf "$OUTPUT_DIR"
        mkdir -p "$OUTPUT_DIR"
    fi
    
    # Build Docker image if requested or if it doesn't exist
    if [[ "$BUILD_IMAGE" == true ]] || ! docker images armory-rust-cross --format "table {{.Repository}}" | grep -q armory-rust-cross; then
        build_docker_image
    fi
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Build targets
    if [[ -z "$TARGET" ]] || [[ "$TARGET" == "all" ]]; then
        build_all
    else
        # Validate target
        local valid_target=false
        for valid in "${TARGETS[@]}"; do
            if [[ "$TARGET" == "$valid" ]]; then
                valid_target=true
                break
            fi
        done
        
        if [[ "$valid_target" == false ]]; then
            print_error "Invalid target: $TARGET"
            echo "Valid targets: ${TARGETS[*]}"
            exit 1
        fi
        
        build_target "$TARGET"
    fi
    
    # Create packages if requested
    if [[ "$CREATE_PACKAGES" == true ]]; then
        create_packages
    fi
    
    echo ""
    print_success "Build completed! Output in: $OUTPUT_DIR"
    echo ""
    echo "📦 Generated files:"
    ls -la "$OUTPUT_DIR" 2>/dev/null || true
}

# Run main function
main