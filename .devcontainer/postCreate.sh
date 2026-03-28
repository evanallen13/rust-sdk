#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing GitHub Copilot CLI extension..."
gh extension install github/gh-copilot 2>/dev/null || echo "gh-copilot extension already installed or unavailable (authenticate with 'gh auth login' first)"

# Create a 'copilot' wrapper script in PATH so tools can call it directly.
# The SDK respects the COPILOT_CLI_PATH env var as an alternative.
COPILOT_WRAPPER="/usr/local/bin/copilot"
if [ ! -f "$COPILOT_WRAPPER" ]; then
  echo "==> Creating 'copilot' wrapper at $COPILOT_WRAPPER..."
  sudo tee "$COPILOT_WRAPPER" > /dev/null <<'EOF'
#!/usr/bin/env bash
# Thin wrapper so 'copilot' resolves to 'gh copilot' in PATH.
exec gh copilot "$@"
EOF
  sudo chmod +x "$COPILOT_WRAPPER"
fi

echo ""
echo "============================================================"
echo " Setup complete!"
echo ""
echo " Next steps:"
echo "   1. Authenticate with GitHub:  gh auth login"
echo "   2. Build the SDK:             cargo build"
echo "   3. Run the quick-start:       cargo run --example quick_start"
echo "============================================================"
