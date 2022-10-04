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

    #[test]
    fn test_table_output_with_title() {
        let pipelines = vec![
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
        ];

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
        let pipelines = vec![
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
        ];

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
}
