# Kubernetes Deployment Guide

Deploy Ulf Orchestrator on Kubernetes for scalable, resilient AI orchestration.

## Prerequisites

- Kubernetes cluster 1.20+ (local or cloud)
- `kubectl` configured with cluster access
- Helm 3.0+ (optional, for Helm deployment)
- Container registry access (Docker Hub, GCR, ECR, etc.)
- Minimum 2 nodes with 4GB RAM each

## Quick Start

### Basic Deployment with kubectl

Create namespace and deploy:

```bash
# Create namespace
kubectl create namespace ulf-orchestrator

# Apply manifests
kubectl apply -f k8s/ -n ulf-orchestrator

# Check deployment
kubectl get pods -n ulf-orchestrator
```

## Kubernetes Manifests

### 1. Namespace and ConfigMap

```yaml
# k8s/00-namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: ulf-orchestrator
---
# k8s/01-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: ulf-config
  namespace: ulf-orchestrator
data:
  ULF_AGENT: "auto"
  ULF_MAX_ITERATIONS: "100"
  ULF_MAX_RUNTIME: "14400"
  ULF_CHECKPOINT_INTERVAL: "5"
  ULF_VERBOSE: "true"
  ULF_ENABLE_METRICS: "true"
```

### 2. Secrets Management

```yaml
# k8s/02-secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: ulf-secrets
  namespace: ulf-orchestrator
type: Opaque
stringData:
  CLAUDE_API_KEY: "sk-ant-..."
  GEMINI_API_KEY: "AIza..."
  Q_API_KEY: "..."
```

Apply secrets from command line:

```bash
# Create secret from literals
kubectl create secret generic ulf-secrets \
  --from-literal=CLAUDE_API_KEY=$CLAUDE_API_KEY \
  --from-literal=GEMINI_API_KEY=$GEMINI_API_KEY \
  --from-literal=Q_API_KEY=$Q_API_KEY \
  -n ulf-orchestrator
```

### 3. Persistent Storage

```yaml
# k8s/03-pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: ulf-workspace
  namespace: ulf-orchestrator
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: ulf-cache
  namespace: ulf-orchestrator
spec:
  accessModes:
    - ReadWriteMany
  storageClassName: standard
  resources:
    requests:
      storage: 5Gi
```

### 4. Deployment

```yaml
# k8s/04-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ulf-orchestrator
  namespace: ulf-orchestrator
  labels:
    app: ulf-orchestrator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: ulf-orchestrator
  template:
    metadata:
      labels:
        app: ulf-orchestrator
    spec:
      serviceAccountName: ulf-sa
      containers:
      - name: ulf
        image: ghcr.io/mikeyobrien/ulf-orchestrator:v1.0.0
        imagePullPolicy: Always
        envFrom:
        - configMapRef:
            name: ulf-config
        - secretRef:
            name: ulf-secrets
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
        volumeMounts:
        - name: workspace
          mountPath: /workspace
        - name: cache
          mountPath: /app/.cache
        - name: prompts
          mountPath: /prompts
        livenessProbe:
          exec:
            command:
            - python
            - -c
            - "import sys; sys.exit(0)"
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          exec:
            command:
            - python
            - -c
            - "import os; sys.exit(0 if os.path.exists('/app/ulf_orchestrator.py') else 1)"
          initialDelaySeconds: 10
          periodSeconds: 10
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: ulf-workspace
      - name: cache
        persistentVolumeClaim:
          claimName: ulf-cache
      - name: prompts
        configMap:
          name: ulf-prompts
```

### 5. Service and Monitoring

```yaml
# k8s/05-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: ulf-metrics
  namespace: ulf-orchestrator
  labels:
    app: ulf-orchestrator
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    name: metrics
  selector:
    app: ulf-orchestrator
---
# k8s/06-servicemonitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: ulf-orchestrator
  namespace: ulf-orchestrator
spec:
  selector:
    matchLabels:
      app: ulf-orchestrator
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### 6. Job for One-Time Tasks

```yaml
# k8s/07-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: ulf-task
  namespace: ulf-orchestrator
spec:
  backoffLimit: 3
  activeDeadlineSeconds: 14400
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: ulf
        image: ghcr.io/mikeyobrien/ulf-orchestrator:v1.0.0
        envFrom:
        - configMapRef:
            name: ulf-config
        - secretRef:
            name: ulf-secrets
        args:
        - "--agent=claude"
        - "--prompt=/prompts/task.md"
        - "--max-iterations=50"
        volumeMounts:
        - name: prompts
          mountPath: /prompts
        - name: output
          mountPath: /output
      volumes:
      - name: prompts
        configMap:
          name: ulf-prompts
      - name: output
        emptyDir: {}
```

## Helm Chart Deployment

### Install with Helm

```bash
# Add repository
helm repo add ulf https://mikeyobrien.github.io/ulf-orchestrator/charts
helm repo update

# Install with custom values
helm install ulf ulf/ulf-orchestrator \
  --namespace ulf-orchestrator \
  --create-namespace \
  --set apiKeys.claude=$CLAUDE_API_KEY \
  --set apiKeys.gemini=$GEMINI_API_KEY \
  --set config.maxIterations=100
```

### Custom values.yaml

```yaml
# values.yaml
replicaCount: 1

image:
  repository: ghcr.io/mikeyobrien/ulf-orchestrator
  tag: v1.0.0
  pullPolicy: IfNotPresent

apiKeys:
  claude: ""
  gemini: ""
  q: ""

config:
  agent: "auto"
  maxIterations: 100
  maxRuntime: 14400
  checkpointInterval: 5
  verbose: true
  enableMetrics: true

resources:
  requests:
    memory: "2Gi"
    cpu: "1"
  limits:
    memory: "4Gi"
    cpu: "2"

persistence:
  enabled: true
  storageClass: "standard"
  workspace:
    size: 10Gi
  cache:
    size: 5Gi

autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80

monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s

ingress:
  enabled: false
  className: "nginx"
  annotations: {}
  hosts:
    - host: ulf.example.com
      paths:
        - path: /
          pathType: Prefix
```

## Horizontal Pod Autoscaling

```yaml
# k8s/08-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ulf-hpa
  namespace: ulf-orchestrator
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ulf-orchestrator
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## CronJob for Scheduled Tasks

```yaml
# k8s/09-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: ulf-daily
  namespace: ulf-orchestrator
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          restartPolicy: OnFailure
          containers:
          - name: ulf
            image: ghcr.io/mikeyobrien/ulf-orchestrator:v1.0.0
            envFrom:
            - configMapRef:
                name: ulf-config
            - secretRef:
                name: ulf-secrets
            args:
            - "--agent=auto"
            - "--prompt=/prompts/daily-task.md"
```

## Service Account and RBAC

```yaml
# k8s/10-rbac.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: ulf-sa
  namespace: ulf-orchestrator
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: ulf-role
  namespace: ulf-orchestrator
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["pods", "pods/log"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: ulf-rolebinding
  namespace: ulf-orchestrator
roleRef:
  apiVersion: rbac.authorization.k8s.io/v1
  kind: Role
  name: ulf-role
subjects:
- kind: ServiceAccount
  name: ulf-sa
  namespace: ulf-orchestrator
```

## Network Policies

```yaml
# k8s/11-networkpolicy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: ulf-network-policy
  namespace: ulf-orchestrator
spec:
  podSelector:
    matchLabels:
      app: ulf-orchestrator
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: TCP
      port: 443  # HTTPS for API calls
    - protocol: TCP
      port: 53   # DNS
    - protocol: UDP
      port: 53   # DNS
```

## Monitoring with Prometheus

```yaml
# k8s/12-prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'ulf-orchestrator'
      kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
          - ulf-orchestrator
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: ulf-orchestrator
```

## Cloud Provider Specific

### Google Kubernetes Engine (GKE)

```bash
# Create cluster
gcloud container clusters create ulf-cluster \
  --zone us-central1-a \
  --num-nodes 3 \
  --machine-type n1-standard-2

# Get credentials
gcloud container clusters get-credentials ulf-cluster \
  --zone us-central1-a

# Create secret for GCR
kubectl create secret docker-registry gcr-json-key \
  --docker-server=gcr.io \
  --docker-username=_json_key \
  --docker-password="$(cat ~/key.json)" \
  -n ulf-orchestrator
```

### Amazon EKS

```bash
# Create cluster
eksctl create cluster \
  --name ulf-cluster \
  --region us-west-2 \
  --nodegroup-name workers \
  --node-type t3.medium \
  --nodes 3

# Update kubeconfig
aws eks update-kubeconfig \
  --name ulf-cluster \
  --region us-west-2
```

### Azure AKS

```bash
# Create cluster
az aks create \
  --resource-group ulf-rg \
  --name ulf-cluster \
  --node-count 3 \
  --node-vm-size Standard_DS2_v2

# Get credentials
az aks get-credentials \
  --resource-group ulf-rg \
  --name ulf-cluster
```

## GitOps with ArgoCD

```yaml
# k8s/argocd-app.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: ulf-orchestrator
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/mikeyobrien/ulf-orchestrator
    targetRevision: HEAD
    path: k8s
  destination:
    server: https://kubernetes.default.svc
    namespace: ulf-orchestrator
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

## Troubleshooting

### Check Pod Status

```bash
# Get pods
kubectl get pods -n ulf-orchestrator

# Describe pod
kubectl describe pod <pod-name> -n ulf-orchestrator

# View logs
kubectl logs -f <pod-name> -n ulf-orchestrator

# Execute into pod
kubectl exec -it <pod-name> -n ulf-orchestrator -- /bin/bash
```

### Common Issues

#### ImagePullBackOff

```bash
# Check image pull secrets
kubectl get secrets -n ulf-orchestrator

# Create pull secret
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=USERNAME \
  --docker-password=TOKEN \
  -n ulf-orchestrator
```

#### PVC Not Bound

```bash
# Check PVC status
kubectl get pvc -n ulf-orchestrator

# Check available storage classes
kubectl get storageclass

# Create PV if needed
kubectl apply -f persistent-volume.yaml
```

#### OOMKilled

```bash
# Increase memory limits
kubectl set resources deployment ulf-orchestrator \
  --limits=memory=8Gi \
  -n ulf-orchestrator
```

## Best Practices

1. **Use namespaces** to isolate Ulf deployments
2. **Implement RBAC** for least privilege access
3. **Use secrets management** (Sealed Secrets, External Secrets)
4. **Set resource limits** to prevent resource starvation
5. **Enable monitoring** with Prometheus/Grafana
6. **Use network policies** for security
7. **Implement health checks** for automatic recovery
8. **Use GitOps** for declarative deployments
9. **Regular backups** of persistent volumes
10. **Use pod disruption budgets** for high availability

## Production Considerations

- **High Availability**: Deploy across multiple availability zones
- **Disaster Recovery**: Regular backups and cross-region replication
- **Security**: Pod Security Policies, Network Policies, RBAC
- **Observability**: Logging (ELK), Metrics (Prometheus), Tracing (Jaeger)
- **Cost Optimization**: Use spot instances, autoscaling, resource quotas
- **Compliance**: Audit logging, encryption at rest and in transit

## Next Steps

- [CI/CD Integration](ci-cd.md) - Automate Kubernetes deployments
- [Production Guide](production.md) - Production best practices
- [Monitoring Setup](../advanced/monitoring.md) - Complete observability