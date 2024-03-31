use std::time::Duration;

use crate::datastructures::{Competitor, Event};
use rand::prelude::*;

pub fn set_random_competitor_pr(competitor: &mut Competitor, event: Event) {
    let mut rng = rand::thread_rng();
    if let Some(_) = &mut competitor.wca_id {
        competitor.personal_records.insert(
            event,
            Duration::new(
                rng.gen_range(7..40),
                rng.gen_range(10..100) * 10 * 1000 * 1000,
            ),
        );
    }
}
