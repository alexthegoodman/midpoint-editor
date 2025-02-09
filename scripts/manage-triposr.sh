#!/usr/bin/env bash

set -e  # Exit on error

# Default configurations
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TRIPOSR_PATH="${SCRIPT_DIR}/triposr"
CONFIG_FILE="${SCRIPT_DIR}/triposr-config.json"
VENV_PATH="${TRIPOSR_PATH}/venv"
LOG_PATH="${SCRIPT_DIR}/logs"
OUTPUT_DIR="${SCRIPT_DIR}/output"
PYTHON_VERSION="3.8"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 could not be found"
        return 1
    fi
    return 0
}

check_python_version() {
    if ! check_command "python3"; then
        log_error "Python 3 is not installed!"
        return 1
    }

    local version=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
    if [ "$(echo -e "${version}\n${PYTHON_VERSION}" | sort -V | head -n1)" = "${PYTHON_VERSION}" ]; then
        log_info "Found Python $version"
        return 0
    else
        log_error "Python version $version found, but version $PYTHON_VERSION or later is required"
        return 1
    fi
}

check_cuda() {
    if ! command -v "nvidia-smi" &> /dev/null; then
        log_warn "NVIDIA GPU driver not found. TripoSR will run in CPU mode (not recommended)"
        return 1
    fi

    local driver_version=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader | head -n1)
    log_info "Found NVIDIA driver version: $driver_version"
    return 0
}

create_config() {
    cat > "$CONFIG_FILE" << EOF
{
    "installPath": "$TRIPOSR_PATH",
    "venvPath": "$VENV_PATH",
    "logPath": "$LOG_PATH",
    "outputDir": "$OUTPUT_DIR",
    "pythonVersion": "$PYTHON_VERSION"
}
EOF
    log_info "Created configuration file at $CONFIG_FILE"
}

install_triposr() {
    # Check Python version
    check_python_version || exit 1

    # Create directories
    mkdir -p "$TRIPOSR_PATH" "$LOG_PATH" "$OUTPUT_DIR"

    # Clone or update repository
    if [ ! -d "${TRIPOSR_PATH}/.git" ]; then
        log_info "Cloning TripoSR repository..."
        git clone https://github.com/VAST-AI-Research/TripoSR.git "$TRIPOSR_PATH"
    else
        log_info "Updating TripoSR repository..."
        (cd "$TRIPOSR_PATH" && git pull)
    fi

    # Create and activate virtual environment
    if [ ! -d "$VENV_PATH" ]; then
        log_info "Creating Python virtual environment..."
        python3 -m venv "$VENV_PATH"
    fi

    # Activate virtual environment
    source "${VENV_PATH}/bin/activate"

    # Upgrade pip and setuptools
    log_info "Upgrading pip and setuptools..."
    python -m pip install --upgrade pip setuptools

    # Install PyTorch based on CUDA availability
    if check_cuda; then
        log_info "Installing PyTorch with CUDA support..."
        # Note: You might want to adjust this based on the specific CUDA version
        python -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118
    else
        log_info "Installing PyTorch without CUDA support..."
        python -m pip install torch torchvision torchaudio
    fi

    # Install requirements
    log_info "Installing dependencies..."
    (cd "$TRIPOSR_PATH" && python -m pip install -r requirements.txt)

    create_config
    log_info "TripoSR installation complete!"
}

run_triposr() {
    local input_image="$1"
    local output_dir="$2"
    local bake_texture="$3"
    local texture_resolution="$4"

    source "${VENV_PATH}/bin/activate"
    
    cd "$TRIPOSR_PATH"
    
    local cmd="python run.py \"$input_image\" --output-dir \"$output_dir\""
    
    if [ "$bake_texture" = "true" ]; then
        cmd="$cmd --bake-texture --texture-resolution $texture_resolution"
    fi
    
    eval "$cmd"
}

check_status() {
    log_info "TripoSR Status:"
    echo "Installation Path: $TRIPOSR_PATH"
    echo "Virtual Environment: $VENV_PATH"
    echo "Output Directory: $OUTPUT_DIR"

    if [ -d "$TRIPOSR_PATH" ]; then
        echo "Installation: Found"
        if [ -d "$VENV_PATH" ]; then
            echo "Virtual Environment: Ready"
        else
            echo "Virtual Environment: Not found"
        fi
    else
        echo "Installation: Not found"
    fi

    check_cuda
}

show_help() {
    cat << EOF
TripoSR Management Script

Usage:
    ./manage-triposr.sh [command] [options]

Commands:
    install         Install or update TripoSR
    status         Check installation status
    run            Process an image with TripoSR
    help           Show this help message

Options for 'run':
    -i, --input    Input image path
    -o, --output   Output directory (default: output)
    -t, --texture  Enable texture baking
    -r, --res      Texture resolution (default: 1024)

Examples:
    # Install TripoSR
    ./manage-triposr.sh install

    # Check status
    ./manage-triposr.sh status

    # Process an image
    ./manage-triposr.sh run -i "examples/chair.png" -o "output" -t -r 2048
EOF
}

# Main script execution
case "$1" in
    "install")
        install_triposr
        ;;
    "status")
        check_status
        ;;
    "run")
        shift
        input_image=""
        output_dir="output"
        bake_texture="false"
        texture_resolution="1024"
        
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -i|--input)
                    input_image="$2"
                    shift 2
                    ;;
                -o|--output)
                    output_dir="$2"
                    shift 2
                    ;;
                -t|--texture)
                    bake_texture="true"
                    shift
                    ;;
                -r|--res)
                    texture_resolution="$2"
                    shift 2
                    ;;
                *)
                    log_error "Unknown option: $1"
                    show_help
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$input_image" ]; then
            log_error "Input image is required"
            show_help
            exit 1
        fi
        
        run_triposr "$input_image" "$output_dir" "$bake_texture" "$texture_resolution"
        ;;
    "help"|"--help"|"-h"|"")
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac