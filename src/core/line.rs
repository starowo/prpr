use super::{
    Anim, AnimFloat, Matrix, Note, Object, Resource, ScopedTransform, Vector,
    JUDGE_LINE_PERFECT_COLOR,
};
use macroquad::prelude::*;

#[derive(Default)]
pub enum JudgeLineKind {
    #[default]
    Normal,
    Texture(Texture2D),
    Text(Anim<String>),
}

#[derive(Default)]
pub struct JudgeLine {
    pub object: Object,
    pub kind: JudgeLineKind,
    pub height: AnimFloat,
    pub notes_above: Vec<Note>,
    pub notes_below: Vec<Note>,
}

impl JudgeLine {
    pub fn update(&mut self, res: &mut Resource) {
        for note in &mut self.notes_above {
            note.update(res, &mut self.object);
        }
        for note in &mut self.notes_below {
            note.update(res, &mut self.object);
        }
        self.object.set_time(res.time);
        if let JudgeLineKind::Text(anim) = &mut self.kind {
            anim.set_time(res.time);
        }
        self.height.set_time(res.time);
    }

    pub fn render(&self, res: &mut Resource) {
        self.object.now().apply_render(|| {
            self.object.now_scale().apply_render(|| match &self.kind {
                JudgeLineKind::Normal => {
                    let mut c = JUDGE_LINE_PERFECT_COLOR;
                    c.a = self.object.now_color().a;
                    draw_line(-6.0, 0.0, 6.0, 0.0, 0.01, c);
                }
                JudgeLineKind::Texture(texture) => {
                    let hw = texture.width() / 2.;
                    let hh = texture.height() / 2.;
                    draw_texture_ex(
                        *texture,
                        -hw,
                        -hh,
                        self.object.now_color(),
                        DrawTextureParams {
                            dest_size: Some(vec2(hw * 2., hh * 2.)),
                            flip_y: true,
                            ..Default::default()
                        },
                    )
                }
                JudgeLineKind::Text(anim) => {
                    let now = anim.now();
                    let size = 100;
                    let scale = 0.0008;
                    let dim = measure_text(&now, Some(res.font), size, scale);
                    Matrix::identity()
                        .append_nonuniform_scaling(&Vector::new(1.0, -1.0))
                        .apply_render(|| {
                            draw_text_ex(
                                &now,
                                -dim.width / 2.,
                                dim.offset_y - dim.height / 2.,
                                TextParams {
                                    font: res.font,
                                    font_size: size,
                                    font_scale: scale,
                                    color: self.object.now_color(),
                                    ..Default::default()
                                },
                            );
                        });
                }
            });
            let height = self.height.now();
            for note in &self.notes_above {
                note.render(res, height);
            }
            Matrix::identity()
                .append_nonuniform_scaling(&Vector::new(1.0, -1.0))
                .apply_render(|| {
                    for note in &self.notes_below {
                        note.render(res, height);
                    }
                });
        });
    }
}