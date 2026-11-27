#!/usr/bin/env bash
# Keeps a Fable Flight relay on the newest published image, so relay
# operators don't have to be around when a new version comes out.
#
# Run it from cron (or a systemd timer) next to the relay's
# docker-compose.yml, for example every 15 minutes:
#
#   */15 * * * * /path/to/relay/autoupdate.sh
#
# Each run pulls the service's image. When a newer one has been published, it
# recreates the container, then checks the relay is up again (its status
# page answers, or without one, the container keeps running). If it isn't
# up, it goes back to the image it had and skips the bad one until another
# is published. Nothing changes when there is nothing new; a player sees a
# few seconds' gap at most, only when the relay is updated.
#
# Settings (environment variables, all optional):
#   FFMP_DIR          the folder with docker-compose.yml (default: this script's)
#   FFMP_SERVICE      the Compose service (default: relay)
#   FFMP_STATUS_URL   the status page to check (default:
#                     http://127.0.0.1:7481/status.json; "none" if it is off)
#   FFMP_UPDATE_LOG   where to log (default: autoupdate.log in FFMP_DIR)
#
# It needs docker with the compose plugin, curl and flock, and a user allowed
# to run docker (root, or in the docker group).
set -euo pipefail

DIR="${FFMP_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
SERVICE="${FFMP_SERVICE:-relay}"
STATUS_URL="${FFMP_STATUS_URL:-http://127.0.0.1:7481/status.json}"
LOG="${FFMP_UPDATE_LOG:-$DIR/autoupdate.log}"
SKIP_FILE="$DIR/.autoupdate.skip"

cd "$DIR"
# One run at a time.
exec 9>"$DIR/.autoupdate.lock"
flock -n 9 || exit 0

log() {
    printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >>"$LOG"
    # (Keep the log short.)
    if [ "$(wc -l <"$LOG")" -gt 2000 ]; then
        tail -n 1000 "$LOG" >"$LOG.tmp" && mv "$LOG.tmp" "$LOG"
    fi
}

# The relay's version, from its status page ("?" when it doesn't say).
version() {
    [ "$STATUS_URL" = none ] && { echo "?"; return; }
    curl -fsS -m 3 "$STATUS_URL" 2>/dev/null | sed -n 's/.*"version":"\([^"]*\)".*/\1/p' | head -n 1 | grep . || echo "?"
}

# Up: the container still runs 20 s later without having restarted, and its
# status page answers then (when it has one).
healthy() {
    local cid="$1" i
    sleep 20
    [ "$(docker inspect -f '{{.State.Running}} {{.RestartCount}}' "$cid" 2>/dev/null)" = "true 0" ] || return 1
    [ "$STATUS_URL" = none ] && return 0
    for i in $(seq 1 5); do
        curl -fsS -m 3 -o /dev/null "$STATUS_URL" 2>/dev/null && return 0
        sleep 2
    done
    return 1
}

cid="$(docker compose ps -q "$SERVICE")"
if [ -z "$cid" ]; then
    log "no running '$SERVICE' container here: nothing to update"
    exit 0
fi
# (The image docker-compose.yml names, so changing it there is picked up too.)
ref="$(docker compose config --images "$SERVICE" | head -n 1)"
running="$(docker inspect -f '{{.Image}}' "$cid")"
# The image to go back to, kept under its own tag beside this one's.
repo="$ref"
case "${ref##*/}" in *:*) repo="${ref%:*}" ;; esac
ROLLBACK_TAG="$repo:rollback"

if ! out="$(docker compose pull -q "$SERVICE" 2>&1)"; then
    log "pull failed: $(echo "$out" | tail -n 1)"
    exit 1
fi
latest="$(docker image inspect -f '{{.Id}}' "$ref")"
if [ "$latest" = "$running" ]; then
    exit 0
fi
if [ -f "$SKIP_FILE" ] && [ "$(cat "$SKIP_FILE")" = "$latest" ]; then
    # (This image failed before: stay on the one running.)
    docker tag "$running" "$ref"
    exit 0
fi

before="$(version)"
old_rollback="$(docker image inspect -f '{{.Id}}' "$ROLLBACK_TAG" 2>/dev/null || true)"
docker tag "$running" "$ROLLBACK_TAG"
docker compose up -d --pull never "$SERVICE" >/dev/null 2>&1
cid="$(docker compose ps -q "$SERVICE")"

if [ -n "$cid" ] && healthy "$cid"; then
    log "updated $ref: $before -> $(version) (${latest#sha256:})"
    rm -f "$SKIP_FILE"
    # The image kept for the rollback before this one is no longer needed,
    # unless something else still names it.
    if [ -n "$old_rollback" ] && [ "$old_rollback" != "$running" ] &&
        [ "$(docker image inspect -f '{{len .RepoTags}}' "$old_rollback" 2>/dev/null)" = 0 ]; then
        docker rmi "$old_rollback" >/dev/null 2>&1 || true
    fi
else
    log "the new image (${latest#sha256:}) did not come up: back to $before"
    echo "$latest" >"$SKIP_FILE"
    docker tag "$ROLLBACK_TAG" "$ref"
    docker compose up -d --pull never "$SERVICE" >/dev/null 2>&1
fi
