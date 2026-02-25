# Aurora DSQL SQLx Connector for Rust

A Rust connector for [Amazon Aurora DSQL](https://aws.amazon.com/rds/aurora/dsql/) using SQLx, providing seamless IAM authentication and connection pooling.

## Features

- **IAM Authentication**: Automatic AWS IAM token generation and injection
- **Token Caching**: Proactive token refresh (5 minutes before expiration)
- **Connection Pooling**: Built on SQLx's robust connection pool
- **Automatic OCC Retry**: Transparent retry with exponential backoff for optimistic concurrency errors
- **Region Auto-Detection**: Extracts region from hostname or uses AWS SDK defaults
- **AWS Profile Support**: Use specific AWS profiles for credentials
- **Type-Safe**: Full Rust type safety with SQLx's compile-time query checking

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aurora-dsql-sqlx-connector = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

1. **Set up AWS credentials and connection string:**

```bash
export AWS_REGION=us-east-1
export DSQL_CONNECTION_STRING="dsql://admin@your-cluster.dsql.us-east-1.on.aws:5432/postgres?region=us-east-1"
```

2. **Run the example:**

```bash
cargo run --example basic
```

3. **Or use in your code:**

```rust
use aurora_dsql_sqlx_connector::{DsqlPool, Result};
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<()> {
    let conn_str = std::env::var("DSQL_CONNECTION_STRING")
        .expect("DSQL_CONNECTION_STRING not set");
    
    let pool = DsqlPool::new(&conn_str).await?;
    
    let row = sqlx::query("SELECT 1 as value")
        .fetch_one(&*pool)
        .await?;
    let value: i32 = row.get("value");
    
    println!("Result: {}", value);
    Ok(())
}
```

## Usage

### Connection String Format

```
dsql://[user]@[host]:[port]/[database]?region=[aws-region]&profile=[aws-profile]
```

- `user`: Database user (typically `admin`)
- `host`: DSQL cluster endpoint
- `port`: Port number (default: 5432)
- `database`: Database name
- `region`: AWS region (optional, auto-detected from hostname or AWS SDK)
- `profile`: AWS profile name (optional, uses default credentials if not specified)

**Region Resolution Priority:**
1. Explicit `?region=...` in connection string
2. Parse from hostname (e.g., `cluster.dsql.us-east-1.on.aws`)
3. AWS SDK default region (`AWS_REGION` env var or `~/.aws/config`)

**Examples:**

```bash
# With explicit region
dsql://admin@cluster.dsql.us-east-1.on.aws/postgres?region=us-east-1

# Region auto-detected from hostname
dsql://admin@cluster.dsql.us-east-1.on.aws/postgres

# With AWS profile
dsql://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev

# Using AWS_REGION env var
export AWS_REGION=us-east-1
dsql://admin@cluster-id/postgres
```

### Query Execution

The connector provides transparent access to SQLx methods via `Deref`:

```rust
// Execute queries
sqlx::query("CREATE TABLE users (id INT PRIMARY KEY, name TEXT)")
    .execute(&*pool)
    .await?;

// Fetch single row
let row = sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(1)
    .fetch_one(&*pool)
    .await?;

// Fetch multiple rows
let rows = sqlx::query("SELECT * FROM users")
    .fetch_all(&*pool)
    .await?;
```

### Token Caching

IAM tokens are automatically cached and proactively refreshed. Each token is valid for 15 minutes and is refreshed at the 12-minute mark (80% of lifetime), providing a 3-minute buffer before expiration.

**Observing token generation:**

```bash
cargo run --example basic
```

**Expected output:**
```
🔄 No cached token, generating new one...
✨ Generated new token (expires in 900 seconds)
Test 1: Simple SELECT
  Result: 1
...
```

The token is generated once at startup and automatically refreshed as needed throughout the application lifecycle.

**Manual cache management:**

```rust
pool.clear_token_cache().await;
```

## OCC Retry Behavior

Aurora DSQL uses optimistic concurrency control. The connector provides helpers to detect and handle OCC errors:

```rust
use aurora_dsql_sqlx_connector::occ_retry::{is_occ_error, calculate_backoff, OCCRetryConfig};

let config = OCCRetryConfig::default(); // max_attempts: 3, exponential backoff

for attempt in 1..=config.max_attempts {
    match sqlx::query("UPDATE ...").execute(&*pool).await {
        Ok(result) => break,
        Err(e) if is_occ_error(&e) => {
            if attempt < config.max_attempts {
                tokio::time::sleep(calculate_backoff(&config, attempt)).await;
                continue;
            }
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    }
}
```

**OCC Error Detection:**
- SQLSTATE `40001` (serialization failure)
- Error codes `OC000` and `OC001`

**Backoff Strategy:**
- Exponential backoff: `base_delay * 2^attempt`
- Additive jitter: 0-25% of delay
- Max delay: 5000ms

## AWS Credentials

The connector uses the AWS SDK for Rust to generate IAM tokens. Credentials are resolved via:

**Default credential chain** (when profile not specified):
```bash
export AWS_REGION=us-east-1
# Uses: environment variables → ~/.aws/credentials (default profile) → IAM role
```

**Specific AWS profile** (via connection string):
```bash
# Uses credentials from ~/.aws/credentials [dev] profile
dsql://admin@cluster.dsql.us-east-1.on.aws/postgres?profile=dev
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires DSQL cluster)
export DSQL_CONNECTION_STRING="dsql://admin@your-cluster.dsql.us-east-1.on.aws/postgres?region=us-east-1"
export AWS_REGION=us-east-1
cargo test
```

## License

Apache-2.0
