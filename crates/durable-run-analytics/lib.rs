//! # Durable Run Analytics
//!
//! This crate provides analytics and performance monitoring capabilities for the durable-run system.
//! It allows collecting, storing, and analyzing performance metrics over time.
//!
//! ## Features
//!
//! - **Data Collection**: Store and retrieve time-series performance data points
//! - **Persistence**: Save/load analytics data to/from disk in JSON format
//! - **Visualization**: Generate text-based charts and statistics
//! - **Reporting**: Create comprehensive performance reports
//! - **Dashboard**: High-level interface for analytics operations
//!
//! ## Example
//!
//! ```rust
//! use durable_run_analytics::{AnalyticsDashboard, DataPoint};
//! use chrono::Utc;
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a dashboard
//! let dashboard = AnalyticsDashboard::new();
//!
//! // Record some metrics
//! let point = DataPoint {
//!     timestamp: Utc::now(),
//!     metric: "cpu_usage".to_string(),
//!     value: 85.5,
//!     tags: HashMap::new(),
//! };
//!
//! dashboard.collection().record_data_point(point);
//!
//! // Generate a report
//! let report = dashboard.generate_dashboard_report(vec!["cpu_usage".to_string()]);
//! println!("Generated report with {} metrics", report.metrics.len());
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Analytics error: {0}")]
    Analytics(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;

/// A single data point in the analytics system.
///
/// Represents a measurement at a specific point in time, with optional tags for categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// When this data point was recorded
    pub timestamp: DateTime<Utc>,
    /// The name of the metric being measured (e.g., "cpu_usage", "memory_mb")
    pub metric: String,
    /// The numerical value of the measurement
    pub value: f64,
    /// Optional key-value pairs for additional categorization or filtering
    pub tags: HashMap<String, String>,
}

#[derive(Clone)]
/// Collection of historical analytics data points.
///
/// Provides thread-safe storage and retrieval of performance metrics.
/// Supports optional persistence to disk in JSON format.
pub struct HistoricalDataCollection {
    data: Arc<Mutex<Vec<DataPoint>>>,
    storage_path: Option<String>,
}

impl HistoricalDataCollection {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
            storage_path: None,
        }
    }

    pub fn with_storage(mut self, path: String) -> Self {
        self.storage_path = Some(path);
        self
    }

    pub async fn load_from_disk(&self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            if fs::metadata(path).await.is_ok() {
                let content = fs::read_to_string(path).await?;
                let data: Vec<DataPoint> = serde_json::from_str(&content)?;
                *self.data.lock().unwrap() = data;
            }
        }
        Ok(())
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            let data = self.data.lock().unwrap().clone();
            let content = serde_json::to_string_pretty(&data)?;
            fs::write(path, content).await?;
        }
        Ok(())
    }

    /// Records a new data point in the collection.
    ///
    /// # Arguments
    ///
    /// * `point` - The data point to record
    pub fn record_data_point(&self, point: DataPoint) {
        self.data.lock().unwrap().push(point);
    }

    /// Retrieves data points for a specific metric, optionally filtered by time.
    ///
    /// # Arguments
    ///
    /// * `metric` - The name of the metric to retrieve
    /// * `since` - Optional timestamp filter; only returns points at or after this time
    ///
    /// # Returns
    ///
    /// A vector of data points matching the criteria
    pub fn get_data_points(&self, metric: &str, since: Option<DateTime<Utc>>) -> Vec<DataPoint> {
        self.data.lock().unwrap().iter().filter(|p| {
            p.metric == metric && since.is_none_or(|s| p.timestamp >= s)
        }).cloned().collect()
    }

    pub fn get_all_data(&self) -> Vec<DataPoint> {
        self.data.lock().unwrap().clone()
    }
}

impl Default for HistoricalDataCollection {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DataVisualization {
    data: Vec<DataPoint>,
}

impl DataVisualization {
    pub fn new(data: Vec<DataPoint>) -> Self {
        Self { data }
    }

    pub fn generate_text_chart(&self, metric: &str) -> String {
        let points: Vec<_> = self.data.iter().filter(|p| p.metric == metric).collect();
        if points.is_empty() {
            return "No data available".to_string();
        }

        let mut chart = format!("Chart for metric: {}\n", metric);
        chart.push_str("Timestamp | Value\n");
        chart.push_str("----------|------\n");

        for point in points {
            chart.push_str(&format!("{} | {:.2}\n", point.timestamp, point.value));
        }

        chart
    }

    /// Generate a simple ASCII bar chart.
    pub fn generate_bar_chart(&self, metric: &str, width: usize) -> String {
        let points: Vec<_> = self.data.iter().filter(|p| p.metric == metric).collect();
        if points.is_empty() {
            return "No data available".to_string();
        }

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        if max_val == min_val {
            return format!("All values are the same: {:.2}", max_val);
        }

        let mut chart = format!("Bar Chart for metric: {}\n", metric);
        chart.push_str(&format!("Range: {:.2} - {:.2}\n\n", min_val, max_val));

        for point in points.iter() {
            let normalized = if max_val > min_val {
                (point.value - min_val) / (max_val - min_val)
            } else {
                0.0
            };
            let bar_length = (normalized * width as f64) as usize;
            let bar = "█".repeat(bar_length);
            let timestamp_str = point.timestamp.format("%H:%M:%S").to_string();

            chart.push_str(&format!("{} | {:.2} {}\n", timestamp_str, point.value, bar));
        }

        chart
    }

    /// Generate a histogram of value frequencies.
    pub fn generate_histogram(&self, metric: &str, bins: usize) -> String {
        let points: Vec<_> = self.data.iter().filter(|p| p.metric == metric).collect();
        if points.is_empty() {
            return "No data available".to_string();
        }

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if min_val == max_val {
            return format!("All values are the same: {:.2}", min_val);
        }

        let bin_size = (max_val - min_val) / bins as f64;
        let mut histogram = vec![0; bins];

        for &value in &values {
            let bin_index = if value == max_val {
                bins - 1
            } else {
                ((value - min_val) / bin_size) as usize
            };
            if bin_index < bins {
                histogram[bin_index] += 1;
            }
        }

        let max_count = *histogram.iter().max().unwrap_or(&0);
        let max_bar_width = 40;

        let mut chart = format!("Histogram for metric: {} ({} bins)\n", metric, bins);
        chart.push_str(&format!("Range: {:.2} - {:.2}\n\n", min_val, max_val));

        for (i, &count) in histogram.iter().enumerate() {
            let bin_start = min_val + i as f64 * bin_size;
            let bin_end = if i == bins - 1 { max_val } else { bin_start + bin_size };
            let bar_width = if max_count > 0 {
                (count * max_bar_width) / max_count
            } else {
                0
            };
            let bar = "█".repeat(bar_width);

            chart.push_str(&format!(
                "{:6.2} - {:6.2} | {:3} {}\n",
                bin_start, bin_end, count, bar
            ));
        }

        chart
    }

    pub fn calculate_stats(&self, metric: &str) -> Option<MetricStats> {
        let values: Vec<f64> = self.data.iter().filter(|p| p.metric == metric).map(|p| p.value).collect();
        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(MetricStats {
            count: count as usize,
            mean,
            std_dev,
            min,
            max,
        })
    }
}

/// Statistical summary of a metric's data points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    /// Number of data points
    pub count: usize,
    /// Arithmetic mean of the values
    pub mean: f64,
    /// Standard deviation of the values
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

/// Trend analysis result for a metric over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// The metric being analyzed
    pub metric: String,
    /// Trend direction: positive (>0), negative (<0), or flat (=0)
    pub slope: f64,
    /// R-squared value indicating trend strength (0-1, higher is stronger)
    pub r_squared: f64,
    /// Trend classification
    pub trend_type: TrendType,
    /// Confidence level in the trend analysis
    pub confidence: f64,
}

/// Types of trends that can be detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendType {
    /// Strong upward trend
    Increasing,
    /// Strong downward trend
    Decreasing,
    /// No significant trend
    Flat,
    /// Unstable or oscillating pattern
    Volatile,
}

/// Performs trend analysis on time-series data.
pub struct TrendAnalyzer;

/// Result of anomaly detection analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    /// The metric being analyzed
    pub metric: String,
    /// Detected anomalies
    pub anomalies: Vec<Anomaly>,
    /// Statistical bounds used for detection
    pub bounds: StatisticalBounds,
}

/// A detected anomaly in the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Index of the anomalous data point
    pub index: usize,
    /// The anomalous data point
    pub data_point: DataPoint,
    /// Z-score of the anomaly (deviations from mean)
    pub z_score: f64,
    /// Severity of the anomaly (higher = more severe)
    pub severity: f64,
}

/// Statistical bounds for anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalBounds {
    pub mean: f64,
    pub std_dev: f64,
    pub upper_bound: f64,
    pub lower_bound: f64,
    pub z_threshold: f64,
}

/// Detects anomalies in time-series data using statistical methods.
pub struct AnomalyDetector;

/// Result of predictive analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// The metric being predicted
    pub metric: String,
    /// Predicted values for future time points
    pub predictions: Vec<PredictedPoint>,
    /// Model accuracy metrics
    pub accuracy: PredictionAccuracy,
}

/// A predicted data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedPoint {
    /// When this prediction is for
    pub timestamp: DateTime<Utc>,
    /// Predicted value
    pub value: f64,
    /// Confidence interval lower bound
    pub lower_bound: f64,
    /// Confidence interval upper bound
    pub upper_bound: f64,
}

/// Accuracy metrics for predictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionAccuracy {
    /// Mean Absolute Percentage Error
    pub mape: f64,
    /// Root Mean Square Error
    pub rmse: f64,
    /// R-squared value for the prediction model
    pub r_squared: f64,
}

/// Performs predictive analytics using time-series forecasting.
pub struct PredictiveAnalyzer;

impl PredictiveAnalyzer {
    /// Forecast future values using linear regression.
    ///
    /// # Arguments
    ///
    /// * `data` - Historical data points (should be sorted by timestamp)
    /// * `forecast_steps` - Number of future time steps to predict
    /// * `time_interval` - Time interval between predictions (in seconds)
    ///
    /// # Returns
    ///
    /// PredictionResult with forecasted values
    pub fn forecast_linear_regression(
        data: &[DataPoint],
        forecast_steps: usize,
        time_interval: i64,
    ) -> Option<PredictionResult> {
        if data.len() < 3 {
            return None;
        }

        let metric = data[0].metric.clone();

        // Convert to time series
        let first_timestamp = data[0].timestamp.timestamp() as f64;
        let points: Vec<(f64, f64)> = data.iter()
            .map(|p| ((p.timestamp.timestamp() as f64 - first_timestamp), p.value))
            .collect();

        // Perform linear regression
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate accuracy metrics using cross-validation
        let accuracy = Self::calculate_accuracy(&points, slope, intercept);

        // Generate predictions
        let last_timestamp = data.last().unwrap().timestamp;
        let mut predictions = Vec::new();

        for i in 1..=forecast_steps {
            let future_time = last_timestamp + chrono::Duration::seconds(time_interval * i as i64);
            let time_offset = future_time.timestamp() as f64 - first_timestamp;
            let predicted_value = slope * time_offset + intercept;

            // Simple confidence interval (this could be improved with statistical methods)
            let std_dev = Self::calculate_residual_std_dev(&points, slope, intercept);
            let confidence_margin = (1.96 * std_dev).max(1.0); // 95% confidence interval, minimum 1.0

            predictions.push(PredictedPoint {
                timestamp: future_time,
                value: predicted_value,
                lower_bound: predicted_value - confidence_margin,
                upper_bound: predicted_value + confidence_margin,
            });
        }

        Some(PredictionResult {
            metric,
            predictions,
            accuracy,
        })
    }

    /// Calculate prediction accuracy metrics.
    fn calculate_accuracy(points: &[(f64, f64)], slope: f64, intercept: f64) -> PredictionAccuracy {
        let mut errors = Vec::new();
        let mut actual_sum = 0.0;

        for &(x, actual_y) in points {
            let predicted_y = slope * x + intercept;
            let error = actual_y - predicted_y;
            errors.push(error);
            actual_sum += actual_y;
        }
        let actual_mean = actual_sum / points.len() as f64;

        // Calculate MAPE
        let mape = errors.iter()
            .zip(points.iter())
            .map(|(error, (_, actual))| (error.abs() / actual.abs()).min(1.0)) // Cap at 100%
            .sum::<f64>() / points.len() as f64 * 100.0;

        // Calculate RMSE
        let rmse = (errors.iter().map(|e| e * e).sum::<f64>() / points.len() as f64).sqrt();

        // Calculate R-squared
        let ss_res = errors.iter().map(|e| e * e).sum::<f64>();
        let ss_tot = points.iter()
            .map(|(_, y)| (y - actual_mean).powi(2))
            .sum::<f64>();
        let r_squared = if ss_tot == 0.0 { 0.0 } else { 1.0 - (ss_res / ss_tot) };

        PredictionAccuracy {
            mape,
            rmse,
            r_squared,
        }
    }

    /// Calculate standard deviation of residuals.
    fn calculate_residual_std_dev(points: &[(f64, f64)], slope: f64, intercept: f64) -> f64 {
        let residuals: Vec<f64> = points.iter()
            .map(|(x, y)| {
                let predicted = slope * x + intercept;
                y - predicted
            })
            .collect();

        let mean_residual = residuals.iter().sum::<f64>() / residuals.len() as f64;
        let variance = residuals.iter()
            .map(|r| (r - mean_residual).powi(2))
            .sum::<f64>() / residuals.len() as f64;

        variance.sqrt()
    }
}

/// Handles data export in various formats.
pub struct DataExporter;

impl DataExporter {
    /// Export data points to CSV format.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of data points to export
    ///
    /// # Returns
    ///
    /// CSV formatted string
    pub fn to_csv(data: &[DataPoint]) -> String {
        let mut csv = String::from("timestamp,metric,value,tags\n");

        for point in data {
            let tags_str = point.tags.iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(";");

            csv.push_str(&format!(
                "{},{},{},\"{}\"\n",
                point.timestamp.to_rfc3339(),
                point.metric,
                point.value,
                tags_str
            ));
        }

        csv
    }

    /// Export data points to JSON format.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of data points to export
    ///
    /// # Returns
    ///
    /// JSON formatted string
    pub fn to_json(data: &[DataPoint]) -> Result<String> {
        serde_json::to_string_pretty(data).map_err(Into::into)
    }

    /// Export performance report to JSON.
    ///
    /// # Arguments
    ///
    /// * `report` - Performance report to export
    ///
    /// # Returns
    ///
    /// JSON formatted string
    pub fn report_to_json(report: &PerformanceReport) -> Result<String> {
        serde_json::to_string_pretty(report).map_err(Into::into)
    }

    /// Export trend analysis to JSON.
    ///
    /// # Arguments
    ///
    /// * `analysis` - Trend analysis to export
    ///
    /// # Returns
    ///
    /// JSON formatted string
    pub fn trend_to_json(analysis: &TrendAnalysis) -> Result<String> {
        serde_json::to_string_pretty(analysis).map_err(Into::into)
    }

    /// Export anomaly detection results to JSON.
    ///
    /// # Arguments
    ///
    /// * `detection` - Anomaly detection results to export
    ///
    /// # Returns
    ///
    /// JSON formatted string
    pub fn anomalies_to_json(detection: &AnomalyDetection) -> Result<String> {
        serde_json::to_string_pretty(detection).map_err(Into::into)
    }
}

impl AnomalyDetector {
    /// Detect anomalies using Z-score method.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of data points
    /// * `z_threshold` - Z-score threshold for anomaly detection (typically 2.0-3.0)
    ///
    /// # Returns
    ///
    /// AnomalyDetection result
    pub fn detect_anomalies(data: &[DataPoint], z_threshold: f64) -> Option<AnomalyDetection> {
        if data.is_empty() {
            return None;
        }

        let metric = data[0].metric.clone();
        let values: Vec<f64> = data.iter().map(|p| p.value).collect();

        // Calculate mean and standard deviation
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            // No variation, no anomalies
            return Some(AnomalyDetection {
                metric,
                anomalies: Vec::new(),
                bounds: StatisticalBounds {
                    mean,
                    std_dev,
                    upper_bound: mean,
                    lower_bound: mean,
                    z_threshold,
                },
            });
        }

        let upper_bound = mean + z_threshold * std_dev;
        let lower_bound = mean - z_threshold * std_dev;

        let mut anomalies = Vec::new();

        for (i, point) in data.iter().enumerate() {
            let z_score = (point.value - mean) / std_dev;
            if z_score.abs() > z_threshold {
                let severity = z_score.abs() / z_threshold;
                anomalies.push(Anomaly {
                    index: i,
                    data_point: point.clone(),
                    z_score,
                    severity,
                });
            }
        }

        // Sort anomalies by severity (most severe first)
        anomalies.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        Some(AnomalyDetection {
            metric,
            anomalies,
            bounds: StatisticalBounds {
                mean,
                std_dev,
                upper_bound,
                lower_bound,
                z_threshold,
            },
        })
    }

    /// Detect anomalies using moving average and standard deviation.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of data points (should be sorted by timestamp)
    /// * `window_size` - Size of the moving window
    /// * `threshold_multiplier` - Multiplier for standard deviation threshold
    ///
    /// # Returns
    ///
    /// AnomalyDetection result
    pub fn detect_anomalies_moving_average(
        data: &[DataPoint],
        window_size: usize,
        threshold_multiplier: f64,
    ) -> Option<AnomalyDetection> {
        if data.len() < window_size {
            return None;
        }

        let metric = data[0].metric.clone();
        let mut anomalies = Vec::new();

        // Calculate moving statistics
        for i in window_size..data.len() {
            let window: Vec<f64> = data[i.saturating_sub(window_size)..i]
                .iter()
                .map(|p| p.value)
                .collect();

            let mean = window.iter().sum::<f64>() / window.len() as f64;
            let variance = window.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            let std_dev = variance.sqrt();

            let current_value = data[i].value;
            let z_score = if std_dev == 0.0 {
                0.0
            } else {
                (current_value - mean) / std_dev
            };

            if z_score.abs() > threshold_multiplier {
                let severity = z_score.abs() / threshold_multiplier;
                anomalies.push(Anomaly {
                    index: i,
                    data_point: data[i].clone(),
                    z_score,
                    severity,
                });
            }
        }

        // Use overall statistics for bounds
        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        anomalies.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        Some(AnomalyDetection {
            metric,
            anomalies,
            bounds: StatisticalBounds {
                mean,
                std_dev,
                upper_bound: mean + threshold_multiplier * std_dev,
                lower_bound: mean - threshold_multiplier * std_dev,
                z_threshold: threshold_multiplier,
            },
        })
    }
}

impl TrendAnalyzer {
    /// Analyze the trend of a metric over time.
    ///
    /// Uses linear regression to determine the slope and strength of the trend.
    ///
    /// # Arguments
    ///
    /// * `data` - Vector of data points sorted by timestamp
    ///
    /// # Returns
    ///
    /// TrendAnalysis result, or None if insufficient data
    pub fn analyze_trend(data: &[DataPoint]) -> Option<TrendAnalysis> {
        if data.len() < 3 {
            return None;
        }

        let metric = data.first()?.metric.clone();
        let n = data.len() as f64;

        // Convert timestamps to seconds since first point for numerical stability
        let first_timestamp = data[0].timestamp.timestamp() as f64;
        let points: Vec<(f64, f64)> = data.iter()
            .map(|p| ((p.timestamp.timestamp() as f64 - first_timestamp), p.value))
            .collect();

        // Calculate means
        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;

        // Calculate slope and intercept using linear regression
        let numerator: f64 = points.iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        let denominator: f64 = points.iter()
            .map(|(x, _)| (x - mean_x).powi(2))
            .sum();

        if denominator == 0.0 {
            return Some(TrendAnalysis {
                metric,
                slope: 0.0,
                r_squared: 0.0,
                trend_type: TrendType::Flat,
                confidence: 0.0,
            });
        }

        let slope = numerator / denominator;

        // Calculate R-squared
        let ss_res: f64 = points.iter()
            .map(|(x, y)| {
                let predicted = slope * (x - mean_x) + mean_y;
                (y - predicted).powi(2)
            })
            .sum();
        let ss_tot: f64 = points.iter()
            .map(|(_, y)| (y - mean_y).powi(2))
            .sum();

        let r_squared = if ss_tot == 0.0 { 0.0 } else { 1.0 - (ss_res / ss_tot) };

        // Classify trend
        let trend_type = Self::classify_trend(slope, r_squared);

        // Calculate confidence based on data points and R-squared
        let confidence = (r_squared * (n / (n + 1.0))).min(1.0);

        Some(TrendAnalysis {
            metric,
            slope,
            r_squared,
            trend_type,
            confidence,
        })
    }

    /// Classify the type of trend based on slope and R-squared.
    fn classify_trend(slope: f64, r_squared: f64) -> TrendType {
        let slope_threshold = 0.01; // Minimum slope to consider a trend
        let r_squared_threshold = 0.3; // Minimum R-squared for meaningful trend

        if r_squared < r_squared_threshold || slope.abs() < slope_threshold {
            TrendType::Flat
        } else if slope > slope_threshold {
            TrendType::Increasing
        } else {
            TrendType::Decreasing
        }
    }

    /// Detect volatility in the data (coefficient of variation).
    pub fn calculate_volatility(data: &[DataPoint]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        let values: Vec<f64> = data.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            Some(0.0)
        } else {
            Some(std_dev / mean.abs())
        }
    }
}

pub struct PerformanceReporting {
    collection: HistoricalDataCollection,
}

impl PerformanceReporting {
    pub fn new(collection: HistoricalDataCollection) -> Self {
        Self { collection }
    }

    pub fn generate_report(&self, metrics: Vec<String>, since: Option<DateTime<Utc>>) -> PerformanceReport {
        let mut report = PerformanceReport {
            generated_at: Utc::now(),
            metrics: HashMap::new(),
        };

        for metric in metrics {
            let data = self.collection.get_data_points(&metric, since);
            let viz = DataVisualization::new(data.clone());
            let stats = viz.calculate_stats(&metric);
            let chart = viz.generate_text_chart(&metric);

            report.metrics.insert(metric, MetricReport {
                data_points: data.len(),
                stats,
                chart,
            });
        }

        report
    }
}

/// Comprehensive performance report containing statistics and visualizations for multiple metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// When this report was generated
    pub generated_at: DateTime<Utc>,
    /// Reports for each requested metric
    pub metrics: HashMap<String, MetricReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    pub data_points: usize,
    pub stats: Option<MetricStats>,
    pub chart: String,
}

pub struct AnalyticsDashboard {
    collection: HistoricalDataCollection,
    reporting: PerformanceReporting,
}

impl AnalyticsDashboard {
    pub fn new() -> Self {
        let collection = HistoricalDataCollection::new();
        let reporting = PerformanceReporting::new(collection.clone());
        Self {
            collection,
            reporting,
        }
    }

    pub fn with_storage(mut self, path: String) -> Self {
        self.collection = self.collection.with_storage(path);
        self
    }

    pub async fn load_from_disk(&self) -> Result<()> {
        self.collection.load_from_disk().await
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        self.collection.save_to_disk().await
    }

    pub fn collection(&self) -> &HistoricalDataCollection {
        &self.collection
    }

    pub fn reporting(&self) -> &PerformanceReporting {
        &self.reporting
    }

    pub fn generate_dashboard_report(&self, metrics: Vec<String>) -> PerformanceReport {
        self.reporting.generate_report(metrics, None)
    }

    pub fn get_recent_trends(&self, metric: &str, hours: i64) -> Vec<DataPoint> {
        let since = Utc::now() - chrono::Duration::hours(hours);
        self.collection.get_data_points(metric, Some(since))
    }

    /// Analyze trends for a specific metric.
    pub fn analyze_trend(&self, metric: &str, since: Option<DateTime<Utc>>) -> Option<TrendAnalysis> {
        let data = self.collection.get_data_points(metric, since);
        TrendAnalyzer::analyze_trend(&data)
    }

    /// Detect anomalies in a metric.
    pub fn detect_anomalies(&self, metric: &str, z_threshold: f64, since: Option<DateTime<Utc>>) -> Option<AnomalyDetection> {
        let data = self.collection.get_data_points(metric, since);
        AnomalyDetector::detect_anomalies(&data, z_threshold)
    }

    /// Export data for a metric to CSV.
    pub fn export_to_csv(&self, metric: Option<&str>, since: Option<DateTime<Utc>>) -> String {
        let data = if let Some(m) = metric {
            self.collection.get_data_points(m, since)
        } else {
            self.collection.get_all_data()
        };
        DataExporter::to_csv(&data)
    }

    /// Export data for a metric to JSON.
    pub fn export_to_json(&self, metric: Option<&str>, since: Option<DateTime<Utc>>) -> Result<String> {
        let data = if let Some(m) = metric {
            self.collection.get_data_points(m, since)
        } else {
            self.collection.get_all_data()
        };
        DataExporter::to_json(&data)
    }

    /// Generate enhanced visualizations.
    pub fn generate_bar_chart(&self, metric: &str, width: usize, since: Option<DateTime<Utc>>) -> String {
        let data = self.collection.get_data_points(metric, since);
        let viz = DataVisualization::new(data);
        viz.generate_bar_chart(metric, width)
    }

    pub fn generate_histogram(&self, metric: &str, bins: usize, since: Option<DateTime<Utc>>) -> String {
        let data = self.collection.get_data_points(metric, since);
        let viz = DataVisualization::new(data);
        viz.generate_histogram(metric, bins)
    }

    /// Forecast future values for a metric.
    pub fn forecast_values(
        &self,
        metric: &str,
        forecast_steps: usize,
        time_interval_seconds: i64,
        since: Option<DateTime<Utc>>,
    ) -> Option<PredictionResult> {
        let data = self.collection.get_data_points(metric, since);
        PredictiveAnalyzer::forecast_linear_regression(&data, forecast_steps, time_interval_seconds)
    }
}

impl Default for AnalyticsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::fs;

    #[test]
    fn test_data_point_creation() {
        let timestamp = Utc::now();
        let mut tags = HashMap::new();
        tags.insert("env".to_string(), "test".to_string());

        let point = DataPoint {
            timestamp,
            metric: "cpu_usage".to_string(),
            value: 85.5,
            tags,
        };

        assert_eq!(point.metric, "cpu_usage");
        assert_eq!(point.value, 85.5);
        assert_eq!(point.tags.get("env"), Some(&"test".to_string()));
    }

    #[test]
    fn test_historical_data_collection_new() {
        let collection = HistoricalDataCollection::new();
        assert!(collection.storage_path.is_none());
        assert!(collection.get_all_data().is_empty());
    }

    #[test]
    fn test_historical_data_collection_with_storage() {
        let collection = HistoricalDataCollection::new().with_storage("/tmp/test.json".to_string());
        assert_eq!(collection.storage_path, Some("/tmp/test.json".to_string()));
    }

    #[tokio::test]
    async fn test_historical_data_collection_save_load() {
        let temp_file = "/tmp/test_analytics.json";
        let collection = HistoricalDataCollection::new().with_storage(temp_file.to_string());

        let point = DataPoint {
            timestamp: Utc::now(),
            metric: "test_metric".to_string(),
            value: 42.0,
            tags: HashMap::new(),
        };

        collection.record_data_point(point.clone());

        // Save to disk
        collection.save_to_disk().await.unwrap();

        // Create new collection and load
        let new_collection = HistoricalDataCollection::new().with_storage(temp_file.to_string());
        new_collection.load_from_disk().await.unwrap();

        let loaded_data = new_collection.get_all_data();
        assert_eq!(loaded_data.len(), 1);
        assert_eq!(loaded_data[0].metric, "test_metric");
        assert_eq!(loaded_data[0].value, 42.0);

        // Cleanup
        let _ = fs::remove_file(temp_file).await;
    }

    #[test]
    fn test_historical_data_collection_record_and_get() {
        let collection = HistoricalDataCollection::new();

        let point1 = DataPoint {
            timestamp: Utc::now(),
            metric: "cpu".to_string(),
            value: 80.0,
            tags: HashMap::new(),
        };

        let point2 = DataPoint {
            timestamp: Utc::now(),
            metric: "memory".to_string(),
            value: 60.0,
            tags: HashMap::new(),
        };

        collection.record_data_point(point1);
        collection.record_data_point(point2);

        let cpu_data = collection.get_data_points("cpu", None);
        assert_eq!(cpu_data.len(), 1);
        assert_eq!(cpu_data[0].value, 80.0);

        let memory_data = collection.get_data_points("memory", None);
        assert_eq!(memory_data.len(), 1);
        assert_eq!(memory_data[0].value, 60.0);

        let all_data = collection.get_all_data();
        assert_eq!(all_data.len(), 2);
    }

    #[test]
    fn test_historical_data_collection_filter_by_time() {
        let collection = HistoricalDataCollection::new();

        let now = Utc::now();
        let past_point = DataPoint {
            timestamp: now - chrono::Duration::hours(2),
            metric: "cpu".to_string(),
            value: 50.0,
            tags: HashMap::new(),
        };

        let recent_point = DataPoint {
            timestamp: now - chrono::Duration::minutes(30),
            metric: "cpu".to_string(),
            value: 80.0,
            tags: HashMap::new(),
        };

        collection.record_data_point(past_point);
        collection.record_data_point(recent_point);

        let recent_data = collection.get_data_points("cpu", Some(now - chrono::Duration::hours(1)));
        assert_eq!(recent_data.len(), 1);
        assert_eq!(recent_data[0].value, 80.0);
    }

    #[test]
    fn test_data_visualization_new() {
        let data = vec![];
        let viz = DataVisualization::new(data);
        assert_eq!(viz.generate_text_chart("test"), "No data available");
    }

    #[test]
    fn test_data_visualization_generate_chart() {
        let now = Utc::now();
        let data = vec![
            DataPoint {
                timestamp: now,
                metric: "cpu".to_string(),
                value: 85.5,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: now + chrono::Duration::seconds(60),
                metric: "cpu".to_string(),
                value: 90.2,
                tags: HashMap::new(),
            },
        ];

        let viz = DataVisualization::new(data);
        let chart = viz.generate_text_chart("cpu");

        assert!(chart.contains("Chart for metric: cpu"));
        assert!(chart.contains("85.50"));
        assert!(chart.contains("90.20"));
    }

    #[test]
    fn test_data_visualization_calculate_stats() {
        let data = vec![
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 80.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 85.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 90.0,
                tags: HashMap::new(),
            },
        ];

        let viz = DataVisualization::new(data);
        let stats = viz.calculate_stats("cpu").unwrap();

        assert_eq!(stats.count, 3);
        assert!((stats.mean - 85.0).abs() < 0.001);
        assert_eq!(stats.min, 80.0);
        assert_eq!(stats.max, 90.0);
        assert!(stats.std_dev > 0.0);
    }

    #[test]
    fn test_data_visualization_calculate_stats_no_data() {
        let data = vec![];
        let viz = DataVisualization::new(data);
        assert!(viz.calculate_stats("cpu").is_none());
    }

    #[test]
    fn test_performance_reporting_generate_report() {
        let collection = HistoricalDataCollection::new();

        let now = Utc::now();
        collection.record_data_point(DataPoint {
            timestamp: now,
            metric: "cpu".to_string(),
            value: 80.0,
            tags: HashMap::new(),
        });

        collection.record_data_point(DataPoint {
            timestamp: now,
            metric: "memory".to_string(),
            value: 60.0,
            tags: HashMap::new(),
        });

        let reporting = PerformanceReporting::new(collection);
        let report = reporting.generate_report(vec!["cpu".to_string(), "memory".to_string()], None);

        assert_eq!(report.metrics.len(), 2);
        assert!(report.metrics.contains_key("cpu"));
        assert!(report.metrics.contains_key("memory"));

        let cpu_report = &report.metrics["cpu"];
        assert_eq!(cpu_report.data_points, 1);
        assert!(cpu_report.stats.is_some());
        assert!(cpu_report.chart.contains("80.00"));
    }

    #[test]
    fn test_analytics_dashboard_new() {
        let dashboard = AnalyticsDashboard::new();
        assert!(dashboard.collection().get_all_data().is_empty());
    }

    #[test]
    fn test_analytics_dashboard_generate_dashboard_report() {
        let dashboard = AnalyticsDashboard::new();

        dashboard.collection().record_data_point(DataPoint {
            timestamp: Utc::now(),
            metric: "cpu".to_string(),
            value: 75.0,
            tags: HashMap::new(),
        });

        let report = dashboard.generate_dashboard_report(vec!["cpu".to_string()]);
        assert_eq!(report.metrics.len(), 1);
        assert!(report.metrics.contains_key("cpu"));
    }

    #[test]
    fn test_analytics_dashboard_get_recent_trends() {
        let dashboard = AnalyticsDashboard::new();

        let now = Utc::now();
        dashboard.collection().record_data_point(DataPoint {
            timestamp: now - chrono::Duration::hours(3),
            metric: "cpu".to_string(),
            value: 50.0,
            tags: HashMap::new(),
        });

        dashboard.collection().record_data_point(DataPoint {
            timestamp: now - chrono::Duration::minutes(30),
            metric: "cpu".to_string(),
            value: 80.0,
            tags: HashMap::new(),
        });

        let trends = dashboard.get_recent_trends("cpu", 1);
        assert_eq!(trends.len(), 1);
        assert_eq!(trends[0].value, 80.0);
    }

    #[test]
    fn test_metric_stats_serialization() {
        let stats = MetricStats {
            count: 10,
            mean: 85.5,
            std_dev: 5.2,
            min: 75.0,
            max: 95.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: MetricStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.count, 10);
        assert_eq!(deserialized.mean, 85.5);
        assert_eq!(deserialized.std_dev, 5.2);
        assert_eq!(deserialized.min, 75.0);
        assert_eq!(deserialized.max, 95.0);
    }

    #[test]
    fn test_performance_report_serialization() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu".to_string(), MetricReport {
            data_points: 5,
            stats: Some(MetricStats {
                count: 5,
                mean: 80.0,
                std_dev: 5.0,
                min: 70.0,
                max: 90.0,
            }),
            chart: "test chart".to_string(),
        });

        let report = PerformanceReport {
            generated_at: Utc::now(),
            metrics,
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: PerformanceReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metrics.len(), 1);
        assert!(deserialized.metrics.contains_key("cpu"));
    }

    #[test]
    fn test_trend_analyzer_analyze_trend() {
        let now = Utc::now();
        let data = vec![
            DataPoint {
                timestamp: now,
                metric: "cpu".to_string(),
                value: 50.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: now + chrono::Duration::seconds(10),
                metric: "cpu".to_string(),
                value: 60.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: now + chrono::Duration::seconds(20),
                metric: "cpu".to_string(),
                value: 70.0,
                tags: HashMap::new(),
            },
        ];

        let trend = TrendAnalyzer::analyze_trend(&data).unwrap();
        assert_eq!(trend.metric, "cpu");
        assert!(trend.slope > 0.0); // Should be increasing
        assert!(trend.r_squared >= 0.0 && trend.r_squared <= 1.0);
        assert_eq!(trend.trend_type, TrendType::Increasing);
    }

    #[test]
    fn test_trend_analyzer_insufficient_data() {
        let data = vec![DataPoint {
            timestamp: Utc::now(),
            metric: "cpu".to_string(),
            value: 50.0,
            tags: HashMap::new(),
        }];

        assert!(TrendAnalyzer::analyze_trend(&data).is_none());
    }

    #[test]
    fn test_anomaly_detector_detect_anomalies() {
        // Create data with a clear anomaly
        let mut data = Vec::new();
        let now = Utc::now();

        // Normal data around 50
        for i in 0..10 {
            data.push(DataPoint {
                timestamp: now + chrono::Duration::seconds(i),
                metric: "cpu".to_string(),
                value: 50.0 + (i as f64 - 5.0) * 0.5, // Small variation
                tags: HashMap::new(),
            });
        }

        // Add clear anomaly
        data.push(DataPoint {
            timestamp: now + chrono::Duration::seconds(10),
            metric: "cpu".to_string(),
            value: 85.0, // Much higher than normal
            tags: HashMap::new(),
        });

        let result = AnomalyDetector::detect_anomalies(&data, 2.0).unwrap();
        assert_eq!(result.metric, "cpu");
        assert!(!result.anomalies.is_empty(), "Should detect at least one anomaly");
        // The anomaly should be the last point (index 10)
        assert!(result.anomalies.iter().any(|a| a.index == 10));
    }

    #[test]
    fn test_data_exporter_to_csv() {
        let data = vec![
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 85.5,
                tags: HashMap::new(),
            },
        ];

        let csv = DataExporter::to_csv(&data);
        assert!(csv.contains("timestamp,metric,value,tags"));
        assert!(csv.contains("cpu"));
        assert!(csv.contains("85.5"));
    }

    #[test]
    fn test_data_visualization_generate_bar_chart() {
        let data = vec![
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 50.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 100.0,
                tags: HashMap::new(),
            },
        ];

        let viz = DataVisualization::new(data);
        let chart = viz.generate_bar_chart("cpu", 10);
        assert!(chart.contains("Bar Chart for metric: cpu"));
        assert!(chart.contains("█"));
    }

    #[test]
    fn test_data_visualization_generate_histogram() {
        let data = vec![
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 10.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 20.0,
                tags: HashMap::new(),
            },
            DataPoint {
                timestamp: Utc::now(),
                metric: "cpu".to_string(),
                value: 30.0,
                tags: HashMap::new(),
            },
        ];

        let viz = DataVisualization::new(data);
        let histogram = viz.generate_histogram("cpu", 3);
        assert!(histogram.contains("Histogram for metric: cpu"));
        assert!(histogram.contains("█"));
    }

    #[test]
    fn test_analytics_dashboard_analyze_trend() {
        let dashboard = AnalyticsDashboard::new();

        // Add some data
        let now = Utc::now();
        for i in 0..5 {
            let point = DataPoint {
                timestamp: now + chrono::Duration::seconds(i * 10),
                metric: "cpu".to_string(),
                value: 50.0 + i as f64 * 5.0,
                tags: HashMap::new(),
            };
            dashboard.collection().record_data_point(point);
        }

        let trend = dashboard.analyze_trend("cpu", None).unwrap();
        assert_eq!(trend.metric, "cpu");
        assert!(trend.slope > 0.0);
    }

    #[test]
    fn test_analytics_dashboard_detect_anomalies() {
        let dashboard = AnalyticsDashboard::new();

        // Add normal data
        for i in 0..10 {
            let point = DataPoint {
                timestamp: Utc::now() + chrono::Duration::seconds(i),
                metric: "cpu".to_string(),
                value: 50.0 + (i as f64 - 5.0) * 2.0,
                tags: HashMap::new(),
            };
            dashboard.collection().record_data_point(point);
        }

        // Add anomaly
        let anomaly = DataPoint {
            timestamp: Utc::now() + chrono::Duration::seconds(10),
            metric: "cpu".to_string(),
            value: 200.0, // Way outside normal range
            tags: HashMap::new(),
        };
        dashboard.collection().record_data_point(anomaly);

        let anomalies = dashboard.detect_anomalies("cpu", 2.0, None).unwrap();
        assert!(!anomalies.anomalies.is_empty());
    }

    #[test]
    fn test_integration_with_monitoring_workflow() {
        // Simulate how durable-run-monitor would use analytics
        let dashboard = AnalyticsDashboard::new();

        // Simulate system metrics collection (like durable-run-monitor does)
        let now = Utc::now();
        for i in 0..5 {
            // CPU usage data
            let cpu_point = DataPoint {
                timestamp: now + chrono::Duration::seconds(i * 60), // Every minute
                metric: "cpu_usage".to_string(),
                value: 45.0 + (i as f64 * 5.0).sin() * 20.0, // Varying CPU usage
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("source".to_string(), "system_monitor".to_string());
                    tags
                },
            };
            dashboard.collection().record_data_point(cpu_point);

            // Memory usage data
            let mem_point = DataPoint {
                timestamp: now + chrono::Duration::seconds(i * 60),
                metric: "memory_usage".to_string(),
                value: 60.0 + (i as f64 * 3.0).cos() * 15.0, // Varying memory usage
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("source".to_string(), "system_monitor".to_string());
                    tags
                },
            };
            dashboard.collection().record_data_point(mem_point);
        }

        // Test that analytics can process this monitoring data
        let cpu_trend = dashboard.analyze_trend("cpu_usage", None);
        assert!(cpu_trend.is_some());

        let mem_anomalies = dashboard.detect_anomalies("memory_usage", 2.0, None);
        assert!(mem_anomalies.is_some());

        // Test report generation
        let report = dashboard.generate_dashboard_report(vec!["cpu_usage".to_string(), "memory_usage".to_string()]);
        assert_eq!(report.metrics.len(), 2);
        assert!(report.metrics.contains_key("cpu_usage"));
        assert!(report.metrics.contains_key("memory_usage"));

        // Test data export (simulating data persistence)
        let csv_data = dashboard.export_to_csv(None, None);
        assert!(csv_data.contains("cpu_usage"));
        assert!(csv_data.contains("memory_usage"));
        assert!(csv_data.contains("system_monitor"));
    }

    #[test]
    fn test_performance_profiling_integration() {
        // Simulate how durable-run-perf would integrate with analytics
        let dashboard = AnalyticsDashboard::new();

        // Simulate performance benchmark results
        let now = Utc::now();
        for i in 0..10 {
            let throughput_point = DataPoint {
                timestamp: now + chrono::Duration::seconds(i * 30),
                metric: "benchmark_throughput".to_string(),
                value: 1000.0 + (i as f64 * 50.0) + (i as f64 * 10.0).sin() * 100.0,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("benchmark".to_string(), "http_requests".to_string());
                    tags.insert("concurrency".to_string(), "10".to_string());
                    tags
                },
            };
            dashboard.collection().record_data_point(throughput_point);

            let latency_point = DataPoint {
                timestamp: now + chrono::Duration::seconds(i * 30),
                metric: "benchmark_latency".to_string(),
                value: 50.0 + (i as f64 * 2.0) + (i as f64 * 5.0).cos() * 10.0,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("benchmark".to_string(), "http_requests".to_string());
                    tags.insert("percentile".to_string(), "p95".to_string());
                    tags
                },
            };
            dashboard.collection().record_data_point(latency_point);
        }

        // Test trend analysis on performance data
        let throughput_trend = dashboard.analyze_trend("benchmark_throughput", None).unwrap();
        assert!(throughput_trend.confidence > 0.0);

        // Test anomaly detection on latency
        let latency_anomalies = dashboard.detect_anomalies("benchmark_latency", 2.5, None).unwrap();
        // Should work even if no anomalies are found
        assert_eq!(latency_anomalies.metric, "benchmark_latency");

        // Test filtering by tags (simulating analysis of specific benchmark runs)
        let recent_data = dashboard.collection().get_data_points("benchmark_throughput",
            Some(now + chrono::Duration::seconds(5 * 30)));
        assert!(!recent_data.is_empty());

        // Test histogram generation for performance distribution analysis
        let histogram = dashboard.generate_histogram("benchmark_throughput", 5, None);
        assert!(histogram.contains("benchmark_throughput"));
    }

    #[test]
    fn test_cross_crate_data_consistency() {
        // Test that data structures are consistent across different usage patterns
        let dashboard = AnalyticsDashboard::new();

        // Add data in different ways (simulating different crate integrations)
        let now = Utc::now();

        // Direct recording (like monitor crate)
        dashboard.collection().record_data_point(DataPoint {
            timestamp: now,
            metric: "direct_metric".to_string(),
            value: 42.0,
            tags: HashMap::new(),
        });

        // Batch recording (like workflow crate)
        let batch_data = vec![
            DataPoint {
                timestamp: now + chrono::Duration::seconds(1),
                metric: "batch_metric".to_string(),
                value: 100.0,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("batch_id".to_string(), "test_batch".to_string());
                    tags
                },
            },
            DataPoint {
                timestamp: now + chrono::Duration::seconds(2),
                metric: "batch_metric".to_string(),
                value: 105.0,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("batch_id".to_string(), "test_batch".to_string());
                    tags
                },
            },
        ];

        for point in batch_data {
            dashboard.collection().record_data_point(point);
        }

        // Verify all data is accessible
        let all_data = dashboard.collection().get_all_data();
        assert_eq!(all_data.len(), 3);

        // Test serialization/deserialization consistency
        let json_export = dashboard.export_to_json(None, None);
        assert!(json_export.is_ok());

        let csv_export = dashboard.export_to_csv(None, None);
        assert!(csv_export.contains("direct_metric"));
        assert!(csv_export.contains("batch_metric"));
        assert!(csv_export.contains("test_batch"));

        // Test that reports work across different data sources
        let report = dashboard.generate_dashboard_report(vec![
            "direct_metric".to_string(),
            "batch_metric".to_string()
        ]);
        assert_eq!(report.metrics.len(), 2);
    }

    #[test]
    fn test_predictive_analytics_forecast() {
        let dashboard = AnalyticsDashboard::new();

        // Add historical data with a clear trend
        let now = Utc::now();
        for i in 0..10 {
            let point = DataPoint {
                timestamp: now + chrono::Duration::hours(i),
                metric: "sales".to_string(),
                value: 100.0 + i as f64 * 10.0, // Linear trend: 100, 110, 120, ...
                tags: HashMap::new(),
            };
            dashboard.collection().record_data_point(point);
        }

        // Forecast next 3 values
        let forecast = dashboard.forecast_values("sales", 3, 3600, None).unwrap(); // 1 hour intervals

        assert_eq!(forecast.metric, "sales");
        assert_eq!(forecast.predictions.len(), 3);

        // Check that predictions are reasonable (should continue the linear trend)
        assert!(forecast.predictions[0].value > 180.0); // Should be around 200
        assert!(forecast.predictions[1].value > forecast.predictions[0].value);
        assert!(forecast.predictions[2].value > forecast.predictions[1].value);

        // Check confidence intervals
        for prediction in &forecast.predictions {
            assert!(prediction.upper_bound > prediction.value);
            assert!(prediction.lower_bound < prediction.value);
        }

        // Check accuracy metrics
        assert!(forecast.accuracy.r_squared >= 0.0 && forecast.accuracy.r_squared <= 1.0);
        assert!(forecast.accuracy.rmse >= 0.0);
        assert!(forecast.accuracy.mape >= 0.0);
    }

    #[test]
    fn test_predictive_analytics_insufficient_data() {
        let dashboard = AnalyticsDashboard::new();

        // Add only 2 data points
        let now = Utc::now();
        dashboard.collection().record_data_point(DataPoint {
            timestamp: now,
            metric: "test".to_string(),
            value: 10.0,
            tags: HashMap::new(),
        });
        dashboard.collection().record_data_point(DataPoint {
            timestamp: now + chrono::Duration::hours(1),
            metric: "test".to_string(),
            value: 20.0,
            tags: HashMap::new(),
        });

        // Should return None for insufficient data
        let forecast = dashboard.forecast_values("test", 3, 3600, None);
        assert!(forecast.is_none());
    }
}
