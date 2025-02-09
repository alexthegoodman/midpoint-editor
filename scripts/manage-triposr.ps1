# TripoSR Setup and Management Script
param(
    [Parameter()]
    [switch]$Install,
    [switch]$Start,
    [switch]$Stop,
    [switch]$Status,
    [string]$PythonVersion = "3.8",
    [string]$OutputDir = "output",
    [switch]$CheckGPU
)

$ErrorActionPreference = "Stop"
$TripoSRPath = "$PSScriptRoot\triposr"
$ConfigPath = "$PSScriptRoot\triposr-config.json"
$VenvPath = "$TripoSRPath\venv"
$LogPath = "$PSScriptRoot\logs"

# Create config if it doesn't exist
$defaultConfig = @{
    InstallPath   = $TripoSRPath
    VenvPath      = $VenvPath
    LogPath       = $LogPath
    OutputDir     = $OutputDir
    PythonVersion = $PythonVersion
}

function Initialize-Config {
    if (-not (Test-Path $ConfigPath)) {
        $defaultConfig | ConvertTo-Json | Out-File $ConfigPath
        Write-Host "Created default configuration at $ConfigPath"
    }
    return Get-Content $ConfigPath | ConvertFrom-Json
}

function Test-Command($Command) {
    return [bool](Get-Command -Name $Command -ErrorAction SilentlyContinue)
}

function Test-PythonVersion {
    param($RequiredVersion)
    
    if (-not (Test-Command "python")) {
        Write-Error "Python is not installed! Please install Python $RequiredVersion or later."
        return $false
    }
    
    $version = python -c "import sys; print('.'.join(map(str, sys.version_info[:2])))"
    if ([version]$version -lt [version]$RequiredVersion) {
        Write-Error "Python version $version found, but version $RequiredVersion or later is required."
        return $false
    }
    
    return $true
}

function Test-CUDA {
    if (-not (Test-Command "nvidia-smi")) {
        Write-Warning "NVIDIA GPU driver not found. TripoSR will run in CPU mode (not recommended)."
        return $false
    }
    
    $cudaInfo = nvidia-smi --query-gpu=driver_version --format=csv, noheader
    Write-Host "Found NVIDIA driver version: $cudaInfo"
    return $true
}

function Install-TripoSR {
    $config = Initialize-Config
    
    # Check Python version
    if (-not (Test-PythonVersion $config.PythonVersion)) {
        return
    }
    
    # Create directories
    New-Item -ItemType Directory -Force -Path $config.InstallPath
    New-Item -ItemType Directory -Force -Path $config.LogPath
    New-Item -ItemType Directory -Force -Path $config.OutputDir
    
    Write-Host "Installing TripoSR to $($config.InstallPath)..."
    
    # Clone TripoSR repository
    if (-not (Test-Path "$($config.InstallPath)\.git")) {
        git clone https://github.com/VAST-AI-Research/TripoSR.git $config.InstallPath
    }
    else {
        Push-Location $config.InstallPath
        git pull
        Pop-Location
    }
    
    # Create and activate virtual environment
    Push-Location $config.InstallPath
    if (-not (Test-Path $config.VenvPath)) {
        Write-Host "Creating Python virtual environment..."
        python -m venv $config.VenvPath
    }
    
    # Activate virtual environment
    $activateScript = Join-Path $config.VenvPath "Scripts\Activate.ps1"
    . $activateScript
    
    # Upgrade pip and setuptools
    Write-Host "Upgrading pip and setuptools..."
    python -m pip install --upgrade pip setuptools
    
    # Check CUDA and install appropriate PyTorch version
    $hasCuda = Test-CUDA
    if ($hasCuda) {
        Write-Host "Installing PyTorch with CUDA support..."
        # Note: You might want to adjust this command based on the specific CUDA version
        python -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118
    }
    else {
        Write-Host "Installing PyTorch without CUDA support..."
        python -m pip install torch torchvision torchaudio
    }
    
    # Install requirements
    Write-Host "Installing dependencies..."
    python -m pip install -r requirements.txt
    
    Pop-Location
    Write-Host "TripoSR installation complete!"
}

function Invoke-TripoSR {
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputImage,
        [string]$OutputDir = "output",
        [switch]$BakeTexture,
        [int]$TextureResolution = 1024
    )
    
    $config = Get-Content $ConfigPath | ConvertFrom-Json
    
    # Activate virtual environment
    $activateScript = Join-Path $config.VenvPath "Scripts\Activate.ps1"
    . $activateScript
    
    Push-Location $config.InstallPath
    
    $args = @(
        "run.py",
        $InputImage,
        "--output-dir", $OutputDir
    )
    
    if ($BakeTexture) {
        $args += "--bake-texture"
        $args += "--texture-resolution"
        $args += $TextureResolution
    }
    
    python $args
    
    Pop-Location
}

function Get-TripoSRStatus {
    $config = Get-Content $ConfigPath | ConvertFrom-Json
    
    Write-Host "TripoSR Status:"
    Write-Host "Installation Path: $($config.InstallPath)"
    Write-Host "Virtual Environment: $($config.VenvPath)"
    Write-Host "Output Directory: $($config.OutputDir)"
    
    if (Test-Path $config.InstallPath) {
        Write-Host "Installation: Found"
        if (Test-Path $config.VenvPath) {
            Write-Host "Virtual Environment: Ready"
        }
        else {
            Write-Host "Virtual Environment: Not found"
        }
    }
    else {
        Write-Host "Installation: Not found"
    }
    
    Test-CUDA
}

# Main execution
if ($Install) { Install-TripoSR }
if ($Status) { Get-TripoSRStatus }
if ($CheckGPU) { Test-CUDA }

# If no parameters specified, show help
if (-not ($Install -or $Status -or $CheckGPU)) {
    Write-Host @"
TripoSR Management Script

Usage:
    .\manage-triposr.ps1 [-Install] [-Status] [-CheckGPU]

Options:
    -Install              Install or update TripoSR
    -Status              Check installation status
    -CheckGPU            Check GPU/CUDA availability
    -PythonVersion       Specify Python version (default: 3.8)
    -OutputDir           Specify output directory (default: output)

Example:
    # Install TripoSR
    .\manage-triposr.ps1 -Install

    # Check installation status
    .\manage-triposr.ps1 -Status

    # Process an image
    .\Invoke-TripoSR -InputImage "examples/chair.png" -OutputDir "output" -BakeTexture -TextureResolution 2048
"@
}