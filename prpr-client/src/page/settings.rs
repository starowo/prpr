use super::{Page, SharedState};
use crate::{dir, get_data, get_data_mut, save_data};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use prpr::{
    audio::{Audio, AudioClip, AudioHandle, DefaultAudio, PlayParams},
    core::{ParticleEmitter, SkinPack, JUDGE_LINE_PERFECT_COLOR, NOTE_WIDTH_RATIO_BASE},
    ext::{poll_future, LocalTask, RectExt, SafeTexture},
    fs,
    scene::{request_file, return_file, show_error, show_message, take_file},
    time::TimeManager,
    ui::{RectButton, Ui},
};
use std::ops::DerefMut;

const RESET_WAIT: f32 = 0.8;

pub struct SettingsPage {
    focus: bool,

    audio: DefaultAudio,
    cali_clip: AudioClip,
    cali_hit_clip: AudioClip,
    cali_handle: Option<AudioHandle>,
    cali_tm: TimeManager,
    cali_last: bool,
    click_texture: SafeTexture,
    emitter: ParticleEmitter,
    _skin: SkinPack, // prevent skin textures from being destroyed (ParticleEmitter holds a `weak` reference)

    chal_buttons: [RectButton; 6],

    load_skin_task: LocalTask<Result<(SkinPack, Option<String>)>>,
    reset_time: f32,
}

impl SettingsPage {
    pub async fn new() -> Result<Self> {
        let audio = DefaultAudio::new(get_data().config.audio_buffer_size)?;
        let cali_clip = audio.create_clip(load_file("cali.ogg").await?)?.0;
        let cali_hit_clip = audio.create_clip(load_file("cali_hit.ogg").await?)?.0;

        let mut cali_tm = TimeManager::new(1., true);
        cali_tm.force = 3e-2;
        let skin = SkinPack::load(fs::fs_from_assets("skin/")?.deref_mut()).await?;
        let emitter = ParticleEmitter::new(&skin, get_data().config.note_scale, skin.info.hide_particles)?;
        Ok(Self {
            focus: false,

            audio,
            cali_clip,
            cali_hit_clip,
            cali_handle: None,
            cali_tm,
            cali_last: false,
            click_texture: skin.note_style.click.clone(),
            emitter,
            _skin: skin,

            chal_buttons: [RectButton::new(); 6],

            load_skin_task: None,
            reset_time: f32::NEG_INFINITY,
        })
    }

    fn new_skin_task(path: Option<String>) -> LocalTask<Result<(SkinPack, Option<String>)>> {
        Some(Box::pin(async move {
            let skin = SkinPack::from_path(path.as_ref()).await?;
            Ok((
                skin,
                if let Some(path) = path {
                    let dst = format!("{}/chart.zip", dir::root()?);
                    std::fs::copy(path, &dst).context("保存皮肤失败")?;
                    Some(dst)
                } else {
                    None
                },
            ))
        }))
    }
}

impl Page for SettingsPage {
    fn label(&self) -> &'static str {
        "设置"
    }

    fn update(&mut self, focus: bool, state: &mut SharedState) -> Result<()> {
        let t = state.t;
        if !self.focus && focus {
            self.cali_handle = Some(self.audio.play(
                &self.cali_clip,
                PlayParams {
                    loop_: true,
                    volume: 0.7,
                    ..Default::default()
                },
            )?);
            self.cali_tm.reset();
        }
        if self.focus && !focus {
            save_data()?;
            if let Some(handle) = &mut self.cali_handle {
                self.audio.pause(handle)?;
            }
            self.cali_handle = None;
        }
        self.focus = focus;

        if let Some(handle) = &self.cali_handle {
            let pos = self.audio.position(handle)?;
            let now = self.cali_tm.now();
            if now > 2. {
                self.cali_tm.seek_to(now - 2.);
                self.cali_tm.dont_wait();
            }
            let now = self.cali_tm.now();
            if now - pos >= -1. {
                self.cali_tm.update(pos);
            }
        }
        if let Some(future) = &mut self.load_skin_task {
            if let Some(result) = poll_future(future.as_mut()) {
                self.load_skin_task = None;
                match result {
                    Err(err) => {
                        show_error(err.context("加载皮肤失败"));
                    }
                    Ok((skin, dst)) => {
                        self.click_texture = skin.note_style.click.clone();
                        self.emitter = ParticleEmitter::new(&skin, get_data().config.note_scale, skin.info.hide_particles)?;
                        self._skin = skin;
                        get_data_mut().config.skin_path = dst;
                        save_data()?;
                        show_message("加载皮肤成功");
                    }
                }
            }
        }
        if let Some((id, file)) = take_file() {
            if id == "skin" {
                self.load_skin_task = Self::new_skin_task(Some(file));
            } else {
                return_file(id, file);
            }
        }
        if t > self.reset_time + RESET_WAIT {
            self.reset_time = f32::NEG_INFINITY;
        }
        Ok(())
    }

    fn touch(&mut self, touch: &Touch, _state: &mut SharedState) -> Result<bool> {
        for (id, button) in self.chal_buttons.iter_mut().enumerate() {
            if button.touch(&touch) {
                use prpr::config::ChallengeModeColor::*;
                get_data_mut().config.challenge_color = [White, Green, Blue, Red, Golden, Rainbow][id].clone();
                save_data()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn render(&mut self, ui: &mut Ui, state: &mut SharedState) -> Result<()> {
        let t = state.t;
        let config = &mut get_data_mut().config;
        let s = 0.01;
        ui.scope(|ui| {
            ui.dy(0.01);
            ui.dx(0.02);
            ui.scope(|ui| {
                let s = 0.005;
                let r = ui.checkbox("自动游玩", &mut config.autoplay);
                ui.dy(r.h + s);
                let r = ui.checkbox("双押提示", &mut config.multiple_hint);
                ui.dy(r.h + s);
                let r = ui.checkbox("固定宽高比", &mut config.fix_aspect_ratio);
                ui.dy(r.h + s);
                let r = ui.checkbox("自动对齐时间", &mut config.adjust_time);
                ui.dy(r.h + s);
                let r = ui.checkbox("粒子效果", &mut config.particle);
                ui.dy(r.h + s);
                let r = ui.checkbox("激进优化", &mut config.aggressive);
                ui.dy(r.h + s);
                let mut low = config.sample_count == 1;
                let r = ui.checkbox("低性能模式", &mut low);
                config.sample_count = if low { 1 } else { 4 };
                ui.dy(r.h + s);
                let r = ui.slider("玩家 RKS", 1.0..17.0, 0.01, &mut config.player_rks, Some(0.45));
                ui.dy(r.h + s);
            });
            ui.dx(0.62);

            ui.scope(|ui| {
                let r = ui.slider("偏移(s)", -0.5..0.5, 0.005, &mut config.offset, None);
                ui.dy(r.h + s);
                let r = ui.slider("速度", 0.5..2.0, 0.005, &mut config.speed, None);
                ui.dy(r.h + s);
                let r = ui.slider("音符大小", 0.8..1.2, 0.005, &mut config.note_scale, None);
                self.emitter.set_scale(config.note_scale);
                ui.dy(r.h + s);
                let r = ui.slider("音乐音量", 0.0..2.0, 0.05, &mut config.volume_music, None);
                ui.dy(r.h + s);
                let r = ui.slider("音效音量", 0.0..2.0, 0.05, &mut config.volume_sfx, None);
                ui.dy(r.h + s);
                let r = ui.text("挑战模式颜色").size(0.4).draw();
                let chosen = config.challenge_color.clone() as usize;
                ui.dy(r.h + s * 2.);
                let dy = ui.scope(|ui| {
                    let mut max: f32 = 0.;
                    for (id, (name, button)) in ["白", "绿", "蓝", "红", "金", "彩"]
                        .into_iter()
                        .zip(self.chal_buttons.iter_mut())
                        .enumerate()
                    {
                        let r = ui.text(name).size(0.4).measure().feather(0.01);
                        button.set(ui, r);
                        ui.fill_rect(r, if chosen == id { ui.accent() } else { WHITE });
                        let color = if chosen == id { WHITE } else { ui.accent() };
                        ui.text(name).size(0.4).color(color).draw();
                        ui.dx(r.w + s);
                        max = max.max(r.h);
                    }
                    max
                });
                ui.dy(dy + s);

                let mut rks = config.challenge_rank as f32;
                let r = ui.slider("挑战模式等级", 0.0..48.0, 1., &mut rks, Some(0.45));
                config.challenge_rank = rks.round() as u32;
                ui.dy(r.h + s);
            });

            ui.scope(|ui| {
                ui.dx(0.65);
                let r = ui.text("皮肤").size(0.4).anchor(1., 0.).draw();
                let mut r = Rect::new(0.02, r.y - 0.01, 0.3, r.h + 0.02);
                if ui.button("choose_skin", r, config.skin_path.as_deref().unwrap_or("[默认]")) {
                    request_file("skin");
                }
                r.x += 0.3 + 0.02;
                r.w = 0.1;
                if ui.button("reset_skin", r, "重置") {
                    self.load_skin_task = Self::new_skin_task(None);
                }
                ui.dy(r.h + s * 2.);
                r.x -= 0.3 + 0.02;
                r.w = 0.4;
                let label = "音频缓冲区";
                let mut input = config.audio_buffer_size.map(|it| it.to_string()).unwrap_or_else(|| "[默认]".to_owned());
                ui.input(label, &mut input, 0.3);
                if input.trim().is_empty() || input == "[默认]" {
                    config.audio_buffer_size = None;
                } else {
                    match input.parse::<u32>() {
                        Err(_) => {
                            show_message("输入非法");
                        }
                        Ok(value) => {
                            config.audio_buffer_size = Some(value);
                        }
                    }
                }
                ui.dy(r.h + s * 2.);
                if ui.button(
                    "reset_all",
                    r,
                    if self.reset_time.is_finite() {
                        "确定？"
                    } else {
                        "恢复默认设定"
                    },
                ) {
                    if self.reset_time.is_finite() {
                        self.reset_time = f32::NEG_INFINITY;
                        *config = prpr::config::Config::default();
                        if let Err(err) = save_data() {
                            show_error(err.context("保存失败"));
                        } else {
                            self.load_skin_task = Self::new_skin_task(None);
                            show_message("设定恢复成功");
                        }
                    } else {
                        self.reset_time = t;
                    }
                }
            });

            let ct = (0.9, ui.top * 1.3);
            let len = 0.25;
            ui.fill_rect(Rect::new(ct.0 - len, ct.1 - 0.005, len * 2., 0.01), WHITE);
            let mut cali_t = self.cali_tm.now() as f32 - config.offset;
            if cali_t < 0. {
                cali_t += 2.;
            }
            if cali_t >= 2. {
                cali_t -= 2.;
            }
            if cali_t <= 1. {
                let w = NOTE_WIDTH_RATIO_BASE * config.note_scale * 2.;
                let h = w * self.click_texture.height() / self.click_texture.width();
                let r = Rect::new(ct.0 - w / 2., ct.1 + (cali_t - 1.) * 0.4, w, h);
                ui.fill_rect(r, (*self.click_texture, r));
                self.cali_last = true;
            } else {
                if self.cali_last {
                    let g = ui.to_global(ct);
                    self.emitter.emit_at(vec2(g.0, g.1), JUDGE_LINE_PERFECT_COLOR);
                    if self.focus {
                        let _ = self.audio.play(&self.cali_hit_clip, PlayParams::default());
                    }
                }
                self.cali_last = false;
            }
        });
        self.emitter.draw(get_frame_time());
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        save_data()?;
        self.cali_tm.pause();
        if let Some(handle) = &mut self.cali_handle {
            self.audio.pause(handle)?;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.cali_tm.resume();
        if let Some(handle) = &mut self.cali_handle {
            self.audio.resume(handle)?;
        }
        Ok(())
    }
}