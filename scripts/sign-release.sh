#!/usr/bin/env bash
set -euo pipefail

# Binary signing script for Klyron releases
# Signs the built binary with an ed25519 key
#
# Usage: ./scripts/sign-release.sh <binary-path> [key-name]
#
# Environment variables:
#   KLYRON_SIGN_KEY_NAME  - Name of the signing key (default: "klyron-release")
#   KLYRON_SIGN_KEY       - Path to secret key PEM file (overrides key-name)
#   KLYRON_PUB_KEY        - Path to public key PEM file (overrides key-name)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BINARY="${1:-$PROJECT_ROOT/target/release/klyron}"
KEY_NAME="${2:-${KLYRON_SIGN_KEY_NAME:-klyron-release}}"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Build first with: cargo build --release"
    exit 1
fi

# Use explicit key paths if provided, otherwise use key name
if [ -n "${KLYRON_SIGN_KEY:-}" ] && [ -n "${KLYRON_PUB_KEY:-}" ]; then
    SEC_KEY="$KLYRON_SIGN_KEY"
    PUB_KEY="$KLYRON_PUB_KEY"
else
    CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/klyron/keys"
    SEC_KEY="$CONFIG_DIR/$KEY_NAME.sec"
    PUB_KEY="$CONFIG_DIR/$KEY_NAME.pub"

    if [ ! -f "$SEC_KEY" ]; then
        echo "Generating new signing key pair: $KEY_NAME"
        mkdir -p "$CONFIG_DIR"
        cargo run --release -- keys generate "$KEY_NAME" 2>/dev/null || {
            # Fallback: generate using the klyron_pm key generation
            echo "Use 'klyron keys generate $KEY_NAME' or place keys at:"
            echo "  $SEC_KEY"
            echo "  $PUB_KEY"
            exit 1
        }
    fi
fi

echo "Signing binary: $BINARY"
echo "Using key: $KEY_NAME"

# Compute SHA-256 hash of binary
BINARY_HASH=$(sha256sum "$BINARY" | cut -d' ' -f1)
echo "Binary SHA-256: $BINARY_HASH"

# Sign the binary hash with the secret key
# Using klyron_pm signing module
SIG_FILE="${BINARY}.sig"
cargo run --release -- sign "$BINARY" "$KEY_NAME" 2>/dev/null || {
    # Direct signing using openssl as fallback
    if command -v openssl &>/dev/null && [ -f "$SEC_KEY" ]; then
        echo "$BINARY_HASH" | openssl pkeyutl -sign -inkey "$SEC_KEY" -out "$SIG_FILE"
        echo "Signed with openssl"
    else
        echo "Error: Cannot sign binary. Use 'klyron sign' command."
        exit 1
    fi
}

echo "Signature written to: $SIG_FILE"
echo "To verify:"
echo "  export KLYRON_VERIFY_SIGNATURE=1"
echo "  export KLYRON_PUBLIC_KEY_PATH=$PUB_KEY"
echo "  ./klyron --version"

# Verify the signature
if [ -f "$PUB_KEY" ]; then
    echo ""
    echo "Verifying signature..."
    if command -v openssl &>/dev/null; then
        VERIFIED=$(echo "$BINARY_HASH" | openssl pkeyutl -verify -pubin -inkey "$PUB_KEY" -sigfile "$SIG_FILE" 2>&1 || true)
        if echo "$VERIFIED" | grep -q "Success"; then
            echo "Signature verified successfully!"
        else
            echo "Warning: Signature verification failed: $VERIFIED"
        fi
    fi
fi
