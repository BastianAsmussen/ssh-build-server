# Set terminal colors to always enabled.
export CARGO_TERM_COLOR=always

# Build the project.
echo "Building the project..." &&
cargo build --release &&
# Test the project.
echo "Testing the project..." &&
cargo test

# Check the exit code.
exit_code=$?
if [ $exit_code -eq 0 ]; then
  echo "Build succeeded! ($exit_code)"
else
  echo "Build failed! ($exit_code)"
fi
