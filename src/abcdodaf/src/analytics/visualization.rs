//! Visualization data structures for charting and UI integration
//!
//! Provides:
//! - Chart data formats (time series, heatmaps, etc.)
//! - Custom metric definitions
//! - Visualization configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main visualization data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    /// Visualization ID
    pub id: String,
    /// Visualization type
    pub viz_type: VisualizationType,
    /// Chart data
    pub chart_data: ChartData,
    /// Configuration
    pub config: VisualizationConfig,
    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
}

/// Types of visualizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizationType {
    /// Time series line chart
    TimeSeries,
    /// Bar chart
    BarChart,
    /// Pie chart
    PieChart,
    /// Heatmap
    Heatmap,
    /// Gauge/metric
    Gauge,
    /// Dashboard summary
    Dashboard,
    /// Scatter plot
    ScatterPlot,
    /// Area chart
    AreaChart,
}

/// Chart data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ChartData {
    /// Time series data
    TimeSeries(TimeSeriesData),
    /// Categorical data
    Categorical(CategoricalData),
    /// Heatmap data
    Heatmap(HeatmapData),
    /// Gauge data
    Gauge(GaugeData),
    /// Composite data (multiple series)
    Composite(CompositeData),
}

/// Time series data for line charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Data points
    pub points: Vec<TimeSeriesPoint>,
    /// X-axis label
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// Series name
    pub series_name: String,
    /// Unit (e.g., "ms", "%")
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Additional data (min, max, etc.)
    pub metadata: Option<HashMap<String, f64>>,
}

impl TimeSeriesData {
    /// Create new time series data
    pub fn new(series_name: impl Into<String>, unit: impl Into<String>) -> Self {
        let series_name_str = series_name.into();
        Self {
            points: Vec::new(),
            x_label: "Time".to_string(),
            y_label: series_name_str.clone(),
            series_name: series_name_str,
            unit: unit.into(),
        }
    }

    /// Add a data point
    pub fn add_point(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.points.push(TimeSeriesPoint { timestamp, value, metadata: None });
    }

    /// Add a point with metadata
    pub fn add_point_with_metadata(
        &mut self,
        timestamp: DateTime<Utc>,
        value: f64,
        metadata: HashMap<String, f64>,
    ) {
        self.points.push(TimeSeriesPoint { timestamp, value, metadata: Some(metadata) });
    }

    /// Sort points by timestamp
    pub fn sort_by_timestamp(&mut self) {
        self.points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }
}

/// Categorical data (bar charts, pie charts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalData {
    /// Categories
    pub categories: Vec<String>,
    /// Data series
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// Series name
    pub name: String,
    /// Values corresponding to categories
    pub values: Vec<f64>,
}

impl CategoricalData {
    /// Create new categorical data
    pub fn new() -> Self {
        Self { categories: Vec::new(), series: Vec::new() }
    }

    /// Add category
    pub fn add_category(&mut self, category: impl Into<String>) {
        self.categories.push(category.into());
    }

    /// Add series
    pub fn add_series(&mut self, name: impl Into<String>, values: Vec<f64>) {
        self.series.push(Series { name: name.into(), values });
    }
}

impl Default for CategoricalData {
    fn default() -> Self {
        Self::new()
    }
}

/// Heatmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    /// Grid cells
    pub cells: Vec<HeatmapCell>,
    /// X-axis labels
    pub x_labels: Vec<String>,
    /// Y-axis labels
    pub y_labels: Vec<String>,
    /// Color scheme (e.g., "viridis", "cool", "warm")
    pub color_scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// X position
    pub x: usize,
    /// Y position
    pub y: usize,
    /// Intensity (0-100)
    pub intensity: f64,
    /// Display value
    pub value: Option<String>,
}

impl HeatmapData {
    /// Create new heatmap
    pub fn new(x_labels: Vec<String>, y_labels: Vec<String>) -> Self {
        Self { cells: Vec::new(), x_labels, y_labels, color_scheme: "viridis".to_string() }
    }

    /// Add cell
    pub fn add_cell(&mut self, x: usize, y: usize, intensity: f64, value: Option<String>) {
        self.cells.push(HeatmapCell { x, y, intensity, value });
    }

    /// Set color scheme
    pub fn set_color_scheme(&mut self, scheme: impl Into<String>) {
        self.color_scheme = scheme.into();
    }
}

/// Gauge/metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeData {
    /// Current value
    pub value: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Thresholds for color coding
    pub thresholds: Vec<GaugeThreshold>,
    /// Unit
    pub unit: String,
    /// Metric name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeThreshold {
    /// Threshold value
    pub value: f64,
    /// Color (e.g., "green", "yellow", "red")
    pub color: String,
}

impl GaugeData {
    /// Create new gauge
    pub fn new(
        name: impl Into<String>,
        value: f64,
        min: f64,
        max: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            value,
            min,
            max,
            thresholds: vec![
                GaugeThreshold { value: min + (max - min) * 0.33, color: "green".to_string() },
                GaugeThreshold { value: min + (max - min) * 0.66, color: "yellow".to_string() },
                GaugeThreshold { value: max, color: "red".to_string() },
            ],
            unit: unit.into(),
            name: name.into(),
        }
    }

    /// Get current color based on value
    pub fn current_color(&self) -> &str {
        for threshold in self.thresholds.iter().rev() {
            if self.value <= threshold.value {
                return &threshold.color;
            }
        }
        &self.thresholds.first().map(|t| t.color.as_str()).unwrap_or(&"gray")
    }
}

/// Composite data (multiple related visualizations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeData {
    /// Sub-visualizations
    pub visualizations: Vec<VisualizationData>,
    /// Layout hint (e.g., "grid_2x2")
    pub layout: String,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Chart title
    pub title: String,
    /// Chart description
    pub description: Option<String>,
    /// Show legend
    pub show_legend: bool,
    /// Show grid
    pub show_grid: bool,
    /// Theme (e.g., "light", "dark")
    pub theme: String,
    /// Custom options
    pub options: HashMap<String, serde_json::Value>,
}

impl VisualizationConfig {
    /// Create new visualization config
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            show_legend: true,
            show_grid: true,
            theme: "light".to_string(),
            options: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set theme
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Add option
    pub fn with_option(mut self, key: String, value: serde_json::Value) -> Self {
        self.options.insert(key, value);
        self
    }
}

/// Custom metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricDefinition {
    /// Metric ID
    pub id: String,
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: String,
    /// Metric type
    pub metric_type: CustomMetricType,
    /// Data source
    pub data_source: DataSource,
    /// Aggregation method
    pub aggregation: AggregationMethod,
    /// Unit
    pub unit: String,
    /// Visualization type
    pub visualization_type: VisualizationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomMetricType {
    /// Derived from other metrics
    Derived,
    /// Raw metric from collector
    Raw,
    /// Business metric
    Business,
    /// Operational metric
    Operational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Source type (e.g., "process_metrics", "resource_metrics")
    pub source_type: String,
    /// Source identifier
    pub source_id: String,
    /// Field name to extract
    pub field: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// Sum all values
    Sum,
    /// Average
    Average,
    /// Min
    Min,
    /// Max
    Max,
    /// Count
    Count,
    /// Latest value
    Latest,
}

impl CustomMetricDefinition {
    /// Create new custom metric
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        metric_type: CustomMetricType,
    ) -> Self {
        let id = format!("metric-{}", uuid::Uuid::new_v4());
        Self {
            id,
            name: name.into(),
            description: description.into(),
            metric_type,
            data_source: DataSource {
                source_type: "process_metrics".to_string(),
                source_id: "".to_string(),
                field: "".to_string(),
            },
            aggregation: AggregationMethod::Average,
            unit: "".to_string(),
            visualization_type: VisualizationType::TimeSeries,
        }
    }

    /// Set data source
    pub fn with_data_source(
        mut self,
        source_type: impl Into<String>,
        source_id: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        self.data_source = DataSource {
            source_type: source_type.into(),
            source_id: source_id.into(),
            field: field.into(),
        };
        self
    }

    /// Set aggregation
    pub fn with_aggregation(mut self, agg: AggregationMethod) -> Self {
        self.aggregation = agg;
        self
    }

    /// Set unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Set visualization type
    pub fn with_visualization_type(mut self, viz_type: VisualizationType) -> Self {
        self.visualization_type = viz_type;
        self
    }
}

/// Metric visualization builder
#[derive(Debug, Clone)]
pub struct MetricVisualization;

impl MetricVisualization {
    /// Create time series visualization
    pub fn time_series(series_name: impl Into<String>, unit: impl Into<String>) -> TimeSeriesData {
        TimeSeriesData::new(series_name, unit)
    }

    /// Create categorical visualization
    pub fn categorical() -> CategoricalData {
        CategoricalData::new()
    }

    /// Create heatmap visualization
    pub fn heatmap(x_labels: Vec<String>, y_labels: Vec<String>) -> HeatmapData {
        HeatmapData::new(x_labels, y_labels)
    }

    /// Create gauge visualization
    pub fn gauge(
        name: impl Into<String>,
        value: f64,
        min: f64,
        max: f64,
        unit: impl Into<String>,
    ) -> GaugeData {
        GaugeData::new(name, value, min, max, unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_creation() {
        let mut ts = TimeSeriesData::new("response_time", "ms");
        ts.add_point(Utc::now(), 100.0);
        ts.add_point(Utc::now(), 150.0);

        assert_eq!(ts.points.len(), 2);
        assert_eq!(ts.unit, "ms");
    }

    #[test]
    fn test_heatmap_creation() {
        let mut heatmap = HeatmapData::new(
            vec!["0:00".to_string(), "1:00".to_string()],
            vec!["Mon".to_string(), "Tue".to_string()],
        );
        heatmap.add_cell(0, 0, 50.0, Some("10".to_string()));

        assert_eq!(heatmap.cells.len(), 1);
    }

    #[test]
    fn test_gauge_creation() {
        let gauge = GaugeData::new("CPU", 65.0, 0.0, 100.0, "%");
        assert_eq!(gauge.value, 65.0);
        assert!(!gauge.current_color().is_empty());
    }

    #[test]
    fn test_custom_metric_definition() {
        let metric = CustomMetricDefinition::new(
            "Avg Response Time",
            "Average response time for processes",
            CustomMetricType::Derived,
        )
        .with_data_source("process_metrics", "process_1", "avg_execution_time_ms")
        .with_unit("ms")
        .with_aggregation(AggregationMethod::Average);

        assert_eq!(metric.name, "Avg Response Time");
        assert_eq!(metric.unit, "ms");
    }
}
