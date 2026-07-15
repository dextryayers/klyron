use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChart {
    pub name: String,
    pub version: String,
    pub description: String,
    pub api_version: String,
    pub app_version: String,
    pub home: String,
    pub sources: Vec<String>,
    pub maintainers: Vec<Maintainer>,
    pub dependencies: Vec<HelmDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    pub name: String,
    pub email: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmDependency {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub condition: Option<String>,
    pub tags: Vec<String>,
}

impl HelmChart {
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            api_version: "v2".into(),
            app_version: version.to_string(),
            home: format!("https://github.com/klyron/{name}"),
            sources: vec![format!("https://github.com/klyron/{name}")],
            maintainers: vec![Maintainer {
                name: "Klyron Team".into(),
                email: "team@klyron.dev".into(),
                url: "https://klyron.dev".into(),
            }],
            dependencies: Vec::new(),
        }
    }

    fn chart_yaml(&self) -> String {
        let deps_yaml: String = self
            .dependencies
            .iter()
            .map(|d| {
                let cond = d.condition.as_ref().map(|c| format!("\n    condition: {c}")).unwrap_or_default();
                format!("    - name: {}\n      version: \"{}\"\n      repository: {}{cond}", d.name, d.version, d.repository)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let deps_section = if self.dependencies.is_empty() {
            String::new()
        } else {
            format!("dependencies:\n{deps_yaml}\n")
        };
        format!(
            r#"apiVersion: {api_version}
name: {name}
description: "{description}"
type: application
version: {version}
appVersion: "{app_version}"
home: {home}
sources:
{sources}
maintainers:
  - name: {m_name}
    email: {m_email}
    url: {m_url}
{deps_section}"#,
            api_version = self.api_version,
            name = self.name,
            description = self.description,
            version = self.version,
            app_version = self.app_version,
            home = self.home,
            sources = self.sources.iter().map(|s| format!("  - {s}")).collect::<Vec<_>>().join("\n"),
            m_name = self.maintainers.first().map(|m| m.name.as_str()).unwrap_or("Klyron Team"),
            m_email = self.maintainers.first().map(|m| m.email.as_str()).unwrap_or("team@klyron.dev"),
            m_url = self.maintainers.first().map(|m| m.url.as_str()).unwrap_or("https://klyron.dev"),
        )
    }

    fn values_yaml() -> String {
        r#"# Default values for klyron deployment.
# This is a YAML-formatted file.
# Declare variables to be passed into your templates.

replicaCount: 3

image:
  repository: ghcr.io/klyron/klyron
  pullPolicy: Always
  tag: ""

imagePullSecrets: []
nameOverride: ""
fullnameOverride: ""

serviceAccount:
  create: true
  automount: true
  annotations: {}
  name: ""

podAnnotations: {}
podLabels: {}

podSecurityContext:
  fsGroup: 2000

securityContext:
  capabilities:
    drop:
      - ALL
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 1000

service:
  type: ClusterIP
  port: 80
  targetPort: 3000

ingress:
  enabled: false
  className: ""
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: klyron.local
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: klyron-tls
      hosts:
        - klyron.local

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

env:
  - name: NODE_ENV
    value: "production"
  - name: LOG_LEVEL
    value: "info"

secrets:
  - name: DATABASE_URL
    key: database-url
  - name: API_KEY
    key: api-key

nodeSelector: {}
tolerations: []
affinity: {}

persistence:
  enabled: false
  storageClass: ""
  accessMode: ReadWriteOnce
  size: 10Gi
  annotations: {}
"#.into()
    }

    fn helpers_tpl() -> String {
        r#"{{- define "klyron.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "klyron.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{- define "klyron.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "klyron.labels" -}}
helm.sh/chart: {{ include "klyron.chart" . }}
{{ include "klyron.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "klyron.selectorLabels" -}}
app.kubernetes.io/name: {{ include "klyron.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "klyron.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "klyron.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
"#.into()
    }

    fn deployment_yaml() -> String {
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "klyron.fullname" . }}
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "klyron.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      {{- with .Values.podAnnotations }}
      annotations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      labels:
        {{- include "klyron.labels" . | nindent 8 }}
        {{- with .Values.podLabels }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "klyron.serviceAccountName" . }}
      securityContext:
        {{- toYaml .Values.podSecurityContext | nindent 8 }}
      containers:
        - name: {{ .Chart.Name }}
          securityContext:
            {{- toYaml .Values.securityContext | nindent 12 }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: http
              containerPort: {{ .Values.service.targetPort }}
              protocol: TCP
          livenessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 10
            periodSeconds: 30
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 5
            periodSeconds: 10
            timeoutSeconds: 3
            successThreshold: 1
            failureThreshold: 3
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          env:
            {{- toYaml .Values.env | nindent 12 }}
            {{- if .Values.secrets }}
            {{- range .Values.secrets }}
            - name: {{ .name }}
              valueFrom:
                secretKeyRef:
                  name: {{ include "klyron.fullname" $ }}-secrets
                  key: {{ .key }}
            {{- end }}
            {{- end }}
          volumeMounts:
            - name: config
              mountPath: /app/config
              readOnly: true
            {{- if .Values.persistence.enabled }}
            - name: data
              mountPath: /app/data
            {{- end }}
      volumes:
        - name: config
          configMap:
            name: {{ include "klyron.fullname" . }}-config
        {{- if .Values.persistence.enabled }}
        - name: data
          persistentVolumeClaim:
            claimName: {{ include "klyron.fullname" . }}-pvc
        {{- end }}
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
"#.into()
    }

    fn service_yaml() -> String {
        r#"apiVersion: v1
kind: Service
metadata:
  name: {{ include "klyron.fullname" . }}
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: {{ .Values.service.targetPort }}
      protocol: TCP
      name: http
  selector:
    {{- include "klyron.selectorLabels" . | nindent 4 }}
"#.into()
    }

    fn ingress_yaml() -> String {
        r#"{{- if .Values.ingress.enabled -}}
{{- $fullName := include "klyron.fullname" . -}}
{{- $svcPort := .Values.service.port -}}
{{- if and .Values.ingress.className (not (semverCompare ">=1.18-0" .Capabilities.KubeVersion.GitVersion)) }}
  {{- if not (hasKey .Values.ingress.annotations "kubernetes.io/ingress.class") }}
  {{- $_ := set .Values.ingress.annotations "kubernetes.io/ingress.class" .Values.ingress.className }}
  {{- end }}
{{- end }}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ $fullName }}
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if and .Values.ingress.className (semverCompare ">=1.18-0" .Capabilities.KubeVersion.GitVersion) }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            {{- if and .pathType (semverCompare ">=1.18-0" $.Capabilities.KubeVersion.GitVersion) }}
            pathType: {{ .pathType }}
            {{- end }}
            backend:
              service:
                name: {{ $fullName }}
                port:
                  number: {{ $svcPort }}
          {{- end }}
    {{- end }}
{{- end }}
"#.into()
    }

    fn hpa_yaml() -> String {
        r#"{{- if .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "klyron.fullname" . }}
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "klyron.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
  metrics:
    {{- if .Values.autoscaling.targetCPUUtilizationPercentage }}
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetCPUUtilizationPercentage }}
    {{- end }}
    {{- if .Values.autoscaling.targetMemoryUtilizationPercentage }}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetMemoryUtilizationPercentage }}
    {{- end }}
{{- end }}
"#.into()
    }

    fn configmap_yaml() -> String {
        r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "klyron.fullname" . }}-config
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
data:
  klyron.yaml: |
    server:
      port: {{ .Values.service.targetPort }}
    logging:
      level: {{ (index .Values.env 1).value }}
"#.into()
    }

    fn secret_yaml() -> String {
        r#"apiVersion: v1
kind: Secret
metadata:
  name: {{ include "klyron.fullname" . }}-secrets
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
type: Opaque
data:
  {{- range .Values.secrets }}
  {{ .key }}: {{ (index $.Values.secrets (sub (len $.Values.secrets) 1)).name | b64enc | quote }}
  {{- end }}
"#.into()
    }

    fn pvc_yaml() -> String {
        r#"{{- if .Values.persistence.enabled }}
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {{ include "klyron.fullname" . }}-pvc
  {{- with .Values.persistence.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
  labels:
    {{- include "klyron.labels" . | nindent 4 }}
spec:
  accessModes:
    - {{ .Values.persistence.accessMode }}
  resources:
    requests:
      storage: {{ .Values.persistence.size }}
  {{- if .Values.persistence.storageClass }}
  {{- if (eq "-" .Values.persistence.storageClass) }}
  storageClassName: ""
  {{- else }}
  storageClassName: "{{ .Values.persistence.storageClass }}"
  {{- end }}
  {{- end }}
{{- end }}
"#.into()
    }

    fn notes_txt() -> String {
        r#"1. Get the application URL by running these commands:
{{- if .Values.ingress.enabled }}
{{- range $host := .Values.ingress.hosts }}
  {{- range .paths }}
  http{{ if $.Values.ingress.tls }}s{{ end }}://{{ $host.host }}{{ .path }}
  {{- end }}
{{- end }}
{{- else if contains "NodePort" .Values.service.type }}
  export NODE_PORT=$(kubectl get --namespace {{ .Release.Namespace }} -o jsonpath="{.spec.ports[0].nodePort}" services {{ include "klyron.fullname" . }})
  export NODE_IP=$(kubectl get nodes --namespace {{ .Release.Namespace }} -o jsonpath="{.items[0].status.addresses[0].address}")
  echo http://$NODE_IP:$NODE_PORT
{{- else if contains "LoadBalancer" .Values.service.type }}
  NOTE: It may take a few minutes for the LoadBalancer IP to be available.
  kubectl get svc -w {{ include "klyron.fullname" . }}
{{- end }}

2. Access the health endpoint:
   curl http://localhost/health

3. View pod logs:
   kubectl logs -f deployment/{{ include "klyron.fullname" . }}

4. Check HPA status:
   kubectl get hpa {{ include "klyron.fullname" . }}

5. Clean up:
   helm uninstall {{ .Release.Name }}
"#.into()
    }

    pub fn generate_chart(&self, output_dir: &Path) -> Result<PathBuf, String> {
        let chart_dir = output_dir.join(&self.name);
        let templates_dir = chart_dir.join("templates");
        fs::create_dir_all(&templates_dir).map_err(|e| format!("Cannot create chart directory: {e}"))?;

        let files: Vec<(&str, &str, &str)> = vec![
            ("Chart.yaml", "", &self.chart_yaml()),
            ("values.yaml", "", &Self::values_yaml()),
            ("templates", "_helpers.tpl", &Self::helpers_tpl()),
            ("templates", "deployment.yaml", &Self::deployment_yaml()),
            ("templates", "service.yaml", &Self::service_yaml()),
            ("templates", "ingress.yaml", &Self::ingress_yaml()),
            ("templates", "hpa.yaml", &Self::hpa_yaml()),
            ("templates", "configmap.yaml", &Self::configmap_yaml()),
            ("templates", "secret.yaml", &Self::secret_yaml()),
            ("templates", "pvc.yaml", &Self::pvc_yaml()),
            ("templates", "NOTES.txt", &Self::notes_txt()),
        ];

        for (subdir, filename, content) in &files {
            let path = if *subdir == "templates" {
                chart_dir.join("templates").join(filename)
            } else {
                chart_dir.join(filename)
            };
            fs::write(&path, content).map_err(|e| format!("Cannot write {filename}: {e}"))?;
        }

        Ok(chart_dir)
    }
}
