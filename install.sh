#!/bin/bash
# ABB-Klipper-Middleware Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Default installation directory
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/abb-klipper"

# Parse command line arguments
INSTALL_METHOD="native"  # native, docker, or dev
SKIP_DEPS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --docker)
            INSTALL_METHOD="docker"
            shift
            ;;
        --dev)
            INSTALL_METHOD="dev"
            shift
            ;;
        --skip-deps)
            SKIP_DEPS=true
            shift
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help)
            echo "ABB-Klipper-Middleware Installation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --docker        Install using Docker"
            echo "  --dev           Install for development"
            echo "  --skip-deps     Skip dependency installation"
            echo "  --install-dir   Specify installation directory (default: ~/.local/bin)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

log_info "Starting ABB-Klipper-Middleware installation..."
log_info "Installation method: $INSTALL_METHOD"
log_info "Installation directory: $INSTALL_DIR"

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."
    
    if [[ "$INSTALL_METHOD" == "docker" ]]; then
        if ! command -v docker &> /dev/null; then
            log_error "Docker is required but not installed"
            exit 1
        fi
        if ! command -v docker-compose &> /dev/null; then
            log_error "Docker Compose is required but not installed"
            exit 1
        fi
        log_success "Docker requirements satisfied"
        return
    fi
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 is required but not installed"
        exit 1
    fi
    
    PYTHON_VERSION=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
    if [[ $(echo "$PYTHON_VERSION >= 3.6" | bc -l) -eq 0 ]]; then
        log_error "Python 3.6+ is required, found $PYTHON_VERSION"
        exit 1
    fi
    
    # Check Rust (for native installation)
    if [[ "$INSTALL_METHOD" == "native" ]] || [[ "$INSTALL_METHOD" == "dev" ]]; then
        if ! command -v cargo &> /dev/null; then
            log_warning "Rust/Cargo not found. Installing..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        fi
    fi
    
    log_success "System requirements satisfied"
}

# Install dependencies
install_dependencies() {
    if [[ "$SKIP_DEPS" == true ]]; then
        log_info "Skipping dependency installation"
        return
    fi
    
    log_info "Installing dependencies..."
    
    if [[ "$INSTALL_METHOD" == "native" ]] || [[ "$INSTALL_METHOD" == "dev" ]]; then
        # Install Python dependencies
        pip3 install --user pytest flake8 black mypy
        
        # Install Rust dependencies (handled by Cargo)
        log_info "Rust dependencies will be installed during build"
    fi
    
    log_success "Dependencies installed"
}

# Build and install
install_software() {
    log_info "Building and installing ABB-Klipper-Middleware..."
    
    case $INSTALL_METHOD in
        "docker")
            log_info "Building Docker containers..."
            docker-compose build
            log_success "Docker containers built"
            ;;
        "native")
            log_info "Building native binaries..."
            make all
            
            # Create installation directories
            mkdir -p "$INSTALL_DIR"
            mkdir -p "$CONFIG_DIR"
            
            # Install binaries
            cp target/release/bridge "$INSTALL_DIR/"
            chmod +x "$INSTALL_DIR/bridge"
            
            # Install Python package
            pip3 install --user -e src/converter
            
            # Copy configuration
            cp config.toml "$CONFIG_DIR/"
            
            log_success "Native installation completed"
            ;;
        "dev")
            log_info "Setting up development environment..."
            make converter
            make bridge
            
            # Install development dependencies
            pip3 install --user -e "src/converter[dev]"
            
            log_success "Development environment setup completed"
            ;;
    esac
}

# Create service files
create_service() {
    if [[ "$INSTALL_METHOD" == "native" ]]; then
        log_info "Creating systemd service..."
        
        SERVICE_FILE="$HOME/.config/systemd/user/abb-bridge.service"
        mkdir -p "$(dirname "$SERVICE_FILE")"
        
        cat > "$SERVICE_FILE" << EOF
[Unit]
Description=ABB-Klipper Bridge Service
After=network.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/bridge
Restart=always
RestartSec=5
WorkingDirectory=$CONFIG_DIR

[Install]
WantedBy=default.target
EOF
        
        systemctl --user daemon-reload
        log_success "Systemd service created"
        log_info "To start the service: systemctl --user start abb-bridge"
        log_info "To enable at boot: systemctl --user enable abb-bridge"
    fi
}

# Main installation flow
main() {
    check_requirements
    install_dependencies
    install_software
    create_service
    
    log_success "Installation completed!"
    
    case $INSTALL_METHOD in
        "docker")
            log_info "To start services: docker-compose up -d"
            log_info "To view logs: docker-compose logs -f"
            ;;
        "native")
            log_info "Bridge binary installed to: $INSTALL_DIR/bridge"
            log_info "Configuration file: $CONFIG_DIR/config.toml"
            log_info "Python converter installed globally"
            ;;
        "dev")
            log_info "Development environment ready"
            log_info "Run tests with: make test"
            log_info "Start bridge with: cargo run --bin bridge"
            ;;
    esac
}

# Run main function
main
