use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use chrono_humanize::HumanTime;
use std::fmt::Display;
use tabled::{style::Style, Panel, Table, Tabled};

#[derive(Clone)]
pub struct TableOutput<T>
where
    T: Tabled,
{
    pub title: Option<String>,
    pub header: Option<String>,
    pub data: Vec<T>,
}

impl<T> Display for TableOutput<T>
where
    T: Tabled,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(title) = &self.title {
            writeln!(f, "{title}")?;
        }

        let mut table = Table::new(&self.data);
        if let Some(header) = &self.header {
            table = table.with(Panel(header, 0));
        }

        let table = table.with(Style::modern()).to_string();
        writeln!(f, "{table}")?;

        Ok(())
    }
}

pub fn fmt_option_milliseconds(o: &Option<i64>) -> String {
    match o {
        Some(s) => {
            let duration = Duration::milliseconds(*s);
            let seconds = duration.num_seconds() % 60;
            let minutes = (duration.num_seconds() / 60) % 60;
            format!("{} mins {} secs", minutes, seconds)
        }
        None => "N/A".to_string(),
    }
}

pub fn fmt_option_timestamp(o: &Option<i64>) -> String {
    match o {
        Some(s) => fmt_timestamp(s),
        None => "N/A".to_string(),
    }
}

pub fn fmt_timestamp(o: &i64) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(o / 1000, (o % 1000) as u32 * 1_000_000).unwrap();
    let dt = DateTime::<Utc>::from_utc(naive, Utc).with_timezone(&Local);
    format!("{}", dt.format("%Y %b %d %H:%M:%S %p"))
}

pub fn fmt_option_string(o: &Option<String>) -> String {
    match o {
        Some(s) => s.to_string(),
        None => "N/A".to_string(),
    }
}

pub fn fmt_option_human_timestamp(o: &Option<i64>) -> String {
    match o {
        Some(s) => fmt_human_timestamp(s),
        None => "N/A".to_string(),
    }
}

pub fn fmt_human_timestamp(o: &i64) -> String {
    let naive = NaiveDateTime::from_timestamp_opt(o / 1000, (o % 1000) as u32 * 1_000_000).unwrap();
    let dt = DateTime::<Utc>::from_utc(naive, Utc).with_timezone(&Local);
    format!("{}", HumanTime::from(dt))
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Tabled)]
    struct Pipeline {
        #[tabled(rename = "Pipeline Name")]
        name: String,
        #[tabled(rename = "Status")]
        status: String,
    }

    fn setup_test_pipelines() -> Vec<Pipeline> {
        vec![
            Pipeline {
                name: "Pipeline 1".to_string(),
                status: "Done".to_string(),
            },
            Pipeline {
                name: "Pipeline 2".to_string(),
                status: "Pending".to_string(),
            },
            Pipeline {
                name: "Pipeline 3".to_string(),
                status: "Terminated".to_string(),
            },
        ]
    }

    #[test]
    fn test_table_output_with_title() {
        let pipelines = setup_test_pipelines();

        let output = TableOutput {
            title: Some("Pipeline list:".to_string()),
            header: None,
            data: pipelines,
        };

        assert_eq!(
            format!("{output}"),
            concat!(
                "Pipeline list:\n",
                "┌───────────────┬────────────┐\n",
                "│ Pipeline Name │ Status     │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 1    │ Done       │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 2    │ Pending    │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 3    │ Terminated │\n",
                "└───────────────┴────────────┘\n"
            )
        );
    }

    #[test]
    fn test_table_output_with_title_and_header() {
        let pipelines = setup_test_pipelines();

        let output = TableOutput {
            title: Some("Pipeline list:".to_string()),
            header: Some("Production Env".to_string()),
            data: pipelines,
        };

        assert_eq!(
            format!("{output}"),
            concat!(
                "Pipeline list:\n",
                "┌───────────────┬────────────┐\n",
                "│ Production Env             │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline Name │ Status     │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 1    │ Done       │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 2    │ Pending    │\n",
                "├───────────────┼────────────┤\n",
                "│ Pipeline 3    │ Terminated │\n",
                "└───────────────┴────────────┘\n"
            )
        );
    }

    #[test]
    fn test_fmt_option_milliseconds_should_return_correct_format() {
        let some_result = fmt_option_milliseconds(&Some(1664524161666));
        assert_eq!(some_result, "49 mins 21 secs");

        let none_result = fmt_option_milliseconds(&None);
        assert_eq!(none_result, "N/A");
    }

    #[test]
    fn test_fmt_option_timestamp_should_return_correct_format() {
        let some_result = fmt_option_timestamp(&Some(1664524161666));
        assert_eq!(some_result, "2022 Sep 30 15:49:21 PM");

        let none_result = fmt_option_timestamp(&None);
        assert_eq!(none_result, "N/A");
    }

    #[test]
    fn test_fmt_timestamp_should_return_correct_format() {
        let result = fmt_timestamp(&1664524161666);
        assert_eq!(result, "2022 Sep 30 15:49:21 PM");
    }

    #[test]
    fn test_fmt_option_string_should_return_correct_format() {
        let some_result = fmt_option_string(&Some("Pipeline 1".to_string()));
        assert_eq!(some_result, "Pipeline 1");

        let none_result = fmt_option_string(&None);
        assert_eq!(none_result, "N/A");
    }

    #[test]
    fn test_fmt_option_human_timestamp_should_return_correct_format() {
        let some_result = fmt_option_human_timestamp(&Some(1664524161666));
        // the result should be something like `4 days ago`
        // but because it is relative time, so we just make sure the format is correct
        assert!(some_result.contains("ago"));

        let none_result = fmt_option_human_timestamp(&None);
        assert_eq!(none_result, "N/A");
    }
}
