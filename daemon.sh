#!/bin/bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PIDFILE="$DIR/ryvos-gateway.pid"
LOGFILE="$DIR/ryvos-gateway.log"
BIN="$DIR/target/release/ryvos"
CONFIG="$DIR/ryvos.toml"

case "$1" in
  start)
    if [ -f "$PIDFILE" ]; then
      PID=$(cat "$PIDFILE")
      if ps -p "$PID" > /dev/null 2>&1; then
        echo "ryvos is already running (PID: $PID)"
        exit 0
      else
        rm -f "$PIDFILE"
      fi
    fi
    echo "Starting ryvos gateway daemon..."
    python3 - <<PYEOF
import os, subprocess, time
pid = os.fork()
if pid > 0:
    os.waitpid(pid, 0)
    raise SystemExit(0)
os.setsid()
pid2 = os.fork()
if pid2 > 0:
    raise SystemExit(0)
log_fd = os.open("$LOGFILE", os.O_WRONLY | os.O_CREAT | os.O_APPEND)
os.dup2(log_fd, 1)
os.dup2(log_fd, 2)
os.close(log_fd)
devnull = os.open("/dev/null", os.O_RDONLY)
os.dup2(devnull, 0)
os.close(devnull)
os.chdir("$DIR")
with open("$PIDFILE", "w") as f:
    f.write(str(os.getpid()))
os.execv("$BIN", ["$BIN", "-c", "$CONFIG", "serve"])
PYEOF
    sleep 2
    if [ -f "$PIDFILE" ] && ps -p $(cat "$PIDFILE") > /dev/null 2>&1; then
      echo "ryvos started with PID $(cat "$PIDFILE"). Logs: $LOGFILE"
    else
      echo "Failed to start ryvos. Check $LOGFILE"
      exit 1
    fi
    ;;
  stop)
    if [ -f "$PIDFILE" ]; then
      PID=$(cat "$PIDFILE")
      echo "Stopping ryvos (PID: $PID)..."
      kill "$PID" 2>/dev/null || true
      sleep 2
      if ps -p "$PID" > /dev/null 2>&1; then
        echo "Force killing ryvos..."
        kill -9 "$PID" 2>/dev/null || true
      fi
      rm -f "$PIDFILE"
      echo "ryvos stopped."
    else
      echo "PID file not found. Checking running processes..."
      pkill -f "ryvos.*serve" || echo "No ryvos process found."
    fi
    ;;
  status)
    if [ -f "$PIDFILE" ] && ps -p $(cat "$PIDFILE") > /dev/null 2>&1; then
      echo "ryvos is running (PID: $(cat "$PIDFILE"))"
      curl -s -I http://127.0.0.1:18789/api/health | head -n 1
    else
      echo "ryvos is not running"
    fi
    ;;
  restart)
    $0 stop
    sleep 1
    $0 start
    ;;
  *)
    echo "Usage: $0 {start|stop|restart|status}"
    exit 1
    ;;
esac
