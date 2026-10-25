#!/usr/bin/env bash
# Lance le serveur MCP « mj » (stdio). Utilisé par .mcp.json.
# Recompile seulement si le binaire est absent ou plus ancien que les sources,
# pour que le démarrage reste rapide (les clients MCP abandonnent après ~30 s).
RACINE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$RACINE/moteur/target/debug/insmv-mcp"
export PATH="$HOME/.cargo/bin:$PATH"

perime() {
  [[ ! -x "$BIN" ]] && return 0
  [[ -n "$(find "$RACINE/moteur" -path "$RACINE/moteur/target" -prune -o \
      \( -name '*.rs' -o -name 'Cargo.toml' -o -name '*.json' \) -newer "$BIN" -print -quit)" ]]
}

if perime; then
  echo "insmv-mcp : compilation..." >&2
  (cd "$RACINE/moteur" && cargo build --quiet -p insmv-mcp >&2) || exit 1
fi
exec "$BIN"
