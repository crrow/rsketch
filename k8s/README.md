# Kubernetes Configuration for rsketch

This directory contains the Kubernetes configuration for deploying the rsketch application and its observability stack.

## Services

*   **Loki**: A horizontally-scalable, highly-available, multi-tenant log aggregation system for logs
*   **Grafana**: The open and composable observability and data visualization platform
*   **Tempo**: Distributed tracing backend for traces
*   **Pyroscope**: Continuous profiling platform for performance monitoring
*   **Consul**: Service mesh and configuration management platform
*   **LocalStack**: Local AWS cloud stack for development

## Installation

To deploy the services, run the following command:

```bash
./install.sh
```

## Accessing Services on Localhost

After installation, you need to set up port forwarding to access the services from your localhost:

### Grafana Web Interface

1. **Port forward Grafana**:
   ```bash
   kubectl port-forward svc/grafana 3000:80
   ```

2. **Access Grafana**:
   - URL: http://localhost:3000
   - Username: `admin`
   - Password: Get it with: `kubectl get secret grafana -o jsonpath="{.data.admin-password}" | base64 --decode`

### Consul Web Interface

1. **Port forward Consul**:
   ```bash
   kubectl port-forward svc/consul-ui 8500:80
   ```

2. **Access Consul**:
   - URL: http://localhost:8500
   - Username: Not required for development setup
   - Features: Service discovery, configuration management, health checks

### Other Services (Optional)

- **Loki**: `kubectl port-forward svc/loki 3100:3100` → http://localhost:3100
- **Tempo**: `kubectl port-forward svc/tempo 3200:3200` → http://localhost:3200
- **Pyroscope**: `kubectl port-forward svc/pyroscope 4040:4040` → http://localhost:4040
- **LocalStack**: `kubectl port-forward svc/localstack 4566:4566` → http://localhost:4566

## Pre-configured Datasources

Grafana comes pre-configured with the following datasources:
- **Loki** (default): For log aggregation and querying
- **Tempo**: For distributed tracing
- **Pyroscope**: For continuous profiling
- **Consul**: For monitoring Consul metrics and health

## Configuration Management with Consul

Consul provides a distributed key-value store for configuration management. Here's how to use it:

### Basic Configuration Operations

1. **Set a configuration value**:
   ```bash
   kubectl exec -it consul-server-0 -- consul kv put config/app/database_url "postgresql://localhost:5432/myapp"
   ```

2. **Get a configuration value**:
   ```bash
   kubectl exec -it consul-server-0 -- consul kv get config/app/database_url
   ```

3. **List all configurations**:
   ```bash
   kubectl exec -it consul-server-0 -- consul kv get -recurse config/
   ```

4. **Delete a configuration**:
   ```bash
   kubectl exec -it consul-server-0 -- consul kv delete config/app/database_url
   ```

### Configuration Structure Recommendations

Use a hierarchical structure for your configurations:

```
config/
├── app/
│   ├── database_url
│   ├── redis_url
│   ├── log_level
│   └── feature_flags/
│       ├── enable_new_ui
│       └── enable_metrics
├── services/
│   ├── auth/
│   │   ├── jwt_secret
│   │   └── token_expiry
│   └── payment/
│       ├── stripe_key
│       └── webhook_secret
└── environment/
    ├── stage
    └── region
```

### Integrating with Your Rust Application

To use Consul for configuration in your Rust application, consider using the `consul` crate:

```toml
[dependencies]
consul = "0.4"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

Example configuration client:
```rust
use consul::Consul;

#[derive(serde::Deserialize)]
struct AppConfig {
    database_url: String,
    log_level: String,
}

async fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let consul = Consul::new("http://localhost:8500")?;
    
    let database_url = consul.kv().get("config/app/database_url", None).await?
        .map(|kv| String::from_utf8(kv.value).unwrap())
        .unwrap_or_else(|| "postgresql://localhost:5432/default".to_string());
    
    let log_level = consul.kv().get("config/app/log_level", None).await?
        .map(|kv| String::from_utf8(kv.value).unwrap())
        .unwrap_or_else(|| "info".to_string());
    
    Ok(AppConfig { database_url, log_level })
}
```

### Configuration Watching

Consul supports watching for configuration changes. Use this for dynamic configuration updates:

```bash
# Watch for changes in a specific key
kubectl exec -it consul-server-0 -- consul watch -type=key -key=config/app/log_level
```

## Troubleshooting

### Check Service Status
```bash
kubectl get pods
kubectl get services
```

### View Service Logs
```bash
kubectl logs -l app.kubernetes.io/name=grafana
kubectl logs -l app.kubernetes.io/name=loki
kubectl logs -l app.kubernetes.io/name=tempo
kubectl logs -l app.kubernetes.io/name=pyroscope
kubectl logs -l app=consul
```

### Restart Services
```bash
kubectl rollout restart deployment/grafana
```
