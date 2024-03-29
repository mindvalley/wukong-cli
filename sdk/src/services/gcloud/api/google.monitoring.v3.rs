/// A single strongly-typed value.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypedValue {
    /// The typed value field.
    #[prost(oneof = "typed_value::Value", tags = "1, 2, 3, 4, 5")]
    pub value: ::core::option::Option<typed_value::Value>,
}
/// Nested message and enum types in `TypedValue`.
pub mod typed_value {
    /// The typed value field.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// A Boolean value: `true` or `false`.
        #[prost(bool, tag = "1")]
        BoolValue(bool),
        /// A 64-bit integer. Its range is approximately ±9.2x10<sup>18</sup>.
        #[prost(int64, tag = "2")]
        Int64Value(i64),
        /// A 64-bit double-precision floating-point number. Its magnitude
        /// is approximately ±10<sup>±300</sup> and it has 16
        /// significant digits of precision.
        #[prost(double, tag = "3")]
        DoubleValue(f64),
        /// A variable-length string value.
        #[prost(string, tag = "4")]
        StringValue(::prost::alloc::string::String),
        /// A distribution value.
        #[prost(message, tag = "5")]
        DistributionValue(super::super::super::api::Distribution),
    }
}
/// A closed time interval. It extends from the start time to the end time, and includes both: `[startTime, endTime]`. Valid time intervals depend on the \[`MetricKind`\](<https://cloud.google.com/monitoring/api/ref_v3/rest/v3/projects.metricDescriptors#MetricKind>) of the metric value. The end time must not be earlier than the start time. When writing data points, the start time must not be more than 25 hours in the past and the end time must not be more than five minutes in the future.
///
/// * For `GAUGE` metrics, the `startTime` value is technically optional; if
///   no value is specified, the start time defaults to the value of the
///   end time, and the interval represents a single point in time. If both
///   start and end times are specified, they must be identical. Such an
///   interval is valid only for `GAUGE` metrics, which are point-in-time
///   measurements. The end time of a new interval must be at least a
///   millisecond after the end time of the previous interval.
///
/// * For `DELTA` metrics, the start time and end time must specify a
///   non-zero interval, with subsequent points specifying contiguous and
///   non-overlapping intervals. For `DELTA` metrics, the start time of
///   the next interval must be at least a millisecond after the end time
///   of the previous interval.
///
/// * For `CUMULATIVE` metrics, the start time and end time must specify a
///   non-zero interval, with subsequent points specifying the same
///   start time and increasing end times, until an event resets the
///   cumulative value to zero and sets a new start time for the following
///   points. The new start time must be at least a millisecond after the
///   end time of the previous interval.
///
/// * The start time of a new interval must be at least a millisecond after the
///   end time of the previous interval because intervals are closed. If the
///   start time of a new interval is the same as the end time of the previous
///   interval, then data written at the new start time could overwrite data
///   written at the previous end time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeInterval {
    /// Required. The end of the time interval.
    #[prost(message, optional, tag = "2")]
    pub end_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Optional. The beginning of the time interval.  The default value
    /// for the start time is the end time. The start time must not be
    /// later than the end time.
    #[prost(message, optional, tag = "1")]
    pub start_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Describes how to combine multiple time series to provide a different view of
/// the data.  Aggregation of time series is done in two steps. First, each time
/// series in the set is *aligned* to the same time interval boundaries, then the
/// set of time series is optionally *reduced* in number.
///
/// Alignment consists of applying the `per_series_aligner` operation
/// to each time series after its data has been divided into regular
/// `alignment_period` time intervals. This process takes *all* of the data
/// points in an alignment period, applies a mathematical transformation such as
/// averaging, minimum, maximum, delta, etc., and converts them into a single
/// data point per period.
///
/// Reduction is when the aligned and transformed time series can optionally be
/// combined, reducing the number of time series through similar mathematical
/// transformations. Reduction involves applying a `cross_series_reducer` to
/// all the time series, optionally sorting the time series into subsets with
/// `group_by_fields`, and applying the reducer to each subset.
///
/// The raw time series data can contain a huge amount of information from
/// multiple sources. Alignment and reduction transforms this mass of data into
/// a more manageable and representative collection of data, for example "the
/// 95% latency across the average of all tasks in a cluster". This
/// representative data can be more easily graphed and comprehended, and the
/// individual time series data is still available for later drilldown. For more
/// details, see [Filtering and
/// aggregation](<https://cloud.google.com/monitoring/api/v3/aggregation>).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Aggregation {
    /// The `alignment_period` specifies a time interval, in seconds, that is used
    /// to divide the data in all the
    /// \[time series\]\\[google.monitoring.v3.TimeSeries\\] into consistent blocks of
    /// time. This will be done before the per-series aligner can be applied to
    /// the data.
    ///
    /// The value must be at least 60 seconds. If a per-series
    /// aligner other than `ALIGN_NONE` is specified, this field is required or an
    /// error is returned. If no per-series aligner is specified, or the aligner
    /// `ALIGN_NONE` is specified, then this field is ignored.
    ///
    /// The maximum value of the `alignment_period` is 104 weeks (2 years) for
    /// charts, and 90,000 seconds (25 hours) for alerting policies.
    #[prost(message, optional, tag = "1")]
    pub alignment_period: ::core::option::Option<::prost_types::Duration>,
    /// An `Aligner` describes how to bring the data points in a single
    /// time series into temporal alignment. Except for `ALIGN_NONE`, all
    /// alignments cause all the data points in an `alignment_period` to be
    /// mathematically grouped together, resulting in a single data point for
    /// each `alignment_period` with end timestamp at the end of the period.
    ///
    /// Not all alignment operations may be applied to all time series. The valid
    /// choices depend on the `metric_kind` and `value_type` of the original time
    /// series. Alignment can change the `metric_kind` or the `value_type` of
    /// the time series.
    ///
    /// Time series data must be aligned in order to perform cross-time
    /// series reduction. If `cross_series_reducer` is specified, then
    /// `per_series_aligner` must be specified and not equal to `ALIGN_NONE`
    /// and `alignment_period` must be specified; otherwise, an error is
    /// returned.
    #[prost(enumeration = "aggregation::Aligner", tag = "2")]
    pub per_series_aligner: i32,
    /// The reduction operation to be used to combine time series into a single
    /// time series, where the value of each data point in the resulting series is
    /// a function of all the already aligned values in the input time series.
    ///
    /// Not all reducer operations can be applied to all time series. The valid
    /// choices depend on the `metric_kind` and the `value_type` of the original
    /// time series. Reduction can yield a time series with a different
    /// `metric_kind` or `value_type` than the input time series.
    ///
    /// Time series data must first be aligned (see `per_series_aligner`) in order
    /// to perform cross-time series reduction. If `cross_series_reducer` is
    /// specified, then `per_series_aligner` must be specified, and must not be
    /// `ALIGN_NONE`. An `alignment_period` must also be specified; otherwise, an
    /// error is returned.
    #[prost(enumeration = "aggregation::Reducer", tag = "4")]
    pub cross_series_reducer: i32,
    /// The set of fields to preserve when `cross_series_reducer` is
    /// specified. The `group_by_fields` determine how the time series are
    /// partitioned into subsets prior to applying the aggregation
    /// operation. Each subset contains time series that have the same
    /// value for each of the grouping fields. Each individual time
    /// series is a member of exactly one subset. The
    /// `cross_series_reducer` is applied to each subset of time series.
    /// It is not possible to reduce across different resource types, so
    /// this field implicitly contains `resource.type`.  Fields not
    /// specified in `group_by_fields` are aggregated away.  If
    /// `group_by_fields` is not specified and all the time series have
    /// the same resource type, then the time series are aggregated into
    /// a single output time series. If `cross_series_reducer` is not
    /// defined, this field is ignored.
    #[prost(string, repeated, tag = "5")]
    pub group_by_fields: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `Aggregation`.
pub mod aggregation {
    /// The `Aligner` specifies the operation that will be applied to the data
    /// points in each alignment period in a time series. Except for
    /// `ALIGN_NONE`, which specifies that no operation be applied, each alignment
    /// operation replaces the set of data values in each alignment period with
    /// a single value: the result of applying the operation to the data values.
    /// An aligned time series has a single data value at the end of each
    /// `alignment_period`.
    ///
    /// An alignment operation can change the data type of the values, too. For
    /// example, if you apply a counting operation to boolean values, the data
    /// `value_type` in the original time series is `BOOLEAN`, but the `value_type`
    /// in the aligned result is `INT64`.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Aligner {
        /// No alignment. Raw data is returned. Not valid if cross-series reduction
        /// is requested. The `value_type` of the result is the same as the
        /// `value_type` of the input.
        AlignNone = 0,
        /// Align and convert to
        /// \\[DELTA\]\[google.api.MetricDescriptor.MetricKind.DELTA\\].
        /// The output is `delta = y1 - y0`.
        ///
        /// This alignment is valid for
        /// \\[CUMULATIVE\]\[google.api.MetricDescriptor.MetricKind.CUMULATIVE\\] and
        /// `DELTA` metrics. If the selected alignment period results in periods
        /// with no data, then the aligned value for such a period is created by
        /// interpolation. The `value_type`  of the aligned result is the same as
        /// the `value_type` of the input.
        AlignDelta = 1,
        /// Align and convert to a rate. The result is computed as
        /// `rate = (y1 - y0)/(t1 - t0)`, or "delta over time".
        /// Think of this aligner as providing the slope of the line that passes
        /// through the value at the start and at the end of the `alignment_period`.
        ///
        /// This aligner is valid for `CUMULATIVE`
        /// and `DELTA` metrics with numeric values. If the selected alignment
        /// period results in periods with no data, then the aligned value for
        /// such a period is created by interpolation. The output is a `GAUGE`
        /// metric with `value_type` `DOUBLE`.
        ///
        /// If, by "rate", you mean "percentage change", see the
        /// `ALIGN_PERCENT_CHANGE` aligner instead.
        AlignRate = 2,
        /// Align by interpolating between adjacent points around the alignment
        /// period boundary. This aligner is valid for `GAUGE` metrics with
        /// numeric values. The `value_type` of the aligned result is the same as the
        /// `value_type` of the input.
        AlignInterpolate = 3,
        /// Align by moving the most recent data point before the end of the
        /// alignment period to the boundary at the end of the alignment
        /// period. This aligner is valid for `GAUGE` metrics. The `value_type` of
        /// the aligned result is the same as the `value_type` of the input.
        AlignNextOlder = 4,
        /// Align the time series by returning the minimum value in each alignment
        /// period. This aligner is valid for `GAUGE` and `DELTA` metrics with
        /// numeric values. The `value_type` of the aligned result is the same as
        /// the `value_type` of the input.
        AlignMin = 10,
        /// Align the time series by returning the maximum value in each alignment
        /// period. This aligner is valid for `GAUGE` and `DELTA` metrics with
        /// numeric values. The `value_type` of the aligned result is the same as
        /// the `value_type` of the input.
        AlignMax = 11,
        /// Align the time series by returning the mean value in each alignment
        /// period. This aligner is valid for `GAUGE` and `DELTA` metrics with
        /// numeric values. The `value_type` of the aligned result is `DOUBLE`.
        AlignMean = 12,
        /// Align the time series by returning the number of values in each alignment
        /// period. This aligner is valid for `GAUGE` and `DELTA` metrics with
        /// numeric or Boolean values. The `value_type` of the aligned result is
        /// `INT64`.
        AlignCount = 13,
        /// Align the time series by returning the sum of the values in each
        /// alignment period. This aligner is valid for `GAUGE` and `DELTA`
        /// metrics with numeric and distribution values. The `value_type` of the
        /// aligned result is the same as the `value_type` of the input.
        AlignSum = 14,
        /// Align the time series by returning the standard deviation of the values
        /// in each alignment period. This aligner is valid for `GAUGE` and
        /// `DELTA` metrics with numeric values. The `value_type` of the output is
        /// `DOUBLE`.
        AlignStddev = 15,
        /// Align the time series by returning the number of `True` values in
        /// each alignment period. This aligner is valid for `GAUGE` metrics with
        /// Boolean values. The `value_type` of the output is `INT64`.
        AlignCountTrue = 16,
        /// Align the time series by returning the number of `False` values in
        /// each alignment period. This aligner is valid for `GAUGE` metrics with
        /// Boolean values. The `value_type` of the output is `INT64`.
        AlignCountFalse = 24,
        /// Align the time series by returning the ratio of the number of `True`
        /// values to the total number of values in each alignment period. This
        /// aligner is valid for `GAUGE` metrics with Boolean values. The output
        /// value is in the range \[0.0, 1.0\] and has `value_type` `DOUBLE`.
        AlignFractionTrue = 17,
        /// Align the time series by using [percentile
        /// aggregation](<https://en.wikipedia.org/wiki/Percentile>). The resulting
        /// data point in each alignment period is the 99th percentile of all data
        /// points in the period. This aligner is valid for `GAUGE` and `DELTA`
        /// metrics with distribution values. The output is a `GAUGE` metric with
        /// `value_type` `DOUBLE`.
        AlignPercentile99 = 18,
        /// Align the time series by using [percentile
        /// aggregation](<https://en.wikipedia.org/wiki/Percentile>). The resulting
        /// data point in each alignment period is the 95th percentile of all data
        /// points in the period. This aligner is valid for `GAUGE` and `DELTA`
        /// metrics with distribution values. The output is a `GAUGE` metric with
        /// `value_type` `DOUBLE`.
        AlignPercentile95 = 19,
        /// Align the time series by using [percentile
        /// aggregation](<https://en.wikipedia.org/wiki/Percentile>). The resulting
        /// data point in each alignment period is the 50th percentile of all data
        /// points in the period. This aligner is valid for `GAUGE` and `DELTA`
        /// metrics with distribution values. The output is a `GAUGE` metric with
        /// `value_type` `DOUBLE`.
        AlignPercentile50 = 20,
        /// Align the time series by using [percentile
        /// aggregation](<https://en.wikipedia.org/wiki/Percentile>). The resulting
        /// data point in each alignment period is the 5th percentile of all data
        /// points in the period. This aligner is valid for `GAUGE` and `DELTA`
        /// metrics with distribution values. The output is a `GAUGE` metric with
        /// `value_type` `DOUBLE`.
        AlignPercentile05 = 21,
        /// Align and convert to a percentage change. This aligner is valid for
        /// `GAUGE` and `DELTA` metrics with numeric values. This alignment returns
        /// `((current - previous)/previous) * 100`, where the value of `previous` is
        /// determined based on the `alignment_period`.
        ///
        /// If the values of `current` and `previous` are both 0, then the returned
        /// value is 0. If only `previous` is 0, the returned value is infinity.
        ///
        /// A 10-minute moving mean is computed at each point of the alignment period
        /// prior to the above calculation to smooth the metric and prevent false
        /// positives from very short-lived spikes. The moving mean is only
        /// applicable for data whose values are `>= 0`. Any values `< 0` are
        /// treated as a missing datapoint, and are ignored. While `DELTA`
        /// metrics are accepted by this alignment, special care should be taken that
        /// the values for the metric will always be positive. The output is a
        /// `GAUGE` metric with `value_type` `DOUBLE`.
        AlignPercentChange = 23,
    }
    impl Aligner {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Aligner::AlignNone => "ALIGN_NONE",
                Aligner::AlignDelta => "ALIGN_DELTA",
                Aligner::AlignRate => "ALIGN_RATE",
                Aligner::AlignInterpolate => "ALIGN_INTERPOLATE",
                Aligner::AlignNextOlder => "ALIGN_NEXT_OLDER",
                Aligner::AlignMin => "ALIGN_MIN",
                Aligner::AlignMax => "ALIGN_MAX",
                Aligner::AlignMean => "ALIGN_MEAN",
                Aligner::AlignCount => "ALIGN_COUNT",
                Aligner::AlignSum => "ALIGN_SUM",
                Aligner::AlignStddev => "ALIGN_STDDEV",
                Aligner::AlignCountTrue => "ALIGN_COUNT_TRUE",
                Aligner::AlignCountFalse => "ALIGN_COUNT_FALSE",
                Aligner::AlignFractionTrue => "ALIGN_FRACTION_TRUE",
                Aligner::AlignPercentile99 => "ALIGN_PERCENTILE_99",
                Aligner::AlignPercentile95 => "ALIGN_PERCENTILE_95",
                Aligner::AlignPercentile50 => "ALIGN_PERCENTILE_50",
                Aligner::AlignPercentile05 => "ALIGN_PERCENTILE_05",
                Aligner::AlignPercentChange => "ALIGN_PERCENT_CHANGE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ALIGN_NONE" => Some(Self::AlignNone),
                "ALIGN_DELTA" => Some(Self::AlignDelta),
                "ALIGN_RATE" => Some(Self::AlignRate),
                "ALIGN_INTERPOLATE" => Some(Self::AlignInterpolate),
                "ALIGN_NEXT_OLDER" => Some(Self::AlignNextOlder),
                "ALIGN_MIN" => Some(Self::AlignMin),
                "ALIGN_MAX" => Some(Self::AlignMax),
                "ALIGN_MEAN" => Some(Self::AlignMean),
                "ALIGN_COUNT" => Some(Self::AlignCount),
                "ALIGN_SUM" => Some(Self::AlignSum),
                "ALIGN_STDDEV" => Some(Self::AlignStddev),
                "ALIGN_COUNT_TRUE" => Some(Self::AlignCountTrue),
                "ALIGN_COUNT_FALSE" => Some(Self::AlignCountFalse),
                "ALIGN_FRACTION_TRUE" => Some(Self::AlignFractionTrue),
                "ALIGN_PERCENTILE_99" => Some(Self::AlignPercentile99),
                "ALIGN_PERCENTILE_95" => Some(Self::AlignPercentile95),
                "ALIGN_PERCENTILE_50" => Some(Self::AlignPercentile50),
                "ALIGN_PERCENTILE_05" => Some(Self::AlignPercentile05),
                "ALIGN_PERCENT_CHANGE" => Some(Self::AlignPercentChange),
                _ => None,
            }
        }
    }
    /// A Reducer operation describes how to aggregate data points from multiple
    /// time series into a single time series, where the value of each data point
    /// in the resulting series is a function of all the already aligned values in
    /// the input time series.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Reducer {
        /// No cross-time series reduction. The output of the `Aligner` is
        /// returned.
        ReduceNone = 0,
        /// Reduce by computing the mean value across time series for each
        /// alignment period. This reducer is valid for
        /// \\[DELTA\]\[google.api.MetricDescriptor.MetricKind.DELTA\\] and
        /// \\[GAUGE\]\[google.api.MetricDescriptor.MetricKind.GAUGE\\] metrics with
        /// numeric or distribution values. The `value_type` of the output is
        /// \\[DOUBLE\]\[google.api.MetricDescriptor.ValueType.DOUBLE\\].
        ReduceMean = 1,
        /// Reduce by computing the minimum value across time series for each
        /// alignment period. This reducer is valid for `DELTA` and `GAUGE` metrics
        /// with numeric values. The `value_type` of the output is the same as the
        /// `value_type` of the input.
        ReduceMin = 2,
        /// Reduce by computing the maximum value across time series for each
        /// alignment period. This reducer is valid for `DELTA` and `GAUGE` metrics
        /// with numeric values. The `value_type` of the output is the same as the
        /// `value_type` of the input.
        ReduceMax = 3,
        /// Reduce by computing the sum across time series for each
        /// alignment period. This reducer is valid for `DELTA` and `GAUGE` metrics
        /// with numeric and distribution values. The `value_type` of the output is
        /// the same as the `value_type` of the input.
        ReduceSum = 4,
        /// Reduce by computing the standard deviation across time series
        /// for each alignment period. This reducer is valid for `DELTA` and
        /// `GAUGE` metrics with numeric or distribution values. The `value_type`
        /// of the output is `DOUBLE`.
        ReduceStddev = 5,
        /// Reduce by computing the number of data points across time series
        /// for each alignment period. This reducer is valid for `DELTA` and
        /// `GAUGE` metrics of numeric, Boolean, distribution, and string
        /// `value_type`. The `value_type` of the output is `INT64`.
        ReduceCount = 6,
        /// Reduce by computing the number of `True`-valued data points across time
        /// series for each alignment period. This reducer is valid for `DELTA` and
        /// `GAUGE` metrics of Boolean `value_type`. The `value_type` of the output
        /// is `INT64`.
        ReduceCountTrue = 7,
        /// Reduce by computing the number of `False`-valued data points across time
        /// series for each alignment period. This reducer is valid for `DELTA` and
        /// `GAUGE` metrics of Boolean `value_type`. The `value_type` of the output
        /// is `INT64`.
        ReduceCountFalse = 15,
        /// Reduce by computing the ratio of the number of `True`-valued data points
        /// to the total number of data points for each alignment period. This
        /// reducer is valid for `DELTA` and `GAUGE` metrics of Boolean `value_type`.
        /// The output value is in the range \[0.0, 1.0\] and has `value_type`
        /// `DOUBLE`.
        ReduceFractionTrue = 8,
        /// Reduce by computing the [99th
        /// percentile](<https://en.wikipedia.org/wiki/Percentile>) of data points
        /// across time series for each alignment period. This reducer is valid for
        /// `GAUGE` and `DELTA` metrics of numeric and distribution type. The value
        /// of the output is `DOUBLE`.
        ReducePercentile99 = 9,
        /// Reduce by computing the [95th
        /// percentile](<https://en.wikipedia.org/wiki/Percentile>) of data points
        /// across time series for each alignment period. This reducer is valid for
        /// `GAUGE` and `DELTA` metrics of numeric and distribution type. The value
        /// of the output is `DOUBLE`.
        ReducePercentile95 = 10,
        /// Reduce by computing the [50th
        /// percentile](<https://en.wikipedia.org/wiki/Percentile>) of data points
        /// across time series for each alignment period. This reducer is valid for
        /// `GAUGE` and `DELTA` metrics of numeric and distribution type. The value
        /// of the output is `DOUBLE`.
        ReducePercentile50 = 11,
        /// Reduce by computing the [5th
        /// percentile](<https://en.wikipedia.org/wiki/Percentile>) of data points
        /// across time series for each alignment period. This reducer is valid for
        /// `GAUGE` and `DELTA` metrics of numeric and distribution type. The value
        /// of the output is `DOUBLE`.
        ReducePercentile05 = 12,
    }
    impl Reducer {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Reducer::ReduceNone => "REDUCE_NONE",
                Reducer::ReduceMean => "REDUCE_MEAN",
                Reducer::ReduceMin => "REDUCE_MIN",
                Reducer::ReduceMax => "REDUCE_MAX",
                Reducer::ReduceSum => "REDUCE_SUM",
                Reducer::ReduceStddev => "REDUCE_STDDEV",
                Reducer::ReduceCount => "REDUCE_COUNT",
                Reducer::ReduceCountTrue => "REDUCE_COUNT_TRUE",
                Reducer::ReduceCountFalse => "REDUCE_COUNT_FALSE",
                Reducer::ReduceFractionTrue => "REDUCE_FRACTION_TRUE",
                Reducer::ReducePercentile99 => "REDUCE_PERCENTILE_99",
                Reducer::ReducePercentile95 => "REDUCE_PERCENTILE_95",
                Reducer::ReducePercentile50 => "REDUCE_PERCENTILE_50",
                Reducer::ReducePercentile05 => "REDUCE_PERCENTILE_05",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "REDUCE_NONE" => Some(Self::ReduceNone),
                "REDUCE_MEAN" => Some(Self::ReduceMean),
                "REDUCE_MIN" => Some(Self::ReduceMin),
                "REDUCE_MAX" => Some(Self::ReduceMax),
                "REDUCE_SUM" => Some(Self::ReduceSum),
                "REDUCE_STDDEV" => Some(Self::ReduceStddev),
                "REDUCE_COUNT" => Some(Self::ReduceCount),
                "REDUCE_COUNT_TRUE" => Some(Self::ReduceCountTrue),
                "REDUCE_COUNT_FALSE" => Some(Self::ReduceCountFalse),
                "REDUCE_FRACTION_TRUE" => Some(Self::ReduceFractionTrue),
                "REDUCE_PERCENTILE_99" => Some(Self::ReducePercentile99),
                "REDUCE_PERCENTILE_95" => Some(Self::ReducePercentile95),
                "REDUCE_PERCENTILE_50" => Some(Self::ReducePercentile50),
                "REDUCE_PERCENTILE_05" => Some(Self::ReducePercentile05),
                _ => None,
            }
        }
    }
}
/// Specifies an ordering relationship on two arguments, called `left` and
/// `right`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ComparisonType {
    /// No ordering relationship is specified.
    ComparisonUnspecified = 0,
    /// True if the left argument is greater than the right argument.
    ComparisonGt = 1,
    /// True if the left argument is greater than or equal to the right argument.
    ComparisonGe = 2,
    /// True if the left argument is less than the right argument.
    ComparisonLt = 3,
    /// True if the left argument is less than or equal to the right argument.
    ComparisonLe = 4,
    /// True if the left argument is equal to the right argument.
    ComparisonEq = 5,
    /// True if the left argument is not equal to the right argument.
    ComparisonNe = 6,
}
impl ComparisonType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ComparisonType::ComparisonUnspecified => "COMPARISON_UNSPECIFIED",
            ComparisonType::ComparisonGt => "COMPARISON_GT",
            ComparisonType::ComparisonGe => "COMPARISON_GE",
            ComparisonType::ComparisonLt => "COMPARISON_LT",
            ComparisonType::ComparisonLe => "COMPARISON_LE",
            ComparisonType::ComparisonEq => "COMPARISON_EQ",
            ComparisonType::ComparisonNe => "COMPARISON_NE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "COMPARISON_UNSPECIFIED" => Some(Self::ComparisonUnspecified),
            "COMPARISON_GT" => Some(Self::ComparisonGt),
            "COMPARISON_GE" => Some(Self::ComparisonGe),
            "COMPARISON_LT" => Some(Self::ComparisonLt),
            "COMPARISON_LE" => Some(Self::ComparisonLe),
            "COMPARISON_EQ" => Some(Self::ComparisonEq),
            "COMPARISON_NE" => Some(Self::ComparisonNe),
            _ => None,
        }
    }
}
/// The tier of service for a Workspace. Please see the
/// [service tiers
/// documentation](<https://cloud.google.com/monitoring/workspaces/tiers>) for more
/// details.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ServiceTier {
    /// An invalid sentinel value, used to indicate that a tier has not
    /// been provided explicitly.
    Unspecified = 0,
    /// The Stackdriver Basic tier, a free tier of service that provides basic
    /// features, a moderate allotment of logs, and access to built-in metrics.
    /// A number of features are not available in this tier. For more details,
    /// see [the service tiers
    /// documentation](<https://cloud.google.com/monitoring/workspaces/tiers>).
    Basic = 1,
    /// The Stackdriver Premium tier, a higher, more expensive tier of service
    /// that provides access to all Stackdriver features, lets you use Stackdriver
    /// with AWS accounts, and has a larger allotments for logs and metrics. For
    /// more details, see [the service tiers
    /// documentation](<https://cloud.google.com/monitoring/workspaces/tiers>).
    Premium = 2,
}
impl ServiceTier {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ServiceTier::Unspecified => "SERVICE_TIER_UNSPECIFIED",
            ServiceTier::Basic => "SERVICE_TIER_BASIC",
            ServiceTier::Premium => "SERVICE_TIER_PREMIUM",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SERVICE_TIER_UNSPECIFIED" => Some(Self::Unspecified),
            "SERVICE_TIER_BASIC" => Some(Self::Basic),
            "SERVICE_TIER_PREMIUM" => Some(Self::Premium),
            _ => None,
        }
    }
}
/// A single data point in a time series.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Point {
    /// The time interval to which the data point applies.  For `GAUGE` metrics,
    /// the start time is optional, but if it is supplied, it must equal the
    /// end time.  For `DELTA` metrics, the start
    /// and end time should specify a non-zero interval, with subsequent points
    /// specifying contiguous and non-overlapping intervals.  For `CUMULATIVE`
    /// metrics, the start and end time should specify a non-zero interval, with
    /// subsequent points specifying the same start time and increasing end times,
    /// until an event resets the cumulative value to zero and sets a new start
    /// time for the following points.
    #[prost(message, optional, tag = "1")]
    pub interval: ::core::option::Option<TimeInterval>,
    /// The value of the data point.
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<TypedValue>,
}
/// A collection of data points that describes the time-varying values
/// of a metric. A time series is identified by a combination of a
/// fully-specified monitored resource and a fully-specified metric.
/// This type is used for both listing and creating time series.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeSeries {
    /// The associated metric. A fully-specified metric used to identify the time
    /// series.
    #[prost(message, optional, tag = "1")]
    pub metric: ::core::option::Option<super::super::api::Metric>,
    /// The associated monitored resource.  Custom metrics can use only certain
    /// monitored resource types in their time series data. For more information,
    /// see [Monitored resources for custom
    /// metrics](<https://cloud.google.com/monitoring/custom-metrics/creating-metrics#custom-metric-resources>).
    #[prost(message, optional, tag = "2")]
    pub resource: ::core::option::Option<super::super::api::MonitoredResource>,
    /// Output only. The associated monitored resource metadata. When reading a
    /// time series, this field will include metadata labels that are explicitly
    /// named in the reduction. When creating a time series, this field is ignored.
    #[prost(message, optional, tag = "7")]
    pub metadata: ::core::option::Option<super::super::api::MonitoredResourceMetadata>,
    /// The metric kind of the time series. When listing time series, this metric
    /// kind might be different from the metric kind of the associated metric if
    /// this time series is an alignment or reduction of other time series.
    ///
    /// When creating a time series, this field is optional. If present, it must be
    /// the same as the metric kind of the associated metric. If the associated
    /// metric's descriptor must be auto-created, then this field specifies the
    /// metric kind of the new descriptor and must be either `GAUGE` (the default)
    /// or `CUMULATIVE`.
    #[prost(enumeration = "super::super::api::metric_descriptor::MetricKind", tag = "3")]
    pub metric_kind: i32,
    /// The value type of the time series. When listing time series, this value
    /// type might be different from the value type of the associated metric if
    /// this time series is an alignment or reduction of other time series.
    ///
    /// When creating a time series, this field is optional. If present, it must be
    /// the same as the type of the data in the `points` field.
    #[prost(enumeration = "super::super::api::metric_descriptor::ValueType", tag = "4")]
    pub value_type: i32,
    /// The data points of this time series. When listing time series, points are
    /// returned in reverse time order.
    ///
    /// When creating a time series, this field must contain exactly one point and
    /// the point's type must be the same as the value type of the associated
    /// metric. If the associated metric's descriptor must be auto-created, then
    /// the value type of the descriptor is determined by the point's type, which
    /// must be `BOOL`, `INT64`, `DOUBLE`, or `DISTRIBUTION`.
    #[prost(message, repeated, tag = "5")]
    pub points: ::prost::alloc::vec::Vec<Point>,
    /// The units in which the metric value is reported. It is only applicable
    /// if the `value_type` is `INT64`, `DOUBLE`, or `DISTRIBUTION`. The `unit`
    /// defines the representation of the stored metric values.
    #[prost(string, tag = "8")]
    pub unit: ::prost::alloc::string::String,
}
/// A descriptor for the labels and points in a time series.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeSeriesDescriptor {
    /// Descriptors for the labels.
    #[prost(message, repeated, tag = "1")]
    pub label_descriptors: ::prost::alloc::vec::Vec<super::super::api::LabelDescriptor>,
    /// Descriptors for the point data value columns.
    #[prost(message, repeated, tag = "5")]
    pub point_descriptors: ::prost::alloc::vec::Vec<
        time_series_descriptor::ValueDescriptor,
    >,
}
/// Nested message and enum types in `TimeSeriesDescriptor`.
pub mod time_series_descriptor {
    /// A descriptor for the value columns in a data point.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ValueDescriptor {
        /// The value key.
        #[prost(string, tag = "1")]
        pub key: ::prost::alloc::string::String,
        /// The value type.
        #[prost(
            enumeration = "super::super::super::api::metric_descriptor::ValueType",
            tag = "2"
        )]
        pub value_type: i32,
        /// The value stream kind.
        #[prost(
            enumeration = "super::super::super::api::metric_descriptor::MetricKind",
            tag = "3"
        )]
        pub metric_kind: i32,
        /// The unit in which `time_series` point values are reported. `unit`
        /// follows the UCUM format for units as seen in
        /// <https://unitsofmeasure.org/ucum.html.>
        /// `unit` is only valid if `value_type` is INTEGER, DOUBLE, DISTRIBUTION.
        #[prost(string, tag = "4")]
        pub unit: ::prost::alloc::string::String,
    }
}
/// Represents the values of a time series associated with a
/// TimeSeriesDescriptor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeSeriesData {
    /// The values of the labels in the time series identifier, given in the same
    /// order as the `label_descriptors` field of the TimeSeriesDescriptor
    /// associated with this object. Each value must have a value of the type
    /// given in the corresponding entry of `label_descriptors`.
    #[prost(message, repeated, tag = "1")]
    pub label_values: ::prost::alloc::vec::Vec<LabelValue>,
    /// The points in the time series.
    #[prost(message, repeated, tag = "2")]
    pub point_data: ::prost::alloc::vec::Vec<time_series_data::PointData>,
}
/// Nested message and enum types in `TimeSeriesData`.
pub mod time_series_data {
    /// A point's value columns and time interval. Each point has one or more
    /// point values corresponding to the entries in `point_descriptors` field in
    /// the TimeSeriesDescriptor associated with this object.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PointData {
        /// The values that make up the point.
        #[prost(message, repeated, tag = "1")]
        pub values: ::prost::alloc::vec::Vec<super::TypedValue>,
        /// The time interval associated with the point.
        #[prost(message, optional, tag = "2")]
        pub time_interval: ::core::option::Option<super::TimeInterval>,
    }
}
/// A label value.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelValue {
    /// The label value can be a bool, int64, or string.
    #[prost(oneof = "label_value::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<label_value::Value>,
}
/// Nested message and enum types in `LabelValue`.
pub mod label_value {
    /// The label value can be a bool, int64, or string.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// A bool label value.
        #[prost(bool, tag = "1")]
        BoolValue(bool),
        /// An int64 label value.
        #[prost(int64, tag = "2")]
        Int64Value(i64),
        /// A string label value.
        #[prost(string, tag = "3")]
        StringValue(::prost::alloc::string::String),
    }
}
/// An error associated with a query in the time series query language format.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryError {
    /// The location of the time series query language text that this error applies
    /// to.
    #[prost(message, optional, tag = "1")]
    pub locator: ::core::option::Option<TextLocator>,
    /// The error message.
    #[prost(string, tag = "2")]
    pub message: ::prost::alloc::string::String,
}
/// A locator for text. Indicates a particular part of the text of a request or
/// of an object referenced in the request.
///
/// For example, suppose the request field `text` contains:
///
/// text: "The quick brown fox jumps over the lazy dog."
///
/// Then the locator:
///
/// source: "text"
/// start_position {
/// line: 1
/// column: 17
/// }
/// end_position {
/// line: 1
/// column: 19
/// }
///
/// refers to the part of the text: "fox".
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TextLocator {
    /// The source of the text. The source may be a field in the request, in which
    /// case its format is the format of the
    /// google.rpc.BadRequest.FieldViolation.field field in
    /// <https://cloud.google.com/apis/design/errors#error_details.> It may also be
    /// be a source other than the request field (e.g. a macro definition
    /// referenced in the text of the query), in which case this is the name of
    /// the source (e.g. the macro name).
    #[prost(string, tag = "1")]
    pub source: ::prost::alloc::string::String,
    /// The position of the first byte within the text.
    #[prost(message, optional, tag = "2")]
    pub start_position: ::core::option::Option<text_locator::Position>,
    /// The position of the last byte within the text.
    #[prost(message, optional, tag = "3")]
    pub end_position: ::core::option::Option<text_locator::Position>,
    /// If `source`, `start_position`, and `end_position` describe a call on
    /// some object (e.g. a macro in the time series query language text) and a
    /// location is to be designated in that object's text, `nested_locator`
    /// identifies the location within that object.
    #[prost(message, optional, boxed, tag = "4")]
    pub nested_locator: ::core::option::Option<::prost::alloc::boxed::Box<TextLocator>>,
    /// When `nested_locator` is set, this field gives the reason for the nesting.
    /// Usually, the reason is a macro invocation. In that case, the macro name
    /// (including the leading '@') signals the location of the macro call
    /// in the text and a macro argument name (including the leading '$') signals
    /// the location of the macro argument inside the macro body that got
    /// substituted away.
    #[prost(string, tag = "5")]
    pub nesting_reason: ::prost::alloc::string::String,
}
/// Nested message and enum types in `TextLocator`.
pub mod text_locator {
    /// The position of a byte within the text.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Position {
        /// The line, starting with 1, where the byte is positioned.
        #[prost(int32, tag = "1")]
        pub line: i32,
        /// The column within the line, starting with 1, where the byte is
        /// positioned. This is a byte index even though the text is UTF-8.
        #[prost(int32, tag = "2")]
        pub column: i32,
    }
}
/// The `ListMonitoredResourceDescriptors` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMonitoredResourceDescriptorsRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>) on
    /// which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER\]
    /// ```
    #[prost(string, tag = "5")]
    pub name: ::prost::alloc::string::String,
    /// An optional \[filter\](<https://cloud.google.com/monitoring/api/v3/filters>)
    /// describing the descriptors to be returned.  The filter can reference the
    /// descriptor's type and labels. For example, the following filter returns
    /// only Google Compute Engine descriptors that have an `id` label:
    ///
    /// ```text
    /// resource.type = starts_with("gce_") AND resource.label:id
    /// ```
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// A positive number that is the maximum number of results to return.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If this field is not empty then it must contain the `nextPageToken` value
    /// returned by a previous call to this method.  Using this field causes the
    /// method to return additional results from the previous method call.
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The `ListMonitoredResourceDescriptors` response.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMonitoredResourceDescriptorsResponse {
    /// The monitored resource descriptors that are available to this project
    /// and that match `filter`, if present.
    #[prost(message, repeated, tag = "1")]
    pub resource_descriptors: ::prost::alloc::vec::Vec<
        super::super::api::MonitoredResourceDescriptor,
    >,
    /// If there are more results than have been returned, then this field is set
    /// to a non-empty value.  To see the additional results,
    /// use that value as `page_token` in the next call to this method.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The `GetMonitoredResourceDescriptor` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMonitoredResourceDescriptorRequest {
    /// Required. The monitored resource descriptor to get.  The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER]/monitoredResourceDescriptors/[RESOURCE_TYPE\]
    /// ```
    ///
    /// The `\[RESOURCE_TYPE\]` is a predefined type, such as
    /// `cloudsql_database`.
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
/// The `ListMetricDescriptors` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMetricDescriptorsRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>) on
    /// which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER\]
    /// ```
    #[prost(string, tag = "5")]
    pub name: ::prost::alloc::string::String,
    /// If this field is empty, all custom and
    /// system-defined metric descriptors are returned.
    /// Otherwise, the \[filter\](<https://cloud.google.com/monitoring/api/v3/filters>)
    /// specifies which metric descriptors are to be
    /// returned. For example, the following filter matches all
    /// [custom metrics](<https://cloud.google.com/monitoring/custom-metrics>):
    ///
    /// ```text
    /// metric.type = starts_with("custom.googleapis.com/")
    /// ```
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// A positive number that is the maximum number of results to return.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If this field is not empty then it must contain the `nextPageToken` value
    /// returned by a previous call to this method.  Using this field causes the
    /// method to return additional results from the previous method call.
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The `ListMetricDescriptors` response.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMetricDescriptorsResponse {
    /// The metric descriptors that are available to the project
    /// and that match the value of `filter`, if present.
    #[prost(message, repeated, tag = "1")]
    pub metric_descriptors: ::prost::alloc::vec::Vec<
        super::super::api::MetricDescriptor,
    >,
    /// If there are more results than have been returned, then this field is set
    /// to a non-empty value.  To see the additional results,
    /// use that value as `page_token` in the next call to this method.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The `GetMetricDescriptor` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMetricDescriptorRequest {
    /// Required. The metric descriptor on which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER]/metricDescriptors/[METRIC_ID\]
    /// ```
    ///
    /// An example value of `\[METRIC_ID\]` is
    /// `"compute.googleapis.com/instance/disk/read_bytes_count"`.
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
/// The `CreateMetricDescriptor` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateMetricDescriptorRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>) on
    /// which to execute the request. The format is:
    /// 4
    /// projects/\\[PROJECT_ID_OR_NUMBER\\]
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    /// Required. The new [custom metric](<https://cloud.google.com/monitoring/custom-metrics>)
    /// descriptor.
    #[prost(message, optional, tag = "2")]
    pub metric_descriptor: ::core::option::Option<super::super::api::MetricDescriptor>,
}
/// The `DeleteMetricDescriptor` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteMetricDescriptorRequest {
    /// Required. The metric descriptor on which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER]/metricDescriptors/[METRIC_ID\]
    /// ```
    ///
    /// An example of `\[METRIC_ID\]` is:
    /// `"custom.googleapis.com/my_test_metric"`.
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
/// The `ListTimeSeries` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTimeSeriesRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>),
    /// organization or folder on which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER\]
    /// organizations/\[ORGANIZATION_ID\]
    /// folders/\[FOLDER_ID\]
    /// ```
    #[prost(string, tag = "10")]
    pub name: ::prost::alloc::string::String,
    /// Required. A [monitoring filter](<https://cloud.google.com/monitoring/api/v3/filters>)
    /// that specifies which time series should be returned.  The filter must
    /// specify a single metric type, and can additionally specify metric labels
    /// and other information. For example:
    ///
    /// ```text
    /// metric.type = "compute.googleapis.com/instance/cpu/usage_time" AND
    ///      metric.labels.instance_name = "my-instance-name"
    /// ```
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Required. The time interval for which results should be returned. Only time series
    /// that contain data points in the specified interval are included
    /// in the response.
    #[prost(message, optional, tag = "4")]
    pub interval: ::core::option::Option<TimeInterval>,
    /// Specifies the alignment of data points in individual time series as
    /// well as how to combine the retrieved time series across specified labels.
    ///
    /// By default (if no `aggregation` is explicitly specified), the raw time
    /// series data is returned.
    #[prost(message, optional, tag = "5")]
    pub aggregation: ::core::option::Option<Aggregation>,
    /// Apply a second aggregation after `aggregation` is applied. May only be
    /// specified if `aggregation` is specified.
    #[prost(message, optional, tag = "11")]
    pub secondary_aggregation: ::core::option::Option<Aggregation>,
    /// Unsupported: must be left blank. The points in each time series are
    /// currently returned in reverse time order (most recent to oldest).
    #[prost(string, tag = "6")]
    pub order_by: ::prost::alloc::string::String,
    /// Required. Specifies which information is returned about the time series.
    #[prost(enumeration = "list_time_series_request::TimeSeriesView", tag = "7")]
    pub view: i32,
    /// A positive number that is the maximum number of results to return. If
    /// `page_size` is empty or more than 100,000 results, the effective
    /// `page_size` is 100,000 results. If `view` is set to `FULL`, this is the
    /// maximum number of `Points` returned. If `view` is set to `HEADERS`, this is
    /// the maximum number of `TimeSeries` returned.
    #[prost(int32, tag = "8")]
    pub page_size: i32,
    /// If this field is not empty then it must contain the `nextPageToken` value
    /// returned by a previous call to this method.  Using this field causes the
    /// method to return additional results from the previous method call.
    #[prost(string, tag = "9")]
    pub page_token: ::prost::alloc::string::String,
}
/// Nested message and enum types in `ListTimeSeriesRequest`.
pub mod list_time_series_request {
    /// Controls which fields are returned by `ListTimeSeries`.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum TimeSeriesView {
        /// Returns the identity of the metric(s), the time series,
        /// and the time series data.
        Full = 0,
        /// Returns the identity of the metric and the time series resource,
        /// but not the time series data.
        Headers = 1,
    }
    impl TimeSeriesView {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                TimeSeriesView::Full => "FULL",
                TimeSeriesView::Headers => "HEADERS",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "FULL" => Some(Self::Full),
                "HEADERS" => Some(Self::Headers),
                _ => None,
            }
        }
    }
}
/// The `ListTimeSeries` response.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTimeSeriesResponse {
    /// One or more time series that match the filter included in the request.
    #[prost(message, repeated, tag = "1")]
    pub time_series: ::prost::alloc::vec::Vec<TimeSeries>,
    /// If there are more results than have been returned, then this field is set
    /// to a non-empty value.  To see the additional results,
    /// use that value as `page_token` in the next call to this method.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
    /// Query execution errors that may have caused the time series data returned
    /// to be incomplete.
    #[prost(message, repeated, tag = "3")]
    pub execution_errors: ::prost::alloc::vec::Vec<super::super::rpc::Status>,
    /// The unit in which all `time_series` point values are reported. `unit`
    /// follows the UCUM format for units as seen in
    /// <https://unitsofmeasure.org/ucum.html.>
    /// If different `time_series` have different units (for example, because they
    /// come from different metric types, or a unit is absent), then `unit` will be
    /// "{not_a_unit}".
    #[prost(string, tag = "5")]
    pub unit: ::prost::alloc::string::String,
}
/// The `CreateTimeSeries` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTimeSeriesRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>) on
    /// which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER\]
    /// ```
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    /// Required. The new data to be added to a list of time series.
    /// Adds at most one data point to each of several time series.  The new data
    /// point must be more recent than any other point in its time series.  Each
    /// `TimeSeries` value must fully specify a unique time series by supplying
    /// all label values for the metric and the monitored resource.
    ///
    /// The maximum number of `TimeSeries` objects per `Create` request is 200.
    #[prost(message, repeated, tag = "2")]
    pub time_series: ::prost::alloc::vec::Vec<TimeSeries>,
}
/// DEPRECATED. Used to hold per-time-series error status.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTimeSeriesError {
    /// DEPRECATED. Time series ID that resulted in the `status` error.
    #[deprecated]
    #[prost(message, optional, tag = "1")]
    pub time_series: ::core::option::Option<TimeSeries>,
    /// DEPRECATED. The status of the requested write operation for `time_series`.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<super::super::rpc::Status>,
}
/// Summary of the result of a failed request to write data to a time series.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTimeSeriesSummary {
    /// The number of points in the request.
    #[prost(int32, tag = "1")]
    pub total_point_count: i32,
    /// The number of points that were successfully written.
    #[prost(int32, tag = "2")]
    pub success_point_count: i32,
    /// The number of points that failed to be written. Order is not guaranteed.
    #[prost(message, repeated, tag = "3")]
    pub errors: ::prost::alloc::vec::Vec<create_time_series_summary::Error>,
}
/// Nested message and enum types in `CreateTimeSeriesSummary`.
pub mod create_time_series_summary {
    /// Detailed information about an error category.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Error {
        /// The status of the requested write operation.
        #[prost(message, optional, tag = "1")]
        pub status: ::core::option::Option<super::super::super::rpc::Status>,
        /// The number of points that couldn't be written because of `status`.
        #[prost(int32, tag = "2")]
        pub point_count: i32,
    }
}
/// The `QueryTimeSeries` request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTimeSeriesRequest {
    /// Required. The \[project\](<https://cloud.google.com/monitoring/api/v3#project_name>) on
    /// which to execute the request. The format is:
    ///
    /// ```text
    /// projects/\[PROJECT_ID_OR_NUMBER\]
    /// ```
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The query in the [Monitoring Query
    /// Language](<https://cloud.google.com/monitoring/mql/reference>) format.
    /// The default time zone is in UTC.
    #[prost(string, tag = "7")]
    pub query: ::prost::alloc::string::String,
    /// A positive number that is the maximum number of time_series_data to return.
    #[prost(int32, tag = "9")]
    pub page_size: i32,
    /// If this field is not empty then it must contain the `nextPageToken` value
    /// returned by a previous call to this method.  Using this field causes the
    /// method to return additional results from the previous method call.
    #[prost(string, tag = "10")]
    pub page_token: ::prost::alloc::string::String,
}
/// The `QueryTimeSeries` response.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTimeSeriesResponse {
    /// The descriptor for the time series data.
    #[prost(message, optional, tag = "8")]
    pub time_series_descriptor: ::core::option::Option<TimeSeriesDescriptor>,
    /// The time series data.
    #[prost(message, repeated, tag = "9")]
    pub time_series_data: ::prost::alloc::vec::Vec<TimeSeriesData>,
    /// If there are more results than have been returned, then this field is set
    /// to a non-empty value.  To see the additional results, use that value as
    /// `page_token` in the next call to this method.
    #[prost(string, tag = "10")]
    pub next_page_token: ::prost::alloc::string::String,
    /// Query execution errors that may have caused the time series data returned
    /// to be incomplete. The available data will be available in the
    /// response.
    #[prost(message, repeated, tag = "11")]
    pub partial_errors: ::prost::alloc::vec::Vec<super::super::rpc::Status>,
}
/// This is an error detail intended to be used with INVALID_ARGUMENT errors.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryErrorList {
    /// Errors in parsing the time series query language text. The number of errors
    /// in the response may be limited.
    #[prost(message, repeated, tag = "1")]
    pub errors: ::prost::alloc::vec::Vec<QueryError>,
    /// A summary of all the errors.
    #[prost(string, tag = "2")]
    pub error_summary: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod metric_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Manages metric descriptors, monitored resource descriptors, and
    /// time series data.
    #[derive(Debug, Clone)]
    pub struct MetricServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MetricServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MetricServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MetricServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MetricServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Lists monitored resource descriptors that match a filter. This method does not require a Workspace.
        pub async fn list_monitored_resource_descriptors(
            &mut self,
            request: impl tonic::IntoRequest<
                super::ListMonitoredResourceDescriptorsRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::ListMonitoredResourceDescriptorsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/ListMonitoredResourceDescriptors",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "ListMonitoredResourceDescriptors",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Gets a single monitored resource descriptor. This method does not require a Workspace.
        pub async fn get_monitored_resource_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetMonitoredResourceDescriptorRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::super::super::api::MonitoredResourceDescriptor>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/GetMonitoredResourceDescriptor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "GetMonitoredResourceDescriptor",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Lists metric descriptors that match a filter. This method does not require a Workspace.
        pub async fn list_metric_descriptors(
            &mut self,
            request: impl tonic::IntoRequest<super::ListMetricDescriptorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListMetricDescriptorsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/ListMetricDescriptors",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "ListMetricDescriptors",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Gets a single metric descriptor. This method does not require a Workspace.
        pub async fn get_metric_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMetricDescriptorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::api::MetricDescriptor>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/GetMetricDescriptor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "GetMetricDescriptor",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Creates a new metric descriptor.
        /// The creation is executed asynchronously and callers may check the returned
        /// operation to track its progress.
        /// User-created metric descriptors define
        /// [custom metrics](https://cloud.google.com/monitoring/custom-metrics).
        pub async fn create_metric_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateMetricDescriptorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::super::api::MetricDescriptor>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/CreateMetricDescriptor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "CreateMetricDescriptor",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Deletes a metric descriptor. Only user-created
        /// [custom metrics](https://cloud.google.com/monitoring/custom-metrics) can be
        /// deleted.
        pub async fn delete_metric_descriptor(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteMetricDescriptorRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/DeleteMetricDescriptor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "DeleteMetricDescriptor",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Lists time series that match a filter. This method does not require a Workspace.
        pub async fn list_time_series(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTimeSeriesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTimeSeriesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/ListTimeSeries",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "ListTimeSeries",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Creates or adds data to one or more time series.
        /// The response is empty if all time series in the request were written.
        /// If any time series could not be written, a corresponding failure message is
        /// included in the error response.
        pub async fn create_time_series(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTimeSeriesRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/CreateTimeSeries",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "CreateTimeSeries",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Creates or adds data to one or more service time series. A service time
        /// series is a time series for a metric from a Google Cloud service. The
        /// response is empty if all time series in the request were written. If any
        /// time series could not be written, a corresponding failure message is
        /// included in the error response. This endpoint rejects writes to
        /// user-defined metrics.
        /// This method is only for use by Google Cloud services. Use
        /// \[projects.timeSeries.create\]\[google.monitoring.v3.MetricService.CreateTimeSeries\]
        /// instead.
        pub async fn create_service_time_series(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTimeSeriesRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.monitoring.v3.MetricService/CreateServiceTimeSeries",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "google.monitoring.v3.MetricService",
                        "CreateServiceTimeSeries",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
