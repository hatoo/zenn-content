use crate::expected_rank_distribution;
use plotters::prelude::*;

pub fn plot(actual: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("1.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Rank Distribution", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..32f32, 0f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(
            LineSeries::new(
                expected_rank_distribution()
                    .iter()
                    .enumerate()
                    .map(|(x, y)| (x as f32, *y as f32)),
                RED.filled(),
            )
            .point_size(4),
        )?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(
            LineSeries::new(
                actual
                    .iter()
                    .enumerate()
                    .map(|(x, y)| (x as f32, *y as f32)),
                BLUE.filled(),
            )
            .point_size(4),
        )?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    root.present()?;

    Ok(())
}
