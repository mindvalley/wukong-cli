use graphql_client::GraphQLQuery;

pub enum AppsignalTimeFrame {
    R1H,
    R4H,
    R8H,
    R12H,
    R24H,
    R7D,
    R30D,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/appsignal_average_error_rate.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AppsignalAverageErrorRateQuery;

impl From<AppsignalTimeFrame> for appsignal_average_error_rate_query::AppsignalTimeframe {
    fn from(value: AppsignalTimeFrame) -> Self {
        use appsignal_average_error_rate_query::AppsignalTimeframe as WukongAPIAppsignalTimeframe;

        match value {
            AppsignalTimeFrame::R1H => WukongAPIAppsignalTimeframe::R1H,
            AppsignalTimeFrame::R4H => WukongAPIAppsignalTimeframe::R4H,
            AppsignalTimeFrame::R8H => WukongAPIAppsignalTimeframe::R8H,
            AppsignalTimeFrame::R12H => WukongAPIAppsignalTimeframe::R12H,
            AppsignalTimeFrame::R24H => WukongAPIAppsignalTimeframe::R24H,
            AppsignalTimeFrame::R7D => WukongAPIAppsignalTimeframe::R7D,
            AppsignalTimeFrame::R30D => WukongAPIAppsignalTimeframe::R30D,
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/appsignal_average_latency.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AppsignalAverageLatencyQuery;

impl From<AppsignalTimeFrame> for appsignal_average_latency_query::AppsignalTimeframe {
    fn from(value: AppsignalTimeFrame) -> Self {
        use appsignal_average_latency_query::AppsignalTimeframe as WukongAPIAppsignalTimeframe;

        match value {
            AppsignalTimeFrame::R1H => WukongAPIAppsignalTimeframe::R1H,
            AppsignalTimeFrame::R4H => WukongAPIAppsignalTimeframe::R4H,
            AppsignalTimeFrame::R8H => WukongAPIAppsignalTimeframe::R8H,
            AppsignalTimeFrame::R12H => WukongAPIAppsignalTimeframe::R12H,
            AppsignalTimeFrame::R24H => WukongAPIAppsignalTimeframe::R24H,
            AppsignalTimeFrame::R7D => WukongAPIAppsignalTimeframe::R7D,
            AppsignalTimeFrame::R30D => WukongAPIAppsignalTimeframe::R30D,
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/appsignal_average_throughput.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AppsignalAverageThroughputQuery;

impl From<AppsignalTimeFrame> for appsignal_average_throughput_query::AppsignalTimeframe {
    fn from(value: AppsignalTimeFrame) -> Self {
        use appsignal_average_throughput_query::AppsignalTimeframe as WukongAPIAppsignalTimeframe;

        match value {
            AppsignalTimeFrame::R1H => WukongAPIAppsignalTimeframe::R1H,
            AppsignalTimeFrame::R4H => WukongAPIAppsignalTimeframe::R4H,
            AppsignalTimeFrame::R8H => WukongAPIAppsignalTimeframe::R8H,
            AppsignalTimeFrame::R12H => WukongAPIAppsignalTimeframe::R12H,
            AppsignalTimeFrame::R24H => WukongAPIAppsignalTimeframe::R24H,
            AppsignalTimeFrame::R7D => WukongAPIAppsignalTimeframe::R7D,
            AppsignalTimeFrame::R30D => WukongAPIAppsignalTimeframe::R30D,
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/query/appsignal_apps.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AppsignalAppsQuery;
