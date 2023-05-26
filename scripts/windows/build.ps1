# Set terminal colors to always enabled.
$Env:CARGO_TERM_COLOR = "always"

# Build the project.
echo "Building the project..." &&
cargo build --release &&
# Test the project.
echo "Testing the project..." &&
cargo test &&
# Make sure cargo-wix is installed.
echo "Installing cargo-wix..." &&
cargo install cargo-wix &&
# Initialize WiX Toolset.
echo "Initializing WiX Toolset..." &&
cargo wix init --force &&
# Build the Windows Installer.
echo "Building the Windows Installer..." &&
cargo wix

# Check the exit code.
$exit_code = $LASTEXITCODE
if ($exit_code -ne 0)
{
    echo "Build failed! ($exit_code)"
}
else
{
    echo "Build succeeded! ($exit_code)"
}
