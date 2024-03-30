use std::time::Duration;

use crate::datastructures::Competitor;
use rand::prelude::*;

pub fn set_random_competitor_pr_avg(competitor: &mut Competitor) {
    let mut rng = rand::thread_rng();
    if let Some(_) = &mut competitor.wca_id {
        competitor.pr_3x3_avg = Some(Duration::new(
            rng.gen_range(7..40),
            rng.gen_range(10..100) * 10 * 1000 * 1000,
        ))
    }
}
