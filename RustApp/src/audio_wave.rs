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

const BUF_SIZE: usize = 500;

#[derive(Debug)]
pub struct AudioWave {
    buf: AllocRingBuffer<f32>,
    max: f32,
}

#[test]
fn a() {
    let mut b = AllocRingBuffer::new(5);

    b.push(5);
    b.push(6);

    for a in b {
        println!("{a}")
    }
}

impl AudioWave {
    pub fn new() -> Self {
        Self {
            buf: AllocRingBuffer::new(BUF_SIZE),
            max: 1.,
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

        if let Some(value) = map_to_f32::<B>(&mut iter, format) {
            self.buf.push(value);
            if self.max < value.abs() {
                self.max = value.abs();
            }
        }

        // while let Some(value) = map_to_f32::<B>(&mut iter, format) {
        //     self.buf.push(value);
        // }
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

        let top = frame.center().y - frame.size().height / 2. + 1.;
        let left = frame.center().x - frame.size().width / 2. + 1.;
        let right = frame.center().x + frame.size().width / 2. - 1.;
        let bottom = frame.center().y + frame.size().height / 2. - 1.;

        let top_left = Point::new(left, top);
        let bottom_right = Point::new(right, bottom);
        let scale = bottom_right - top_left;

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

        // draw missing line
        let missing = BUF_SIZE - self.buf.len();
        let mut no_sound_builder = path::Builder::new();

        let left_center = Point::new(left, frame.center().y);
        let end_no_sound: Point = Point::new(
            left + missing as f32 / BUF_SIZE as f32 * scale.x,
            frame.center().y,
        );
        no_sound_builder.move_to(left_center);
        no_sound_builder.line_to(end_no_sound);
        frame.stroke(
            &no_sound_builder.build(),
            canvas::Stroke {
                style: canvas::Style::Solid({
                    let half_accent = cosmic.accent_color();
                    half_accent.into()
                }),
                width: 1.5,
                ..Default::default()
            },
        );

        // draw sound
        let mut builder = path::Builder::new();

        builder.move_to(end_no_sound);
        for (pos, value) in self.buf.iter().enumerate() {
            if value.is_sign_positive() {
                builder.line_to(Point::new(
                    end_no_sound.x + pos as f32 / self.buf.len() as f32 * scale.x,
                    frame.center().y + value / self.max * scale.y,
                ));
            }
        }
        builder.line_to(Point::new(right, frame.center().y));
        for (pos, value) in self.buf.iter().rev().enumerate() {
            let pos = self.buf.len() - (pos + 1);
            if value.is_sign_negative() {
                builder.line_to(Point::new(
                    end_no_sound.x + pos as f32 / self.buf.len() as f32 * scale.x,
                    frame.center().y + value / self.max * scale.y,
                ));
            }
        }
        builder.line_to(end_no_sound);
        frame.fill(
            &builder.build(),
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
