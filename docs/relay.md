# Hosting a Fable Flight Multiplayer Relay

Fable Flight multiplayer uses lightweight UDP relay servers. You can host one for friends, join the public relay network, or run a separate private network.

## Quick start

You need a server with Docker and a publicly reachable UDP port **7480**.

From the Fable Flight repository:

```sh
cd relay
docker compose up -d
```

Or run the published image directly:

```sh
docker run -d --name fableflight-relay --restart unless-stopped \
  -p 7480:7480/udp \
  ghcr.io/rolandjg/fableflight-relay
```

Allow inbound **UDP 7480** in your server firewall and hosting provider's firewall. Players can select your relay in **Settings → Multiplayer**, or enter its hostname manually. The game can also use `--mp host[,name]` for testing.

## Configuration

The relay is configured through environment variables. Add them to your Compose service's `environment:` section or pass them to `docker run` with `-e NAME=value`.

| Variable | Purpose |
| --- | --- |
| `FFMP_PORT` | UDP port used by the relay (normally `7480`). |
| `FFMP_HTTP_PORT` | HTTP status port (normally `7481`); set to `0` to disable. |
| `FFMP_NAME` | Human-readable relay name. |
| `FFMP_PUBLIC` | Public `host[:port]` address advertised to other relays and players. |
| `FFMP_MAX_PLAYERS` | Player limit. |
| `FFMP_RANGE_KM` | Visibility range for nearby players. |
| `FFMP_PEER_SECRET` | Shared secret (at least 16 characters) for relay-to-relay traffic. |
| `FFMP_HUB` | Comma-separated main relay addresses; `none` disables the built-in hubs. |
| `FFMP_PEERS` | Additional relay addresses to connect to directly. |
| `FFMP_MAX_PER_IP` | Players at most from one address (default `32`). |
| `FFMP_BANS` | A file of banned addresses, networks and names (see Moderation). |
| `FFMP_STATUS_POSITIONS` | `1` to show the pilots' positions on the status page (off by default). |

For example, give your relay a recognizable identity:

```sh
docker run -d --name fableflight-relay --restart unless-stopped \
  -p 7480:7480/udp \
  -e FFMP_NAME="My Relay" \
  -e FFMP_PUBLIC="relay.example.com" \
  ghcr.io/rolandjg/fableflight-relay
```

Replace `relay.example.com` with a hostname that resolves to your server. If you change the listening UDP port, update the Docker port mapping, firewall rule and advertised address accordingly.

## Status page

The relay exposes `/` and `/status.json` on its HTTP status port (`7481` by default). The page reports connected pilots and start-screen watchers. It shows no aircraft positions unless `FFMP_STATUS_POSITIONS` is on. To access it outside the server, expose that port only as needed; for a public status page, put it behind an HTTPS reverse proxy.

The directory endpoint `/servers.json` is also used for relay discovery.

With `FFMP_STATUS_POSITIONS=1`, the page also shows where each pilot is, on an OpenStreetMap world map and in the list (position and heading). Positions are rounded to about a kilometre, as on the game's world map. This is off by default: without it, the page shows no positions.

`/healthz` answers `ok` while the relay runs, for health checks. `/metrics` gives this relay's counts in Prometheus' text format: players, start-screen watchers, players on the other relays, relays up and down, packets, and what was turned away (banned, full, too many from one address, over the packet rate). It holds no names or positions.

## Moderation

To keep someone off your relay, list them in a text file and point `FFMP_BANS` at it. One entry per line: an address, a network in CIDR notation, or `name:` followed by a pilot name (any case). `#` starts a comment.

```text
203.0.113.7
198.51.100.0/24
2001:db8::/32
name: Some Griefer
```

The relay reads the file again within a second of it changing, without a restart. Whoever it now bans is dropped at once and gets no more answers. Lines it cannot read are reported in its log and skipped. With Docker, mount the file read-only and give its path inside the container:

```yaml
    environment:
      FFMP_BANS: "/bans.txt"
    volumes:
      - ./bans.txt:/bans.txt:ro
```

A ban applies to this relay only; players on other relays of the network are still shown. Since the name is the player's own choice, an address ban holds better than a name ban.

## Automatic updates

A new relay image is published whenever the relay or its protocol changes on `main` (tagged `latest`, the commit and the relay's version, for example `0.5.0`). So that every relay of the network picks it up without its operator being around, run `relay/autoupdate.sh` from cron next to your `docker-compose.yml`:

```sh
cp relay/autoupdate.sh /path/to/your/relay/
crontab -e
# then add:
7-59/15 * * * * /path/to/your/relay/autoupdate.sh
```

Every 15 minutes it pulls the image your `docker-compose.yml` names. When a newer one has been published, it recreates the container and checks that the relay is still running 20 seconds later and that its status page answers. If it isn't, it goes back to the previous image (kept as `<image>:rollback`) and skips the failed one until another is published. When there is nothing new, nothing changes. Players see a gap of a few seconds at most, only when an update is applied. What it did goes to `autoupdate.log` beside the script.

`init: true` in the provided `docker-compose.yml` lets the relay stop at once when it is restarted (without it, Docker waits 10 seconds and then kills it), so an update takes a second or two. Players' games say Hello again by themselves after 5 seconds without news from the relay.

It needs `docker` with the compose plugin, `curl` and `flock`, run by root or by a user in the `docker` group. Optional environment variables: `FFMP_DIR` (the folder with `docker-compose.yml`, by default the script's), `FFMP_SERVICE` (default `relay`), `FFMP_STATUS_URL` (default `http://127.0.0.1:7481/status.json`, or `none` when the status page is off) and `FFMP_UPDATE_LOG`.

To stay on one version instead, name it in `docker-compose.yml` (`ghcr.io/rolandjg/fableflight-relay:0.5.0`) and don't run the script.

**Protocol changes.** Relays and games only talk to the same protocol version (`VERSION` in `src/mp_proto.rs`). With every relay updating itself, a change reaches the whole network within about 15 minutes of its image being published. A change that bumps `VERSION` should still decode the previous version for one release, so that relays (and players) a little behind are not cut off in the meantime.

## Public and private networks

Relays can share player information, so pilots on different servers still see one another. Relays exchange signed, dated, sequenced messages using a common `FFMP_PEER_SECRET`. **Do not publish that secret in a repository or documentation.**

By default, relays use Fable Flight's built-in main relays for discovery. To join an existing coordinated network, configure the network's shared secret and public hostname with its operators. `FFMP_PEERS` can add direct peer connections.

For a separate private network, configure the same secret on your participating relays and use `FFMP_HUB=none` to exclude the built-in hubs, or set `FFMP_HUB` to your own comma-separated hub addresses. Configure clients to reach your private servers directly; they will not automatically appear in the public directory.

## Operating a main relay

If you are adding a new built-in main relay to the project:

1. Give it a stable public hostname, UDP 7480 access, `FFMP_PUBLIC`, `FFMP_NAME`, and the network's shared `FFMP_PEER_SECRET`.
2. Serve its status page and `https://<host>/servers.json` through HTTPS. Leave `FFMP_HUB` unset so it can link with the existing main relays.
3. Add its hostname to `MAIN_RELAYS` in `src/mp_proto.rs` and its directory URL to `DIRECTORY_URLS` in `src/mp.rs`. The project's test checks that the lists agree.
4. Deploy the updated game/relay builds. Older clients can still discover the new relay through existing hubs.

## Troubleshooting

- **Cannot connect:** Check that Docker is running, UDP 7480 is published, and both the host and cloud firewalls allow it. UDP is required; opening only TCP will not work.
- **Not shown in server list:** Confirm `FFMP_PUBLIC` is reachable and that the directory/hub configuration is correct. A quiet relay can disappear from discovery after about a minute.
- **Players on different relays cannot see each other:** Verify that they belong to the same network, use the same `FFMP_PEER_SECRET`, and have reachable peer addresses.
- **Status page unavailable:** Check `FFMP_HTTP_PORT`, the reverse proxy, and whether HTTP access was deliberately disabled.

The relay keeps no persistent player data on disk. It sends aircraft state while multiplayer is enabled; on the start screen, clients request network-map updates instead.
