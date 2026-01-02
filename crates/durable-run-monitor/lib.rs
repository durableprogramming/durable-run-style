use chrono::{DateTime, Utc};
use durable_run_analytics::{AnalyticsDashboard, DataPoint};
use durable_run_config_batch::ResourceControl;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, PidExt, ProcessExt, System, SystemExt};
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tokio::time::interval;

#[derive(Debug, Error)]
pub enum MonitorError {
    #[error("System info error: {0}")]
    SysInfo(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, MonitorError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub disk_usage: u64,
    pub disk_total: u64,
    pub process_count: usize,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interface_name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub timestamp: DateTime<Utc>,
}

impl PerformanceMetrics {
    pub fn new(system: &System) -> Self {
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let memory_total = system.total_memory();
        let memory_used = system.used_memory();
        let disk_total = system.disks().iter().map(|d| d.total_space()).sum();
        let disk_used = disk_total - system.disks().iter().map(|d| d.available_space()).sum::<u64>();

        let network_rx_bytes = system.networks().iter().map(|(_, network)| network.received()).sum();
        let network_tx_bytes = system.networks().iter().map(|(_, network)| network.transmitted()).sum();

        Self {
            timestamp: Utc::now(),
            cpu_usage,
            memory_usage: memory_used,
            memory_total,
            disk_usage: disk_used,
            disk_total,
            process_count: system.processes().len(),
            network_rx_bytes,
            network_tx_bytes,
        }
    }
}

pub struct PerformanceProfiler {
    system: Arc<Mutex<System>>,
    analytics: AnalyticsDashboard,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(Mutex::new(system)),
            analytics: AnalyticsDashboard::new(),
        }
    }

    pub fn with_storage(mut self, path: String) -> Self {
        self.analytics = self.analytics.with_storage(path);
        self
    }

    pub async fn load_from_disk(&self) -> Result<()> {
        self.analytics.load_from_disk().await.map_err(|e| MonitorError::SysInfo(e.to_string()))
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        self.analytics.save_to_disk().await.map_err(|e| MonitorError::SysInfo(e.to_string()))
    }

    pub fn snapshot(&self) -> Result<PerformanceMetrics> {
        let mut system = self.system.lock().unwrap();
        system.refresh_all();
        let metrics = PerformanceMetrics::new(&system);

        // Record to analytics
        let timestamp = metrics.timestamp;
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "cpu_usage".to_string(),
            value: metrics.cpu_usage as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "memory_usage".to_string(),
            value: metrics.memory_usage as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "memory_total".to_string(),
            value: metrics.memory_total as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "disk_usage".to_string(),
            value: metrics.disk_usage as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "disk_total".to_string(),
            value: metrics.disk_total as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "process_count".to_string(),
            value: metrics.process_count as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "network_rx_bytes".to_string(),
            value: metrics.network_rx_bytes as f64,
            tags: HashMap::new(),
        });
        self.analytics.collection().record_data_point(DataPoint {
            timestamp,
            metric: "network_tx_bytes".to_string(),
            value: metrics.network_tx_bytes as f64,
            tags: HashMap::new(),
        });

        Ok(metrics)
    }

    pub fn get_process_metrics(&self, limit: Option<usize>) -> Result<Vec<ProcessMetrics>> {
        let system = self.system.lock().unwrap();
        let mut processes: Vec<_> = system.processes().iter().map(|(pid, process)| {
            ProcessMetrics {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
                timestamp: Utc::now(),
            }
        }).collect();

        // Sort by CPU usage descending
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit if specified
        if let Some(limit) = limit {
            processes.truncate(limit);
        }

        Ok(processes)
    }

    pub fn get_network_metrics(&self) -> Result<Vec<NetworkMetrics>> {
        let system = self.system.lock().unwrap();
        let networks: Vec<_> = system.networks().iter().map(|(name, network)| {
            NetworkMetrics {
                interface_name: name.clone(),
                rx_bytes: network.received(),
                tx_bytes: network.transmitted(),
                rx_packets: network.packets_received(),
                tx_packets: network.packets_transmitted(),
                timestamp: Utc::now(),
            }
        }).collect();

        Ok(networks)
    }

    pub fn history(&self) -> Vec<PerformanceMetrics> {
        // Convert analytics data points back to PerformanceMetrics
        let cpu_data = self.analytics.collection().get_data_points("cpu_usage", None);
        let memory_usage_data = self.analytics.collection().get_data_points("memory_usage", None);
        let memory_total_data = self.analytics.collection().get_data_points("memory_total", None);
        let disk_usage_data = self.analytics.collection().get_data_points("disk_usage", None);
        let disk_total_data = self.analytics.collection().get_data_points("disk_total", None);
        let process_data = self.analytics.collection().get_data_points("process_count", None);
        let network_rx_data = self.analytics.collection().get_data_points("network_rx_bytes", None);
        let network_tx_data = self.analytics.collection().get_data_points("network_tx_bytes", None);

        // Group by timestamp and reconstruct PerformanceMetrics
        let mut metrics_map = std::collections::HashMap::new();

        for point in cpu_data {
            metrics_map.entry(point.timestamp).or_insert(PerformanceMetrics {
                timestamp: point.timestamp,
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_total: 0,
                disk_usage: 0,
                disk_total: 0,
                process_count: 0,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
            }).cpu_usage = point.value as f32;
        }

        for point in memory_usage_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.memory_usage = point.value as u64;
            }
        }

        for point in memory_total_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.memory_total = point.value as u64;
            }
        }

        for point in disk_usage_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.disk_usage = point.value as u64;
            }
        }

        for point in disk_total_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.disk_total = point.value as u64;
            }
        }

        for point in process_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.process_count = point.value as usize;
            }
        }

        for point in network_rx_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.network_rx_bytes = point.value as u64;
            }
        }

        for point in network_tx_data {
            if let Some(metrics) = metrics_map.get_mut(&point.timestamp) {
                metrics.network_tx_bytes = point.value as u64;
            }
        }

        let mut result: Vec<_> = metrics_map.into_values().collect();
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        result
    }

    pub fn analytics(&self) -> &AnalyticsDashboard {
        &self.analytics
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrend {
    pub metric: String,
    pub values: Vec<f64>,
    pub timestamps: Vec<DateTime<Utc>>,
}

impl ResourceTrend {
    pub fn new(metric: String) -> Self {
        Self {
            metric,
            values: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    pub fn add_point(&mut self, value: f64, timestamp: DateTime<Utc>) {
        self.values.push(value);
        self.timestamps.push(timestamp);

        // Keep only last 100 points
        if self.values.len() > 100 {
            self.values.remove(0);
            self.timestamps.remove(0);
        }
    }

    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }
}

pub struct ResourceTrendsTracker {
    trends: Arc<Mutex<HashMap<String, ResourceTrend>>>,
}

impl Default for ResourceTrendsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceTrendsTracker {
    pub fn new() -> Self {
        Self {
            trends: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_trend(&self, metric: &str, value: f64) {
        let mut trends = self.trends.lock().unwrap();
        let trend = trends.entry(metric.to_string()).or_insert_with(|| ResourceTrend::new(metric.to_string()));
        trend.add_point(value, Utc::now());
    }

    pub fn get_trend(&self, metric: &str) -> Option<ResourceTrend> {
        self.trends.lock().unwrap().get(metric).cloned()
    }

    pub fn all_trends(&self) -> HashMap<String, ResourceTrend> {
        self.trends.lock().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub metric: String,
    pub threshold: f64,
    pub current_value: f64,
    pub condition: AlertCondition,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub name: String,
    pub description: Option<String>,
    pub monitoring_interval_secs: u64,
    pub storage_path: Option<String>,
    pub enable_persistence: bool,
    pub alert_thresholds: HashMap<String, (f64, AlertCondition)>,
    pub custom_metrics: Vec<String>,
    pub resource_limits: Option<ResourceControl>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            name: "default-monitor".to_string(),
            description: Some("Default system monitor configuration".to_string()),
            monitoring_interval_secs: 60,
            storage_path: None,
            enable_persistence: false,
            alert_thresholds: HashMap::new(),
            custom_metrics: vec!["cpu_usage".to_string(), "memory_usage".to_string(), "disk_usage".to_string()],
            resource_limits: None,
        }
    }
}

impl MonitorConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(MonitorError::Yaml)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(MonitorError::Json)
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(MonitorError::Yaml)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(MonitorError::Json)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| MonitorError::SysInfo(e.to_string()))?;
        Self::from_yaml(&contents)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml).map_err(|e| MonitorError::SysInfo(e.to_string()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(MonitorError::SysInfo("name cannot be empty".to_string()));
        }
        if self.monitoring_interval_secs == 0 {
            return Err(MonitorError::SysInfo("monitoring_interval_secs must be greater than 0".to_string()));
        }
        for (metric, (threshold, _)) in &self.alert_thresholds {
            if metric.trim().is_empty() {
                return Err(MonitorError::SysInfo("alert threshold metric cannot be empty".to_string()));
            }
            if threshold.is_nan() || threshold.is_infinite() {
                return Err(MonitorError::SysInfo(format!("invalid threshold value for metric {}", metric)));
            }

            // Validate thresholds against resource limits if provided
            if let Some(ref limits) = self.resource_limits {
                match metric.as_str() {
                    "cpu_usage" => {
                        if *threshold > limits.max_cpu_percent as f64 {
                            return Err(MonitorError::SysInfo(format!(
                                "CPU threshold {} exceeds resource limit {}",
                                threshold, limits.max_cpu_percent
                            )));
                        }
                    }
                    "memory_usage" => {
                        if *threshold > limits.max_memory_mb as f64 * 1024.0 * 1024.0 { // Convert MB to bytes
                            return Err(MonitorError::SysInfo(format!(
                                "Memory threshold {} exceeds resource limit {} MB",
                                threshold, limits.max_memory_mb
                            )));
                        }
                    }
                    "disk_usage" => {
                        if *threshold > limits.max_disk_mb as f64 * 1024.0 * 1024.0 { // Convert MB to bytes
                            return Err(MonitorError::SysInfo(format!(
                                "Disk threshold {} exceeds resource limit {} MB",
                                threshold, limits.max_disk_mb
                            )));
                        }
                    }
                    _ => {} // Custom metrics don't have resource limits
                }
            }
        }
        Ok(())
    }
}

pub struct CustomMetricsDashboard {
    metrics: Arc<Mutex<HashMap<String, Vec<CustomMetric>>>>,
}

impl Default for CustomMetricsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomMetricsDashboard {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_metric(&self, name: String, value: f64, unit: String) {
        let mut metrics = self.metrics.lock().unwrap();
        let metric_list = metrics.entry(name.clone()).or_default();
        metric_list.push(CustomMetric {
            name: name.clone(),
            value,
            unit,
            timestamp: Utc::now(),
        });

        // Keep only last 1000 metrics per name
        if metric_list.len() > 1000 {
            metric_list.remove(0);
        }
    }

    pub fn get_metrics(&self, name: &str) -> Vec<CustomMetric> {
        self.metrics.lock().unwrap().get(name).cloned().unwrap_or_default()
    }

    pub fn all_metrics(&self) -> HashMap<String, Vec<CustomMetric>> {
        self.metrics.lock().unwrap().clone()
    }
}

pub struct AlertManager {
    thresholds: Arc<Mutex<HashMap<String, (f64, AlertCondition)>>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            thresholds: Arc::new(Mutex::new(HashMap::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_threshold(&self, metric: String, threshold: f64, condition: AlertCondition) {
        self.thresholds.lock().unwrap().insert(metric, (threshold, condition));
    }

    pub fn check_alerts(&self, metric: &str, value: f64) -> Vec<Alert> {
        let thresholds = self.thresholds.lock().unwrap();
        let mut new_alerts = Vec::new();

        if let Some((threshold, condition)) = thresholds.get(metric) {
            let triggered = match condition {
                AlertCondition::Above => value > *threshold,
                AlertCondition::Below => value < *threshold,
                AlertCondition::Equal => (value - *threshold).abs() < f64::EPSILON,
            };

            if triggered {
                let alert = Alert {
                    id: format!("{}_{}", metric, Utc::now().timestamp()),
                    metric: metric.to_string(),
                    threshold: *threshold,
                    current_value: value,
                    condition: condition.clone(),
                    timestamp: Utc::now(),
                    message: format!("Alert: {} is {} threshold {:.2}", metric, 
                        match condition {
                            AlertCondition::Above => "above",
                            AlertCondition::Below => "below",
                            AlertCondition::Equal => "equal to",
                        }, threshold),
                };
                new_alerts.push(alert.clone());
                self.alerts.lock().unwrap().push(alert);
            }
        }

        new_alerts
    }

    pub fn get_alerts(&self) -> Vec<Alert> {
        self.alerts.lock().unwrap().clone()
    }

    pub fn clear_alerts(&self) {
        self.alerts.lock().unwrap().clear();
    }
}

pub struct Monitor {
    profiler: PerformanceProfiler,
    trends: ResourceTrendsTracker,
    dashboard: CustomMetricsDashboard,
    alerts: AlertManager,
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
            trends: ResourceTrendsTracker::new(),
            dashboard: CustomMetricsDashboard::new(),
            alerts: AlertManager::new(),
        }
    }

    pub fn from_config(config: MonitorConfig) -> Result<Self> {
        config.validate()?;

        let mut profiler = PerformanceProfiler::new();
        if let Some(storage_path) = config.storage_path {
            profiler = profiler.with_storage(storage_path);
        }

        let monitor = Self {
            profiler,
            trends: ResourceTrendsTracker::new(),
            dashboard: CustomMetricsDashboard::new(),
            alerts: AlertManager::new(),
        };

        // Set up alert thresholds
        for (metric, (threshold, condition)) in config.alert_thresholds {
            monitor.alerts.set_threshold(metric, threshold, condition);
        }

        Ok(monitor)
    }

    pub fn profiler(&self) -> &PerformanceProfiler {
        &self.profiler
    }

    pub fn trends(&self) -> &ResourceTrendsTracker {
        &self.trends
    }

    pub fn dashboard(&self) -> &CustomMetricsDashboard {
        &self.dashboard
    }

    pub fn alerts(&self) -> &AlertManager {
        &self.alerts
    }

    pub fn record_custom_metric(&self, name: String, value: f64, unit: String) {
        self.dashboard.record_metric(name, value, unit);
    }

    pub fn get_custom_metrics(&self, name: &str) -> Vec<CustomMetric> {
        self.dashboard.get_metrics(name)
    }

    pub fn get_process_metrics(&self, limit: Option<usize>) -> Result<Vec<ProcessMetrics>> {
        self.profiler.get_process_metrics(limit)
    }

    pub fn get_network_metrics(&self) -> Result<Vec<NetworkMetrics>> {
        self.profiler.get_network_metrics()
    }

    pub async fn load_state(&self) -> Result<()> {
        self.profiler.load_from_disk().await
    }

    pub async fn save_state(&self) -> Result<()> {
        self.profiler.save_to_disk().await
    }

    pub fn generate_report(&self, metrics: Vec<String>) -> durable_run_analytics::PerformanceReport {
        self.profiler.analytics.generate_dashboard_report(metrics)
    }

    pub fn analyze_trend(&self, metric: &str) -> Option<durable_run_analytics::TrendAnalysis> {
        self.profiler.analytics.analyze_trend(metric, None)
    }

    pub fn detect_anomalies(&self, metric: &str, z_threshold: f64) -> Option<durable_run_analytics::AnomalyDetection> {
        self.profiler.analytics.detect_anomalies(metric, z_threshold, None)
    }

    pub async fn start_monitoring(&self, interval_secs: u64, cancellation_token: Option<CancellationToken>) {
        let mut interval = interval(tokio::time::Duration::from_secs(interval_secs));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                if let Ok(metrics) = self.profiler.snapshot() {
                    self.trends.update_trend("cpu_usage", metrics.cpu_usage as f64);
                    self.trends.update_trend("memory_usage", metrics.memory_usage as f64);
                    self.trends.update_trend("disk_usage", metrics.disk_usage as f64);
                    self.trends.update_trend("network_rx_bytes", metrics.network_rx_bytes as f64);
                    self.trends.update_trend("network_tx_bytes", metrics.network_tx_bytes as f64);

                    // Check for alerts
                    self.alerts.check_alerts("cpu_usage", metrics.cpu_usage as f64);
                    self.alerts.check_alerts("memory_usage", metrics.memory_usage as f64);
                    self.alerts.check_alerts("disk_usage", metrics.disk_usage as f64);
                    self.alerts.check_alerts("network_rx_bytes", metrics.network_rx_bytes as f64);
                    self.alerts.check_alerts("network_tx_bytes", metrics.network_tx_bytes as f64);
                }
                }
                _ = async {
                    if let Some(token) = &cancellation_token {
                        token.cancelled().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::time::{timeout, Duration};

    #[test]
    fn test_performance_metrics_new() {
        // Mock system for testing
        let system = System::new_all();
        // In a real test, we'd mock the system, but for now assume it works
        let metrics = PerformanceMetrics::new(&system);
        assert!(metrics.timestamp <= Utc::now());
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_total > 0);
    }

    #[test]
    fn test_performance_profiler_new() {
        let profiler = PerformanceProfiler::new();
        assert!(profiler.snapshot().is_ok());
    }

    #[test]
    fn test_performance_profiler_snapshot() {
        let profiler = PerformanceProfiler::new();
        let metrics = profiler.snapshot().unwrap();
        assert!(metrics.cpu_usage >= 0.0);

    }

    #[test]
    fn test_resource_trend_new() {
        let trend = ResourceTrend::new("cpu".to_string());
        assert_eq!(trend.metric, "cpu");
        assert!(trend.values.is_empty());
        assert!(trend.timestamps.is_empty());
    }

    #[test]
    fn test_resource_trend_add_point() {
        let mut trend = ResourceTrend::new("cpu".to_string());
        let now = Utc::now();
        trend.add_point(85.0, now);
        assert_eq!(trend.values.len(), 1);
        assert_eq!(trend.timestamps.len(), 1);
        assert_eq!(trend.values[0], 85.0);
    }

    #[test]
    fn test_resource_trend_average() {
        let mut trend = ResourceTrend::new("cpu".to_string());
        trend.add_point(80.0, Utc::now());
        trend.add_point(90.0, Utc::now());
        assert_eq!(trend.average(), 85.0);
    }

    #[test]
    fn test_resource_trend_average_empty() {
        let trend = ResourceTrend::new("cpu".to_string());
        assert_eq!(trend.average(), 0.0);
    }

    #[test]
    fn test_resource_trends_tracker_new() {
        let tracker = ResourceTrendsTracker::new();
        assert!(tracker.get_trend("cpu").is_none());
    }

    #[test]
    fn test_resource_trends_tracker_update_and_get() {
        let tracker = ResourceTrendsTracker::new();
        tracker.update_trend("cpu", 85.0);
        let trend = tracker.get_trend("cpu").unwrap();
        assert_eq!(trend.metric, "cpu");
        assert_eq!(trend.values.len(), 1);
        assert_eq!(trend.values[0], 85.0);
    }

    #[test]
    fn test_resource_trends_tracker_all_trends() {
        let tracker = ResourceTrendsTracker::new();
        tracker.update_trend("cpu", 85.0);
        tracker.update_trend("memory", 60.0);
        let all = tracker.all_trends();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("cpu"));
        assert!(all.contains_key("memory"));
    }

    #[test]
    fn test_custom_metrics_dashboard_new() {
        let dashboard = CustomMetricsDashboard::new();
        assert!(dashboard.get_metrics("test").is_empty());
    }

    #[test]
    fn test_custom_metrics_dashboard_record_and_get() {
        let dashboard = CustomMetricsDashboard::new();
        dashboard.record_metric("response_time".to_string(), 150.0, "ms".to_string());
        let metrics = dashboard.get_metrics("response_time");
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "response_time");
        assert_eq!(metrics[0].value, 150.0);
        assert_eq!(metrics[0].unit, "ms");
    }

    #[test]
    fn test_custom_metrics_dashboard_all_metrics() {
        let dashboard = CustomMetricsDashboard::new();
        dashboard.record_metric("cpu".to_string(), 80.0, "%".to_string());
        dashboard.record_metric("memory".to_string(), 60.0, "%".to_string());
        let all = dashboard.all_metrics();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_alert_manager_new() {
        let manager = AlertManager::new();
        assert!(manager.get_alerts().is_empty());
    }

    #[test]
    fn test_alert_manager_set_threshold_and_check() {
        let manager = AlertManager::new();
        manager.set_threshold("cpu".to_string(), 90.0, AlertCondition::Above);
        
        // Should not trigger
        let alerts = manager.check_alerts("cpu", 85.0);
        assert!(alerts.is_empty());
        
        // Should trigger
        let alerts = manager.check_alerts("cpu", 95.0);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].metric, "cpu");
        assert_eq!(alerts[0].threshold, 90.0);
        assert_eq!(alerts[0].current_value, 95.0);
    }

    #[test]
    fn test_alert_manager_below_condition() {
        let manager = AlertManager::new();
        manager.set_threshold("memory".to_string(), 20.0, AlertCondition::Below);
        
        let alerts = manager.check_alerts("memory", 15.0);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].message.contains("below"));
    }

    #[test]
    fn test_alert_manager_equal_condition() {
        let manager = AlertManager::new();
        manager.set_threshold("disk".to_string(), 100.0, AlertCondition::Equal);
        
        let alerts = manager.check_alerts("disk", 100.0);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].message.contains("equal to"));
    }

    #[test]
    fn test_alert_manager_get_alerts() {
        let manager = AlertManager::new();
        manager.set_threshold("cpu".to_string(), 90.0, AlertCondition::Above);
        manager.check_alerts("cpu", 95.0);
        
        let alerts = manager.get_alerts();
        assert_eq!(alerts.len(), 1);
    }

    #[test]
    fn test_alert_manager_clear_alerts() {
        let manager = AlertManager::new();
        manager.set_threshold("cpu".to_string(), 90.0, AlertCondition::Above);
        manager.check_alerts("cpu", 95.0);
        manager.clear_alerts();
        assert!(manager.get_alerts().is_empty());
    }

    #[test]
    fn test_monitor_new() {
        let monitor = Monitor::new();
        assert!(monitor.profiler().snapshot().is_ok());
        assert!(monitor.trends().get_trend("cpu").is_none());
        assert!(monitor.dashboard().get_metrics("test").is_empty());
        assert!(monitor.alerts().get_alerts().is_empty());
    }

    #[tokio::test]
    async fn test_monitor_start_monitoring_cancellation() {
        let monitor = Monitor::new();
        let token = CancellationToken::new();
        let token_clone = token.clone();
        
        let monitoring_task = tokio::spawn(async move {
            monitor.start_monitoring(1, Some(token_clone)).await;
        });
        
        // Let it run for a short time
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Cancel
        token.cancel();
        
        // Should complete quickly
        let result = timeout(Duration::from_secs(1), monitoring_task).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = PerformanceMetrics {
            timestamp: Utc::now(),
            cpu_usage: 85.5,
            memory_usage: 1024,
            memory_total: 8192,
            disk_usage: 2048,
            disk_total: 10240,
            process_count: 150,
            network_rx_bytes: 1000000,
            network_tx_bytes: 500000,
        };
        
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.cpu_usage, 85.5);
        assert_eq!(deserialized.memory_usage, 1024);
        assert_eq!(deserialized.process_count, 150);
    }

    #[test]
    fn test_alert_serialization() {
        let alert = Alert {
            id: "test_alert".to_string(),
            metric: "cpu".to_string(),
            threshold: 90.0,
            current_value: 95.0,
            condition: AlertCondition::Above,
            timestamp: Utc::now(),
            message: "CPU above threshold".to_string(),
        };
        
        let json = serde_json::to_string(&alert).unwrap();
        let deserialized: Alert = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.metric, "cpu");
        assert_eq!(deserialized.threshold, 90.0);
        assert_eq!(deserialized.current_value, 95.0);
    }

    #[test]
    fn test_custom_metric_serialization() {
        let metric = CustomMetric {
            name: "response_time".to_string(),
            value: 150.0,
            unit: "ms".to_string(),
            timestamp: Utc::now(),
        };
        
        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: CustomMetric = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.name, "response_time");
        assert_eq!(deserialized.value, 150.0);
        assert_eq!(deserialized.unit, "ms");
    }

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.name, "default-monitor");
        assert_eq!(config.monitoring_interval_secs, 60);
        assert!(!config.enable_persistence);
        assert!(config.storage_path.is_none());
    }

    #[test]
    fn test_monitor_config_new() {
        let config = MonitorConfig::new("test-monitor".to_string());
        assert_eq!(config.name, "test-monitor");
        assert_eq!(config.monitoring_interval_secs, 60);
    }

    #[test]
    fn test_monitor_config_validation() {
        let mut config = MonitorConfig::default();
        assert!(config.validate().is_ok());

        config.name = "".to_string();
        assert!(config.validate().is_err());

        config.name = "test".to_string();
        config.monitoring_interval_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_monitor_config_serialization() {
        let mut config = MonitorConfig::default();
        config.alert_thresholds.insert("cpu".to_string(), (90.0, AlertCondition::Above));

        let yaml = config.to_yaml().unwrap();
        let deserialized = MonitorConfig::from_yaml(&yaml).unwrap();
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.alert_thresholds.len(), 1);
    }

    #[test]
    fn test_monitor_from_config() {
        let mut config = MonitorConfig::default();
        config.alert_thresholds.insert("cpu".to_string(), (90.0, AlertCondition::Above));

        let monitor = Monitor::from_config(config).unwrap();
        assert!(monitor.profiler().snapshot().is_ok());
    }

    #[test]
    fn test_monitor_from_config_invalid() {
        let mut config = MonitorConfig::default();
        config.name = "".to_string();

        assert!(Monitor::from_config(config).is_err());
    }

    #[tokio::test]
    async fn test_monitor_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_monitor.json");

        let mut config = MonitorConfig::default();
        config.storage_path = Some(storage_path.to_string_lossy().to_string());
        config.enable_persistence = true;

        let monitor = Monitor::from_config(config).unwrap();

        // Take a snapshot to create data
        monitor.profiler().snapshot().unwrap();

        // Save state
        monitor.save_state().await.unwrap();

        // Create new monitor and load state
        let new_monitor = Monitor::from_config(MonitorConfig {
            storage_path: Some(storage_path.to_string_lossy().to_string()),
            ..Default::default()
        }).unwrap();

        new_monitor.load_state().await.unwrap();

        // Check that history was loaded
        let history = new_monitor.profiler().history();
        assert!(!history.is_empty());
    }

    #[test]
    fn test_monitor_reporting() {
        let monitor = Monitor::new();

        // Add some data
        monitor.profiler().snapshot().unwrap();

        let report = monitor.generate_report(vec!["cpu_usage".to_string()]);
        assert!(!report.metrics.is_empty());

        let _trend = monitor.analyze_trend("cpu_usage");
        // Trend analysis might return None if insufficient data
        // This is okay for the test
    }

    #[test]
    fn test_monitor_anomaly_detection() {
        let monitor = Monitor::new();

        // Add some data points
        for _ in 0..10 {
            monitor.profiler().snapshot().unwrap();
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let _anomalies = monitor.detect_anomalies("cpu_usage", 2.0);
        // Anomaly detection might return None if no anomalies found
        // This is okay for the test
    }

    #[tokio::test]
    async fn test_monitor_integration_workflow() {
        let monitor = Monitor::new();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        // Test that monitoring can be started and cancelled
        let monitoring_handle = tokio::spawn(async move {
            monitor.start_monitoring(1, Some(token_clone)).await;
        });

        // Let it run briefly
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Cancel monitoring
        token.cancel();

        // Wait for monitoring to stop
        let result = tokio::time::timeout(tokio::time::Duration::from_secs(2), monitoring_handle).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_monitor_data_collection() {
        let monitor = Monitor::new();

        // Take a few snapshots and update trends
        for _ in 0..3 {
            if let Ok(metrics) = monitor.profiler().snapshot() {
                monitor.trends().update_trend("cpu_usage", metrics.cpu_usage as f64);
                monitor.trends().update_trend("memory_usage", metrics.memory_usage as f64);
                monitor.trends().update_trend("disk_usage", metrics.disk_usage as f64);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Check that data was collected
        let history = monitor.profiler().history();
        assert!(!history.is_empty());
        assert!(history.len() >= 3);

        // Check trends
        let cpu_trend = monitor.trends().get_trend("cpu_usage");
        assert!(cpu_trend.is_some());
        assert!(cpu_trend.unwrap().values.len() >= 3);
    }

    #[test]
    fn test_process_metrics() {
        let profiler = PerformanceProfiler::new();
        let processes = profiler.get_process_metrics(Some(5)).unwrap();
        // Should return at most 5 processes
        assert!(processes.len() <= 5);

        // If there are processes, check structure
        if !processes.is_empty() {
            let process = &processes[0];
            assert!(!process.name.is_empty());
            assert!(process.cpu_usage >= 0.0);
            assert!(process.memory_usage >= 0);
        }
    }

    #[test]
    fn test_network_metrics() {
        let profiler = PerformanceProfiler::new();
        let networks = profiler.get_network_metrics().unwrap();

        // Should have at least one network interface (usually loopback)
        assert!(!networks.is_empty());

        for network in networks {
            assert!(!network.interface_name.is_empty());
            assert!(network.rx_bytes >= 0);
            assert!(network.tx_bytes >= 0);
            assert!(network.rx_packets >= 0);
            assert!(network.tx_packets >= 0);
        }
    }

    #[test]
    fn test_custom_metrics_recording() {
        let monitor = Monitor::new();

        monitor.record_custom_metric("response_time".to_string(), 150.0, "ms".to_string());
        monitor.record_custom_metric("error_rate".to_string(), 0.05, "percent".to_string());

        let response_times = monitor.get_custom_metrics("response_time");
        assert_eq!(response_times.len(), 1);
        assert_eq!(response_times[0].name, "response_time");
        assert_eq!(response_times[0].value, 150.0);
        assert_eq!(response_times[0].unit, "ms");

        let error_rates = monitor.get_custom_metrics("error_rate");
        assert_eq!(error_rates.len(), 1);
        assert_eq!(error_rates[0].name, "error_rate");
        assert_eq!(error_rates[0].value, 0.05);
        assert_eq!(error_rates[0].unit, "percent");
    }

    #[test]
    fn test_monitor_config_with_resource_limits() {
        let resource_limits = ResourceControl {
            max_memory_mb: 2048,
            max_cpu_percent: 80.0,
            max_disk_mb: 10240,
            priority: 0,
        };

        let mut config = MonitorConfig {
            resource_limits: Some(resource_limits),
            ..Default::default()
        };

        // Add valid thresholds
        config.alert_thresholds.insert("cpu_usage".to_string(), (70.0, AlertCondition::Above));
        config.alert_thresholds.insert("memory_usage".to_string(), (1024.0 * 1024.0 * 1024.0, AlertCondition::Above)); // 1GB in bytes

        assert!(config.validate().is_ok());

        // Add invalid threshold that exceeds resource limit
        config.alert_thresholds.insert("cpu_usage".to_string(), (90.0, AlertCondition::Above)); // Above 80% limit
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_performance_metrics_with_network() {
        let system = System::new_all();
        let metrics = PerformanceMetrics::new(&system);

        assert!(metrics.network_rx_bytes >= 0);
        assert!(metrics.network_tx_bytes >= 0);

        // Test serialization includes network fields
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.network_rx_bytes, metrics.network_rx_bytes);
        assert_eq!(deserialized.network_tx_bytes, metrics.network_tx_bytes);
    }
}
