use std::path::PathBuf;

use crate::datastructures::{Competitor, Event};
use plotters::{prelude::*, style::full_palette::GREY};

struct PlotData {
    histogram_data: Vec<u64>,
    lowest: u64,
    highest: u64,
    max_count: u64,
    title: String,
    x_desc: String,
    y_desc: String,
}

fn generate_plot_data(competitors: &[Competitor], event: &Event) -> PlotData {
    let times_sec: Vec<_> = competitors
        .iter()
        .filter_map(|comp| comp.personal_records.get(event).map(|time| time.as_secs()))
        .collect();
    let fastest_time = *times_sec.iter().min().unwrap_or(&0);
    let slowest_time = *times_sec.iter().max().unwrap_or(&0);
    let bar_count = slowest_time - fastest_time + 1;
    let mut histogram_data = vec![0u64; bar_count as usize];
    for time in times_sec {
        histogram_data[(time - fastest_time) as usize] += 1;
    }
    let max_count = *histogram_data.iter().max().unwrap_or(&0);

    let title = format!(
        "{} PR {} Histogram",
        event.pretty_name(),
        match event.use_average() {
            true => "Average",
            false => "Single",
        }
    );

    PlotData {
        histogram_data,
        lowest: fastest_time,
        highest: slowest_time,
        max_count,
        title,
        x_desc: "Solve time [s]".to_string(),
        y_desc: "Count".to_string(),
    }
}

fn generate_plot(pd: &PlotData, out_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = BitMapBackend::new(out_path, (1000, 400)).into_drawing_area();
    root_area.fill(&GREY)?;

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption(&pd.title, ("sans-serif", 40))
        .build_cartesian_2d((pd.lowest..pd.highest).into_segmented(), 0..pd.max_count)?;

    ctx.configure_mesh()
        .x_desc(&pd.x_desc)
        .y_desc(&pd.y_desc)
        .draw()?;

    ctx.draw_series((pd.lowest..).zip(pd.histogram_data.iter()).map(|(x, y)| {
        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);
        let mut bar = Rectangle::new([(x0, 0), (x1, *y)], BLUE.filled());
        bar.set_margin(0, 0, 5, 5);
        bar
    }))?;

    root_area.present()?;
    Ok(())
}

pub fn plot(
    competitors: &[Competitor],
    event: &Event,
    out_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let plot_data = generate_plot_data(competitors, event);
    generate_plot(&plot_data, out_path)
}
