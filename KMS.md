# KMS Integration Guide

dugout supports hybrid encryption: secrets encrypted for both **age keys** (developers) and **cloud KMS** (production). Developers decrypt locally. Servers decrypt via IAM — no key files.

## How It Works

When KMS is configured, secrets are encrypted in **hybrid mode**: each secret is stored as an envelope containing both age-encrypted and KMS-encrypted ciphertext:

```json
{
  "version": "dugout-envelope-v2",
  "age": "<age-encrypted for team members>",
  "kms": "<KMS-encrypted ciphertext>",
  "provider": "aws"
}
```

At decrypt time:
1. **Age identity available?** → decrypt via age (fast, offline)
2. **No age identity + cloud creds?** → decrypt via KMS API

Developers hit path 1. Production servers hit path 2. Same vault, same secrets.

## Setup

### AWS KMS

1. Create a KMS key:

```bash
aws kms create-key --description "dugout secrets"
# Note the KeyId or Arn from the output
```

2. Initialize with KMS:

```bash
dugout init --kms-key arn:aws:kms:us-east-1:123456789012:key/abc-123
```

3. Grant your production IAM role decrypt access:

```json
{
  "Effect": "Allow",
  "Action": "kms:Decrypt",
  "Resource": "arn:aws:kms:us-east-1:123456789012:key/abc-123"
}
```

4. In production (EC2/ECS/Lambda with the IAM role):

```bash
dugout run -- ./start.sh
```

### GCP Cloud KMS

1. Create a key ring and key:

```bash
gcloud kms keyrings create my-ring --location global
gcloud kms keys create my-key --keyring my-ring --location global --purpose encryption
```

2. Initialize with KMS:

```bash
dugout init --kms-key projects/my-project/locations/global/keyRings/my-ring/cryptoKeys/my-key
```

3. Grant your service account decrypt access:

```bash
gcloud kms keys add-iam-policy-binding my-key \
  --keyring my-ring --location global \
  --member serviceAccount:my-app@my-project.iam.gserviceaccount.com \
  --role roles/cloudkms.cryptoKeyDecrypter
```

4. In production (Compute Engine/Cloud Run with the service account):

```bash
dugout run -- ./start.sh
```

## Adding KMS to an Existing Vault

To add KMS to an existing age-only vault:

```bash
# Edit .dugout.toml and add:
# [kms]
# key = "arn:aws:kms:us-east-1:123456789012:key/abc-123"

# Re-encrypt all secrets with hybrid envelope
dugout secrets rotate
```

After rotation, all secrets will be encrypted for both age recipients and the KMS key.

## Provider Detection

dugout auto-detects the KMS provider from the key format:

| Format | Provider |
|---|---|
| `arn:aws:kms:...` | AWS KMS |
| `projects/.../cryptoKeys/...` | GCP Cloud KMS |

## Feature Flags

KMS backends require feature flags at compile time:

```bash
cargo install dugout --features aws   # AWS KMS support
cargo install dugout --features gcp   # GCP KMS support
```

## Multi-Region KMS

AWS KMS supports multi-region keys. Use a multi-region key ARN and replicate to your deployment regions. dugout encrypts once — each region decrypts locally using the regional replica.

## Security Notes

- KMS keys should have minimal IAM permissions (decrypt only for production)
- Age keys provide defense-in-depth: even if KMS is compromised, age encryption is independent
- The hybrid envelope ensures developers can always work offline
- `dugout secrets rotate` re-encrypts everything — use after removing team members
