#!/bin/bash
# Copyright 2025 Crrow
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Add the Grafana Helm repository
helm repo add grafana https://grafana.github.io/helm-charts
# Add the LocalStack Helm repository
helm repo add localstack https://localstack.github.io/helm-charts
# Add the HashiCorp Helm repository for Consul
helm repo add hashicorp https://helm.releases.hashicorp.com

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

# Install Consul for configuration management
helm upgrade --install consul hashicorp/consul --values "${SCRIPT_DIR}/consul-values.yaml"