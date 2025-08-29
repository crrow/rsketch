#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Add the Grafana Helm repository
helm repo add grafana https://grafana.github.io/helm-charts
# Add the LocalStack Helm repository
helm repo add localstack https://localstack.github.io/helm-charts

helm repo update

# Install Loki
helm upgrade --install loki grafana/loki-stack --values "${SCRIPT_DIR}/loki-values.yaml"

# Install Grafana
helm upgrade --install grafana grafana/grafana --values "${SCRIPT_DIR}/grafana-values.yaml"

# Install Tempo
helm upgrade --install tempo grafana/tempo --values "${SCRIPT_DIR}/tempo-values.yaml"

# Install Pyroscope
helm upgrade --install pyroscope grafana/pyroscope --values "${SCRIPT_DIR}/pyroscope-values.yaml"

# Install the LocalStack Helm chart
helm upgrade --install localstack localstack/localstack --values "${SCRIPT_DIR}/localstack.yaml"