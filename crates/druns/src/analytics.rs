use crate::cli::AnalyticsSubcommands;
use durable_run_analytics::{AnalyticsDashboard, DataExporter};
use chrono::Utc;
use std::collections::HashMap;

pub async fn handle_analytics(subcommand: &AnalyticsSubcommands) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load existing analytics data, or create new dashboard
    let dashboard = AnalyticsDashboard::new();

    // For demo purposes, add some sample data if none exists
    if dashboard.collection().get_all_data().is_empty() {
        add_sample_data(&dashboard);
    }

    match subcommand {
        AnalyticsSubcommands::Collect => {
            println!("Collecting analytics data...");
            // In a real implementation, this would collect data from running processes
            // For now, we'll add some sample data
            add_sample_data(&dashboard);
            println!("Sample data collected successfully.");
        }
        AnalyticsSubcommands::Show => {
            println!("Analytics Data Overview");
            println!("======================");

            let all_data = dashboard.collection().get_all_data();
            let metrics: std::collections::HashSet<_> = all_data.iter().map(|p| &p.metric).collect();

            println!("Total data points: {}", all_data.len());
            println!("Metrics tracked: {}", metrics.len());

            for metric in metrics {
                let metric_data = dashboard.collection().get_data_points(metric, None);
                println!("\nMetric: {}", metric);
                println!("  Data points: {}", metric_data.len());
                if let Some(stats) = dashboard.reporting().generate_report(vec![metric.clone()], None).metrics.get(metric) {
                    if let Some(metric_stats) = &stats.stats {
                        println!("  Mean: {:.2}", metric_stats.mean);
                        println!("  Min: {:.2}, Max: {:.2}", metric_stats.min, metric_stats.max);
                    }
                }
            }
        }
        AnalyticsSubcommands::Stats => {
            println!("Analytics Statistics");
            println!("===================");

            let all_data = dashboard.collection().get_all_data();
            let metrics: std::collections::HashSet<_> = all_data.iter().map(|p| &p.metric).collect();

            for metric in metrics {
                println!("\nStatistics for metric: {}", metric);

                // Basic stats
                if let Some(stats) = dashboard.reporting().generate_report(vec![metric.clone()], None).metrics.get(metric) {
                    if let Some(metric_stats) = &stats.stats {
                        println!("  Count: {}", metric_stats.count);
                        println!("  Mean: {:.2}", metric_stats.mean);
                        println!("  Std Dev: {:.2}", metric_stats.std_dev);
                        println!("  Min: {:.2}", metric_stats.min);
                        println!("  Max: {:.2}", metric_stats.max);
                    }
                }

                // Trend analysis
                if let Some(trend) = dashboard.analyze_trend(metric, None) {
                    println!("  Trend: {:?}", trend.trend_type);
                    println!("  Slope: {:.4}", trend.slope);
                    println!("  R-squared: {:.4}", trend.r_squared);
                    println!("  Confidence: {:.2}%", trend.confidence * 100.0);
                }

                // Anomaly detection
                if let Some(anomalies) = dashboard.detect_anomalies(metric, 2.0, None) {
                    println!("  Anomalies detected: {}", anomalies.anomalies.len());
                    if !anomalies.anomalies.is_empty() {
                        println!("  Most severe anomaly: {:.2} (z-score: {:.2})",
                                anomalies.anomalies[0].data_point.value,
                                anomalies.anomalies[0].z_score);
                    }
                }
            }
        }
        AnalyticsSubcommands::Chart => {
            println!("Analytics Charts");
            println!("===============");

            let all_data = dashboard.collection().get_all_data();
            let metrics: std::collections::HashSet<_> = all_data.iter().map(|p| &p.metric).collect();

            for metric in metrics {
                println!("\nChart for metric: {}", metric);
                println!("{}", "─".repeat(50));

                let chart = dashboard.generate_bar_chart(metric, 20, None);
                println!("{}", chart);

                // Also show histogram
                println!("\nHistogram for metric: {}", metric);
                println!("{}", "─".repeat(50));
                let histogram = dashboard.generate_histogram(metric, 10, None);
                println!("{}", histogram);
            }
        }
        AnalyticsSubcommands::Report => {
            println!("Analytics Report");
            println!("===============");

            let all_data = dashboard.collection().get_all_data();
            let metrics: Vec<String> = all_data.iter().map(|p| p.metric.clone()).collect();

            let report = dashboard.generate_dashboard_report(metrics);
            println!("Report generated at: {}", report.generated_at);

            for (metric_name, metric_report) in &report.metrics {
                println!("\nMetric: {}", metric_name);
                println!("  Data points: {}", metric_report.data_points);

                if let Some(stats) = &metric_report.stats {
                    println!("  Statistics:");
                    println!("    Mean: {:.2}", stats.mean);
                    println!("    Std Dev: {:.2}", stats.std_dev);
                    println!("    Range: {:.2} - {:.2}", stats.min, stats.max);
                }

                println!("  Chart:");
                println!("{}", metric_report.chart);
            }

            // Export to JSON
            let json_report = DataExporter::report_to_json(&report)?;
            println!("\nJSON Export (first 500 chars):");
            println!("{}", &json_report[..json_report.len().min(500)]);
        }
        AnalyticsSubcommands::Dashboard => {
            println!("Analytics Dashboard");
            println!("==================");

            let all_data = dashboard.collection().get_all_data();
            let metrics: std::collections::HashSet<_> = all_data.iter().map(|p| &p.metric).collect();

            println!("📊 Analytics Dashboard Overview");
            println!("Total Metrics: {}", metrics.len());
            println!("Total Data Points: {}", all_data.len());
            println!("Last Updated: {}", Utc::now());

            println!("\n📈 Key Metrics Summary:");
            for metric in &metrics {
                let data = dashboard.collection().get_data_points(metric, None);
                if let Some(stats) = dashboard.reporting().generate_report(vec![metric.to_string()], None).metrics.get(*metric) {
                    if let Some(metric_stats) = &stats.stats {
                        println!("  {}: {:.2} (avg), {} points", metric, metric_stats.mean, data.len());
                    }
                }
            }

            println!("\n📉 Trend Analysis:");
            for metric in &metrics {
                if let Some(trend) = dashboard.analyze_trend(metric, None) {
                    let trend_icon = match trend.trend_type {
                        durable_run_analytics::TrendType::Increasing => "↗️",
                        durable_run_analytics::TrendType::Decreasing => "↘️",
                        durable_run_analytics::TrendType::Flat => "➡️",
                        durable_run_analytics::TrendType::Volatile => "🔄",
                    };
                    println!("  {} {} ({:.1}% confidence)", metric, trend_icon, trend.confidence * 100.0);
                }
            }

            println!("\n🚨 Anomaly Detection:");
            for metric in &metrics {
                if let Some(anomalies) = dashboard.detect_anomalies(metric, 2.0, None) {
                    if anomalies.anomalies.is_empty() {
                        println!("  {}: No anomalies detected", metric);
                    } else {
                        println!("  {}: {} anomalies detected", metric, anomalies.anomalies.len());
                    }
                }
            }

            println!("\n💾 Data Export Options:");
            println!("  Use 'durable-run analytics show' for detailed data");
            println!("  Use 'durable-run analytics stats' for statistics");
            println!("  Use 'durable-run analytics chart' for visualizations");
            println!("  Use 'durable-run analytics report' for comprehensive reports");
        }
    }

    Ok(())
}

fn add_sample_data(dashboard: &AnalyticsDashboard) {
    use durable_run_analytics::DataPoint;
    use chrono::Duration;

    let now = Utc::now();
    let mut rng = 42u32; // Simple LCG for reproducible "randomness"

    // Generate CPU usage data
    for i in 0..20 {
        let base_value = 50.0 + (i as f64 * 0.5).sin() * 20.0;
        let noise = (rng % 100) as f64 - 50.0;
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);

        let point = DataPoint {
            timestamp: now - Duration::minutes(20 - i),
            metric: "cpu_usage".to_string(),
            value: (base_value + noise * 0.1).max(0.0).min(100.0),
            tags: HashMap::new(),
        };
        dashboard.collection().record_data_point(point);
    }

    // Generate memory usage data
    for i in 0..20 {
        let base_value = 60.0 + (i as f64 * 0.3).cos() * 15.0;
        let noise = (rng % 100) as f64 - 50.0;
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);

        let point = DataPoint {
            timestamp: now - Duration::minutes(20 - i),
            metric: "memory_usage".to_string(),
            value: (base_value + noise * 0.05).max(0.0).min(100.0),
            tags: HashMap::new(),
        };
        dashboard.collection().record_data_point(point);
    }

    // Add some anomalies
    let anomaly1 = DataPoint {
        timestamp: now - Duration::minutes(10),
        metric: "cpu_usage".to_string(),
        value: 95.0, // High CPU spike
        tags: HashMap::new(),
    };
    dashboard.collection().record_data_point(anomaly1);

    let anomaly2 = DataPoint {
        timestamp: now - Duration::minutes(5),
        metric: "memory_usage".to_string(),
        value: 15.0, // Low memory (possible issue)
        tags: HashMap::new(),
    };
    dashboard.collection().record_data_point(anomaly2);
}
