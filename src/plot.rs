use std::path::PathBuf;

use crate::datastructures::{Competitor, Event};
use plotters::prelude::*;

pub fn plot(
    competitors: &[Competitor],
    event: &Event,
    out_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let times_sec: Vec<_> = competitors
        .iter()
        .filter_map(|comp| comp.personal_records.get(event).map(|time| time.as_secs()))
        .collect();
    let fastest_time = *times_sec.iter().min().unwrap_or(&0);
    let slowest_time = *times_sec.iter().max().unwrap_or(&0);
    let bar_count = slowest_time - fastest_time + 1;
    let mut counts = vec![0u64; bar_count as usize];
    for time in times_sec {
        counts[(time - fastest_time) as usize] += 1;
    }
    let max_count = *counts.iter().max().unwrap_or(&0);

    let root_area = BitMapBackend::new(out_path, (1000, 400)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("3x3 PR Ao5 Histogram", ("sans-serif", 40))
        .build_cartesian_2d((fastest_time..slowest_time).into_segmented(), 0..max_count)?;

    ctx.configure_mesh()
        .x_desc("Solve time [s]")
        .y_desc("Count")
        .draw()?;

    ctx.draw_series((fastest_time..).zip(counts.iter()).map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0), (x1, *y)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))?;

    root_area.present()?;
    Ok(())
}
