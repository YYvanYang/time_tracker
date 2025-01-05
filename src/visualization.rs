use plotters::prelude::*;
use crate::error::Result;

pub struct ChartBuilder {
    width: u32,
    height: u32,
    title: String,
    data: Vec<(f64, f64)>,
}

impl ChartBuilder {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            title: String::new(),
            data: Vec::new(),
        }
    }

    pub fn build(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (self.width, self.height))
                .into_drawing_area();
            root.fill(&WHITE)?;

            let mut chart = plotters::chart::ChartBuilder::on(&root)
                .caption(&self.title, ("sans-serif", 20))
                .margin(10)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(
                    0f64..24f64,
                    0f64..self.data.iter().map(|(_, y)| *y).fold(0./0., f64::max),
                )?;

            chart.configure_mesh().draw()?;

            chart.draw_series(LineSeries::new(
                self.data.clone(),
                &BLUE,
            ))?;
        }
        Ok(buffer)
    }
}