use std::time::Duration;

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

pub const BUF_SIZE: usize = 200;
pub const CYCLE_TIME: Duration = Duration::from_millis(3500);

#[derive(Debug)]
struct Value {
    time: u32,
    value: f32,
}

#[derive(Debug)]
pub struct AudioWave {
    now: u32,
    buf: AllocRingBuffer<Value>,
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
            now: 0,
        }
    }

    pub fn push<B: ByteOrder>(&mut self, data: impl Iterator<Item = u8>, format: &AudioFormat) {
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
            self.buf.push(Value {
                time: self.now,
                value,
            });
            if self.max < value.abs() {
                self.max = value.abs();
            }
        }
    }

    pub fn tick(&mut self) {
        self.now = self.now.wrapping_add(1);

        if let Some(v) = self.buf.front() {
            if v.time.wrapping_add(BUF_SIZE as u32) <= self.now {
                #[allow(clippy::float_equality_without_abs)]
                if self.max - v.value.abs() < f32::EPSILON {
                    self.max *= 0.9;
                }
                self.buf.dequeue();
            }
        }
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
        {
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
        }

        {
            let mut no_sound_builder = path::Builder::new();
            let mut sound_builder = path::Builder::new();
            // represent the distance from the time
            // 0 is the most recent value, BUF_SIZE - 1 the oldest
            let mut i = (BUF_SIZE - 1) as i32;
            let mut iter = self.buf.iter().peekable();
            let mut is_current_range_no_sound = false;

            while i >= 0 {
                match iter.next_if(|value| (self.now - value.time) as i32 == i) {
                    Some(value) => {
                        let x = right - i as f32 / BUF_SIZE as f32 * scale.x;
                        if is_current_range_no_sound {
                            no_sound_builder.line_to(Point::new(x, frame.center().y));
                            is_current_range_no_sound = false;
                        }

                        // why a factor of 2 doesn't remove the half of the line ???
                        let delta_y = (value.value.abs() / (self.max * 3.)) * scale.y;

                        sound_builder.move_to(Point::new(x, frame.center().y + delta_y));
                        sound_builder.line_to(Point::new(x, frame.center().y - delta_y));
                    }
                    None => {
                        if !is_current_range_no_sound {
                            no_sound_builder.move_to(Point::new(
                                right - i as f32 / BUF_SIZE as f32 * scale.x,
                                frame.center().y,
                            ));
                            is_current_range_no_sound = true;
                        }
                    }
                }

                i -= 1;
            }
            if is_current_range_no_sound {
                no_sound_builder.line_to(Point::new(right, frame.center().y));
            }
            frame.stroke(
                &no_sound_builder.build(),
                canvas::Stroke {
                    style: canvas::Style::Solid({
                        let half_accent = cosmic.accent_color();
                        half_accent.into()
                    }),
                    ..Default::default()
                },
            );
            frame.stroke(
                &sound_builder.build(),
                canvas::Stroke {
                    style: canvas::Style::Solid({
                        let half_accent = cosmic.accent_color();
                        half_accent.into()
                    }),
                    width: 1.5,
                    ..Default::default()
                },
            );
        }

        vec![frame.into_geometry()]
    }
}
