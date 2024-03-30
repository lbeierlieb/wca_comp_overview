use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Competitor {
    pub name: String,
    pub wca_id: Option<String>,
    pub pr_3x3_avg: Option<Duration>,
}

impl Competitor {
    pub fn new(name: String, wca_id: Option<String>) -> Self {
        Competitor {
            name,
            wca_id,
            pr_3x3_avg: None,
        }
    }
}
