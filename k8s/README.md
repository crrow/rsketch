# Kubernetes Configuration for rsketch

This directory contains the Kubernetes configuration for deploying the rsketch application and its observability stack.

## Services

*   **Loki**: A horizontally-scalable, highly-available, multi-tenant log aggregation system for logs
*   **Grafana**: The open and composable observability and data visualization platform
*   **Tempo**: Distributed tracing backend for traces
*   **Pyroscope**: Continuous profiling platform for performance monitoring
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
```

### Restart Services
```bash
kubectl rollout restart deployment/grafana
```
