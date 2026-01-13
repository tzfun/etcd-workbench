#!/bin/bash
ENV_FILE="../.env"

while IFS= read -r line || [ -n "$line" ]; do
    line=$(echo "$line" | tr -d '\r')
    if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
        continue
    fi
    if [[ "$line" =~ ^([^=]+)=(.*)$ ]]; then
        key="${BASH_REMATCH[1]}"
        value="${BASH_REMATCH[2]}"
        key=$(echo "$key" | xargs)
        value=$(echo "$value" | xargs)
        value="${value%\"}"
        value="${value#\"}"
        value="${value%\'}"
        value="${value#\'}"
        export "$key=$value"
        echo "Key: [$key], Value length: ${#value}"
    fi
done < "$ENV_FILE"

echo ""
echo "TAURI_PRIVATE_KEY length: ${#TAURI_PRIVATE_KEY}"
echo "TAURI_KEY_PASSWORD length: ${#TAURI_KEY_PASSWORD}"
