use iced::{
    mouse,
    widget::canvas::{self, Frame, Geometry, Path, Program, Renderer, Stroke},
    Color, Element, Length, Point, Rectangle, Size, Theme,
};
use crate::presentation::ui::Message;

pub struct Chart {
    data: Vec<(f32, f32)>,
}

impl Chart {
    pub fn new(data: Vec<(f32, f32)>) -> Self {
        Self { data }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        canvas::Canvas::new(ChartRenderer {
            data: self.data.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    }
}

struct ChartRenderer {
    data: Vec<(f32, f32)>,
}

impl Program<Message> for ChartRenderer {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, Size::new(bounds.width, bounds.height));

        // Draw chart background
        let background_color = Color::from_rgb(0.95, 0.95, 0.95);
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(bounds.width, bounds.height),
            background_color,
        );

        // Draw data points
        if !self.data.is_empty() {
            let x_scale = bounds.width / (self.data.len() - 1) as f32;
            let y_scale = bounds.height;

            let mut builder = Path::builder();
            builder.move_to(Point::new(0.0, bounds.height - self.data[0].1 * y_scale));

            for (i, (_x, y)) in self.data.iter().enumerate().skip(1) {
                builder.line_to(Point::new(
                    i as f32 * x_scale,
                    bounds.height - y * y_scale,
                ));
            }

            let path = builder.build();
            let line_color = Color::from_rgb(0.2, 0.6, 0.9);
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(line_color)
                    .with_width(2.0),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
} 