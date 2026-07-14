# 07 — Microservices

A microservices architecture example with an API gateway (Klyron), a Python user service, and a Node.js orders service. Demonstrates Klyron's polyglot capabilities in a distributed setup.

## Files

| File                              | Description                  |
|-----------------------------------|------------------------------|
| `docker-compose.yml`              | Multi-service orchestration  |
| `package.json`                    | Project metadata             |
| `api-gateway/server.js`           | Klyron-based API gateway     |
| `services/users/server.py`        | Python user microservice     |
| `services/orders/server.js`       | Node.js orders microservice  |

## Run

```bash
docker compose up
```

Or run each service manually:

```bash
# API Gateway
klyron run api-gateway/server.js

# Users service (Python)
python services/users/server.py

# Orders service
klyron run services/orders/server.js
```

## Architecture

```
Client -> API Gateway (:8080) -> Users Service (:4001)
                              -> Orders Service (:4002)
```
