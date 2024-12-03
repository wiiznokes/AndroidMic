use crate::app::AppMsg;
use crate::config::AudioFormat;
use crate::map_bytes::MapBytes;
use byteordered::byteorder::ByteOrder;
use cosmic::iced::mouse::Cursor;
use cosmic::iced::{Point, Rectangle, Renderer};
use cosmic::iced_widget::canvas::Geometry;
use cosmic::theme;
use cosmic::widget::canvas::{self, path};
use ringbuffer::{AllocRingBuffer, RingBuffer};

const BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub struct AudioWave {
    buf: AllocRingBuffer<f32>,
}

impl AudioWave {
    pub fn new() -> Self {
        Self {
            buf: AllocRingBuffer::new(BUF_SIZE),
        }
    }

    pub fn push<B: ByteOrder>(&mut self, data: Vec<u8>, format: &AudioFormat) {
        #[inline]
        fn map_to_f32<B>(data: &mut impl Iterator<Item = u8>, format: &AudioFormat) -> Option<f32>
        where
            B: ByteOrder,
        {
            #[inline]
            fn map_to_primitive<B, F>(data: &mut impl Iterator<Item = u8>) -> Option<F>
            where
                B: ByteOrder,
                F: MapBytes,
            {
                F::map_bytes::<B>(data)
            }

            match format {
                AudioFormat::I8 => todo!(),
                AudioFormat::I16 => map_to_primitive::<B, i16>(data).map(|v| v as f32),
                AudioFormat::I24 => todo!(),
                AudioFormat::I32 => todo!(),
                AudioFormat::I48 => todo!(),
                AudioFormat::I64 => todo!(),
                AudioFormat::U8 => todo!(),
                AudioFormat::U16 => todo!(),
                AudioFormat::U24 => todo!(),
                AudioFormat::U32 => todo!(),
                AudioFormat::U48 => todo!(),
                AudioFormat::U64 => todo!(),
                AudioFormat::F32 => todo!(),
                AudioFormat::F64 => todo!(),
            }
        }

        let mut iter = data.into_iter();

        while let Some(value) = map_to_f32::<B>(&mut iter, format) {
            self.buf.push(value);
        }
    }

    pub fn tick(&mut self) {
        self.buf.dequeue();
    }
}

impl canvas::Program<AppMsg, theme::Theme> for AudioWave {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &theme::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let cosmic = theme.cosmic();
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let top_left = Point::new(
            frame.center().x - frame.size().width / 2. + 1.,
            frame.center().y - frame.size().height / 2. + 1.,
        );
        let bottom_right = Point::new(
            frame.center().x + frame.size().width / 2. - 1.,
            frame.center().y + frame.size().height / 2. - 1.,
        );
        let scale = bottom_right - top_left;

        let max_value = if self.autoscale {
            let max_point = self.points.iter().cloned().fold(0.0, f32::max);
            if max_point > 0.0 {
                max_point
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Draw rounded square background
        let bg_square =
            path::Path::rounded_rectangle(top_left, scale.into(), cosmic.radius_xs()[0].into());
        frame.stroke(
            &bg_square,
            canvas::Stroke {
                style: canvas::Style::Solid(cosmic.accent_color().into()),
                width: 2.0,
                ..Default::default()
            },
        );

        // Draw grid
        let mut grid_builder = path::Builder::new();
        let grid_step_x = scale.x / 10.;
        let grid_step_y = scale.y / 10.;
        for i in 1..10 {
            // Vertical
            let top = Point::new(top_left.x + grid_step_x * i as f32, top_left.y);
            let bottom = Point::new(top_left.x + grid_step_x * i as f32, bottom_right.y);
            grid_builder.move_to(top);
            grid_builder.line_to(bottom);

            // Horizontal
            let left = Point::new(top_left.x, top_left.y + grid_step_y * i as f32);
            let right = Point::new(bottom_right.x, top_left.y + grid_step_y * i as f32);
            grid_builder.move_to(left);
            grid_builder.line_to(right);
        }
        frame.stroke(
            &grid_builder.build(),
            canvas::Stroke {
                style: canvas::Style::Solid({
                    let mut half_accent = cosmic.accent_color();
                    half_accent.alpha = 0.25;
                    half_accent.into()
                }),
                ..Default::default()
            },
        );

        // Draw graph
        let step_length = scale.x / self.steps as f32;
        let mut builder = path::Builder::new();
        let mut shade_builder = path::Builder::new();
        let mut points = Vec::new();
        for i in 0..self.points.len() {
            points.push(Point::new(
                top_left.x + step_length * i as f32,
                bottom_right.y - self.points[i] / max_value * scale.y,
            ));
        }
        shade_builder.move_to(Point::new(top_left.x, bottom_right.y));
        shade_builder.line_to(points[0]);
        for i in 1..points.len() {
            let previous_point = points[i - 1];
            let control_previous =
                Point::new(previous_point.x + step_length * 0.5, previous_point.y);
            let point = points[i];
            let control_current = Point::new(point.x - step_length * 0.5, point.y);
            builder.move_to(previous_point);
            builder.bezier_curve_to(control_previous, control_current, point);
            shade_builder.bezier_curve_to(control_previous, control_current, point);
        }
        shade_builder.line_to(bottom_right);

        // Draw the curve
        frame.stroke(
            &builder.build(),
            canvas::Stroke {
                style: canvas::Style::Solid(cosmic.accent_color().into()),
                width: 2.0,
                line_join: canvas::LineJoin::Round,
                ..Default::default()
            },
        );

        // Draw the shading
        frame.fill(
            &shade_builder.build(),
            canvas::Fill {
                style: canvas::Style::Solid({
                    let mut half_accent = cosmic.accent_color();
                    half_accent.alpha = 0.25;
                    half_accent.into()
                }),
                ..Default::default()
            },
        );

        vec![frame.into_geometry()]
    }
}
