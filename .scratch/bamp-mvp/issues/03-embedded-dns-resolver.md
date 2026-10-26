# 03 - Embedded DNS Resolver for *.test

Status: ready-for-agent

## Goal
Implement a lightweight UDP DNS server in `src-tauri/src/dns/` listening on `127.0.0.1:53` that answers queries for any domain ending in `.test` with `127.0.0.1`.

## Requirements
1. Bind UDP socket to `127.0.0.1:53`.
   - If binding fails (e.g. port already used by systemd-resolved or another resolver), log a clear diagnostic warning and fallback to `/etc/hosts` / Windows `hosts` synchronization.
2. Parse incoming DNS queries:
   - Extract domain query name (QNAME).
   - If QNAME ends with `.test` and query type is `A` (IPv4):
     - Construct standard DNS response packet containing `127.0.0.1` with a 60-second TTL.
     - Send packet back to querying socket.
3. Run as an asynchronous Tokio background task managed by the Supervisor.

## Completion Criteria
- Dig/nslookup query `dig @127.0.0.1 -p 53 anyproject.test` returns `127.0.0.1`.

## Comments
