# PowerShell script to download the Whisper v3 Turbo ONNX model

# Create the resources directory if it doesn't exist
$resourcesDir = "$PSScriptRoot\resources"
$onnxDir = "$resourcesDir\onnx"

if (!(Test-Path $resourcesDir)) {
    New-Item -ItemType Directory -Path $resourcesDir
    Write-Host "Created resources directory"
}

if (!(Test-Path $onnxDir)) {
    New-Item -ItemType Directory -Path $onnxDir
    Write-Host "Created onnx directory"
}

# URL for the ONNX model
$modelUrl = "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/main/onnx/model.onnx"
$tokenizerUrl = "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/main/tokenizer.json"

# Download the model
Write-Host "Downloading Whisper v3 Turbo ONNX model..."
Invoke-WebRequest -Uri $modelUrl -OutFile "$onnxDir\model.onnx"
Write-Host "Model downloaded successfully"

# Download the tokenizer
Write-Host "Downloading tokenizer..."
Invoke-WebRequest -Uri $tokenizerUrl -OutFile "$resourcesDir\tokenizer.json"
Write-Host "Tokenizer downloaded successfully"

Write-Host "Setup complete. You can now build and run the application."