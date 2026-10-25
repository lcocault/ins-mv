#!/usr/bin/env bash
# Pilote le serveur Web de la fiche de personnage (lecture seule).
# Usage : scripts/web.sh start | stop | restart | status | logs
set -u

RACINE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXEC="$RACINE/moteur/target/debug/insmv-web"
DOSSIER_RUN="$RACINE/.run"
PID_FILE="$DOSSIER_RUN/web.pid"
LOG_FILE="$DOSSIER_RUN/web.log"
PORT="${INSMV_WEB_PORT:-8787}"
URL="http://localhost:$PORT"
export PATH="$HOME/.cargo/bin:$PATH"

# PID du serveur : fichier PID s'il est valide, sinon recherche par nom
# (couvre un serveur lancé à la main).
pid_serveur() {
  local pid
  if [[ -f "$PID_FILE" ]]; then
    pid="$(cat "$PID_FILE")"
    if [[ -n "$pid" && "$(ps -p "$pid" -o comm= 2>/dev/null)" == "insmv-web" ]]; then
      echo "$pid"
      return 0
    fi
  fi
  pgrep -x insmv-web | head -n 1
}

repond() {
  curl -s -o /dev/null --max-time 2 "$URL/"
}

demarrer() {
  local pid
  pid="$(pid_serveur)"
  if [[ -n "$pid" ]]; then
    echo "Déjà démarré (PID $pid) : $URL"
    return 0
  fi
  echo "Compilation si nécessaire..."
  if ! (cd "$RACINE/moteur" && cargo build --quiet -p insmv-web); then
    echo "Échec de la compilation." >&2
    return 1
  fi
  mkdir -p "$DOSSIER_RUN"
  setsid nohup "$EXEC" >"$LOG_FILE" 2>&1 &
  echo $! >"$PID_FILE"
  for _ in $(seq 1 30); do
    if repond; then
      # setsid peut créer un processus intermédiaire : on enregistre le vrai PID du serveur.
      pgrep -x insmv-web | head -n 1 >"$PID_FILE"
      echo "Serveur démarré (PID $(cat "$PID_FILE")) : $URL"
      return 0
    fi
    if [[ -z "$(pid_serveur)" ]]; then
      echo "Le serveur s'est arrêté au démarrage :" >&2
      tail -n 20 "$LOG_FILE" >&2
      rm -f "$PID_FILE"
      return 1
    fi
    sleep 0.5
  done
  echo "Le serveur ne répond pas après 15 s ; voir $LOG_FILE" >&2
  return 1
}

arreter() {
  local pid
  pid="$(pid_serveur)"
  if [[ -z "$pid" ]]; then
    rm -f "$PID_FILE"
    echo "Le serveur n'est pas démarré."
    return 0
  fi
  kill "$pid" 2>/dev/null
  for _ in $(seq 1 20); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.25
  done
  if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null
  fi
  rm -f "$PID_FILE"
  echo "Serveur arrêté (PID $pid)."
}

etat() {
  local pid
  pid="$(pid_serveur)"
  if [[ -z "$pid" ]]; then
    echo "Arrêté."
    return 1
  fi
  if repond; then
    echo "En marche (PID $pid) : $URL"
  else
    echo "Processus présent (PID $pid) mais ne répond pas sur $URL ; voir $LOG_FILE" >&2
    return 2
  fi
}

case "${1:-}" in
  start)   demarrer ;;
  stop)    arreter ;;
  restart) arreter && demarrer ;;
  status)  etat ;;
  logs)    tail -n 50 "$LOG_FILE" 2>/dev/null || echo "Pas encore de journal." ;;
  *)
    echo "Usage : $0 start | stop | restart | status | logs" >&2
    exit 64
    ;;
esac
