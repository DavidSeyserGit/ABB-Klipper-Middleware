#!/bin/bash
# Docker management script for ABB-Klipper-Middleware

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose and try again."
        exit 1
    fi
}

# Build the Docker image
build() {
    log_info "Building ABB-Klipper-Middleware Docker image..."
    docker-compose build --no-cache
    log_success "Docker image built successfully!"
}

# Start the services
start() {
    log_info "Starting ABB-Klipper-Middleware services..."
    docker-compose up -d
    log_success "Services started!"
    
    log_info "Waiting for services to be ready..."
    sleep 5
    
    log_info "Service status:"
    docker-compose ps
}

# Stop the services
stop() {
    log_info "Stopping ABB-Klipper-Middleware services..."
    docker-compose down
    log_success "Services stopped!"
}

# View logs
logs() {
    if [ -n "$1" ]; then
        docker-compose logs -f "$1"
    else
        docker-compose logs -f
    fi
}

# Show status
status() {
    log_info "Service status:"
    docker-compose ps
    
    log_info "\nContainer resource usage:"
    docker stats --no-stream $(docker-compose ps -q) 2>/dev/null || log_warning "No running containers found"
}

# Run converter service
convert() {
    if [ $# -eq 0 ]; then
        log_error "Usage: $0 convert <input_file> [output_format]"
        log_info "Example: $0 convert /path/to/file.mod rapid"
        exit 1
    fi
    
    input_file="$1"
    output_format="${2:-rapid}"
    
    if [ ! -f "$input_file" ]; then
        log_error "Input file '$input_file' does not exist"
        exit 1
    fi
    
    # Copy file to input directory
    mkdir -p ./input ./output
    cp "$input_file" ./input/
    filename=$(basename "$input_file")
    
    log_info "Converting $filename with format: $output_format"
    docker-compose run --rm converter-service python3 -c "
from converter import RobotConverter
import sys
converter = RobotConverter('$output_format')
result = converter.process_file('/app/input/$filename')
with open('/app/output/${filename%.mod}_converted.mod', 'w') as f:
    f.write(result)
print('Conversion completed!')
"
    
    log_success "Conversion completed! Output saved to ./output/${filename%.mod}_converted.mod"
}

# Clean up everything
clean() {
    log_warning "This will remove all containers, images, and volumes. Continue? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        docker-compose down -v --rmi all --remove-orphans
        log_success "Cleanup completed!"
    else
        log_info "Cleanup cancelled."
    fi
}

# Show help
help() {
    echo "ABB-Klipper-Middleware Docker Management Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build           Build Docker images"
    echo "  start           Start all services"
    echo "  stop            Stop all services"
    echo "  restart         Restart all services"
    echo "  logs [service]  View logs (optionally for specific service)"
    echo "  status          Show service status and resource usage"
    echo "  convert <file>  Convert ABB file using Docker"
    echo "  clean           Remove all Docker resources (use with caution)"
    echo "  help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build                          # Build the Docker images"
    echo "  $0 start                          # Start the bridge service"
    echo "  $0 logs abb-bridge                # View bridge service logs"
    echo "  $0 convert robot_program.mod      # Convert a robot file"
    echo "  $0 status                         # Check service status"
}

# Main script logic
case "${1:-help}" in
    build)
        check_docker
        build
        ;;
    start)
        check_docker
        start
        ;;
    stop)
        check_docker
        stop
        ;;
    restart)
        check_docker
        stop
        build
        start
        ;;
    logs)
        check_docker
        logs "$2"
        ;;
    status)
        check_docker
        status
        ;;
    convert)
        check_docker
        shift
        convert "$@"
        ;;
    clean)
        check_docker
        clean
        ;;
    help|--help|-h)
        help
        ;;
    *)
        log_error "Unknown command: $1"
        help
        exit 1
        ;;
esac
