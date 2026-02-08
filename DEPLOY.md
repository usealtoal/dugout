# Deployment Guide

dugout works in CI/CD and production with zero cloud dependencies.

## Quick Start

1. Generate a CI identity:

```bash
dugout setup --output ci.key
```

2. Admit it to your project:

```bash
dugout team add ci "$(dugout whoami)"
```

3. Store the private key as a CI secret (e.g., `DUGOUT_IDENTITY` in GitHub Actions).

4. In CI, decrypt and run:

```yaml
- env:
    DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
  run: dugout run -- ./deploy.sh
```

## Identity Resolution

dugout checks for identities in this order:

| Priority | Source | Use case |
|---|---|---|
| 1 | `DUGOUT_IDENTITY` env var | CI/CD — inline age key |
| 2 | `DUGOUT_IDENTITY_FILE` env var | Servers — path to key file |
| 3 | `~/.dugout/keys/<project>/identity.key` | Developer — project-local |
| 4 | `~/.dugout/identity` | Developer — global |

The first valid identity that is a recipient in the vault wins. No flags needed.

## CI/CD Examples

### GitHub Actions

```yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install dugout
        run: curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh

      - name: Deploy
        env:
          DUGOUT_IDENTITY: ${{ secrets.DUGOUT_IDENTITY }}
        run: dugout run -- ./deploy.sh
```

### GitLab CI

```yaml
deploy:
  script:
    - curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
    - dugout run -- ./deploy.sh
  variables:
    DUGOUT_IDENTITY: $DUGOUT_IDENTITY
```

### Docker

```dockerfile
FROM debian:bookworm-slim
RUN curl -LsSf https://raw.githubusercontent.com/usealtoal/dugout/main/scripts/install.sh | sh
COPY . /app
WORKDIR /app
CMD ["dugout", "run", "--", "./start.sh"]
```

```bash
docker run -e DUGOUT_IDENTITY="$KEY" myapp
```

### Kubernetes

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: dugout-identity
type: Opaque
stringData:
  key: AGE-SECRET-KEY-1...
---
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: app
          env:
            - name: DUGOUT_IDENTITY
              valueFrom:
                secretKeyRef:
                  name: dugout-identity
                  key: key
          command: ["dugout", "run", "--", "./start.sh"]
```

## Security

- **Never bake `DUGOUT_IDENTITY` into a Docker image.** Inject at runtime.
- Use your CI provider's secret storage for the key.
- The identity is a single age private key (~200 bytes).
- Rotate by generating a new identity, admitting it, and removing the old one.
- After removing a team member, run `dugout secrets rotate` to re-encrypt all secrets.
