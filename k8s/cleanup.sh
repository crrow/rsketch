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

set -e

echo "🧹 Cleaning up Kubernetes resources..."

# Function to safely uninstall Helm releases
uninstall_helm_release() {
    local release_name="$1"
    local namespace="${2:-default}"
    
    if helm list -n "$namespace" | grep -q "^$release_name"; then
        echo "  🗑️  Uninstalling $release_name..."
        helm uninstall "$release_name" -n "$namespace" || echo "    ⚠️  Failed to uninstall $release_name"
    else
        echo "  ⏭️  $release_name not found, skipping..."
    fi
}

# Uninstall all services in reverse order of installation
echo "📦 Uninstalling Helm releases..."
uninstall_helm_release "consul"
uninstall_helm_release "localstack"
uninstall_helm_release "pyroscope"
uninstall_helm_release "tempo"
uninstall_helm_release "grafana"
uninstall_helm_release "loki"

# Wait a moment for resources to be deleted
echo "⏳ Waiting for resources to be cleaned up..."
sleep 5

# Clean up any remaining resources
echo "🔍 Cleaning up remaining resources..."

# Delete any persistent volume claims
echo "  💾 Cleaning up PVCs..."
kubectl delete pvc --all --timeout=60s 2>/dev/null || echo "    ℹ️  No PVCs to clean up"

# Delete any secrets that might remain
echo "  🔐 Cleaning up secrets..."
kubectl delete secret --selector="app.kubernetes.io/managed-by=Helm" --timeout=60s 2>/dev/null || echo "    ℹ️  No Helm secrets to clean up"

# Delete any config maps that might remain
echo "  📋 Cleaning up config maps..."
kubectl delete configmap --selector="app.kubernetes.io/managed-by=Helm" --timeout=60s 2>/dev/null || echo "    ℹ️  No Helm config maps to clean up"

# Check for any remaining pods
echo "🔍 Checking for remaining pods..."
remaining_pods=$(kubectl get pods --no-headers 2>/dev/null | wc -l)
if [ "$remaining_pods" -gt 0 ]; then
    echo "  ⚠️  Some pods are still running:"
    kubectl get pods
    echo "  💡 You may need to wait for them to terminate or manually delete them."
else
    echo "  ✅ No pods remaining"
fi

# Check for any remaining services
echo "🔍 Checking for remaining services..."
remaining_services=$(kubectl get svc --no-headers 2>/dev/null | grep -v kubernetes | wc -l)
if [ "$remaining_services" -gt 0 ]; then
    echo "  ⚠️  Some services are still running:"
    kubectl get svc | grep -v kubernetes
    echo "  💡 You may need to manually delete them."
else
    echo "  ✅ No services remaining (except kubernetes default)"
fi

echo ""
echo "✨ Cleanup completed!"
echo ""
echo "📝 Summary:"
echo "   • All Helm releases have been uninstalled"
echo "   • PVCs, secrets, and config maps have been cleaned up"
echo "   • Check above for any remaining resources that need manual cleanup"
echo ""
echo "💡 To completely reset your cluster, you can also run:"
echo "   kubectl delete all --all"
echo "   (⚠️  Warning: This will delete ALL resources in the current namespace)"
