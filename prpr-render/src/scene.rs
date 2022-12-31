use crate::{VideoConfig, INFO_EDIT, VIDEO_CONFIG};
use anyhow::{bail, Result};
use macroquad::prelude::*;
use prpr::{
    config::Config,
    ext::{poll_future, screen_aspect},
    fs::{FileSystem, PatchedFileSystem},
    info::ChartInfo,
    scene::{show_message, LoadingScene, NextScene, Scene},
    time::TimeManager,
    ui::{render_chart_info, ChartInfoEdit, Scroll, Ui},
};
use std::{future::Future, pin::Pin};

pub struct MainScene {
    target: Option<RenderTarget>,

    scroll: Scroll,
    edit: ChartInfoEdit,
    config: Config,
    fs: Box<dyn FileSystem>,
    next_scene: Option<NextScene>,
    v_config: VideoConfig,

    future_to_loading: Option<Pin<Box<dyn Future<Output = Result<LoadingScene>>>>>,
}

impl MainScene {
    pub fn new(target: Option<RenderTarget>, info: ChartInfo, config: Config, fs: Box<dyn FileSystem>) -> Self {
        Self {
            target,

            scroll: Scroll::new(),
            edit: ChartInfoEdit::new(info),
            config,
            fs,
            next_scene: None,
            v_config: VideoConfig::default(),

            future_to_loading: None,
        }
    }
}

impl Scene for MainScene {
    fn touch(&mut self, tm: &mut TimeManager, touch: Touch) -> Result<()> {
        self.scroll.touch(&touch, tm.now() as _);
        Ok(())
    }

    fn update(&mut self, tm: &mut TimeManager) -> Result<()> {
        self.scroll.update(tm.now() as _);
        if let Some(future) = &mut self.future_to_loading {
            if let Some(scene) = poll_future(future.as_mut()) {
                self.future_to_loading = None;
                self.next_scene = Some(NextScene::Overlay(Box::new(scene?)));
            }
        }
        Ok(())
    }

    fn render(&mut self, _tm: &mut TimeManager, ui: &mut Ui) -> Result<()> {
        set_camera(&Camera2D {
            zoom: vec2(1., -screen_aspect()),
            render_target: self.target,
            ..Default::default()
        });
        clear_background(GRAY);
        let width = 1.;
        ui.scope(|ui| {
            ui.dx(-1.);
            ui.dy(-ui.top);
            let h = 0.1;
            let pad = 0.01;
            self.scroll.size((width, ui.top * 2. - h));
            self.scroll.render(ui, |ui| {
                let (w, mut h) = render_chart_info(ui, &mut self.edit, width);
                ui.scope(|ui| {
                    h += 0.01;
                    ui.dy(h);
                    let width = ui.text("一二三四").size(0.4).measure().w;
                    ui.dx(width);
                    let res = self.v_config.resolution;
                    let mut string = format!("{}x{}", res.0, res.1);
                    let r = ui.input("分辨率", &mut string, 0.8);
                    match || -> Result<(u32, u32)> {
                        if let Some((w, h)) = string.split_once(&['x', 'X', '×', '*']) {
                            Ok((w.parse::<u32>()?, h.parse::<u32>()?))
                        } else {
                            bail!("格式应当为 “宽x高”")
                        }
                    }() {
                        Err(err) => {
                            warn!("{:?}", err);
                            show_message("输入非法");
                        }
                        Ok(value) => {
                            self.v_config.resolution = value;
                        }
                    }
                    ui.dy(r.h + pad);
                    h += r.h;

                    let mut string = self.v_config.fps.to_string();
                    let old = string.clone();
                    let r = ui.input("FPS", &mut string, 0.8);
                    if string != old {
                        match string.parse::<u32>() {
                            Err(err) => {
                                warn!("{:?}", err);
                                show_message("输入非法");
                            }
                            Ok(value) => {
                                self.v_config.fps = value;
                            }
                        }
                    }
                    ui.dy(r.h + pad);
                    h += r.h;

                    let mut string = format!("{:.2}", self.v_config.ending_length);
                    let old = string.clone();
                    let r = ui.input("结算时间", &mut string, 0.8);
                    if string != old {
                        match string.parse::<f64>() {
                            Err(err) => {
                                warn!("{:?}", err);
                                show_message("输入非法");
                            }
                            Ok(value) => {
                                if !value.is_finite() || value < 0. {
                                    show_message("输入非法");
                                }
                                self.v_config.ending_length = value;
                            }
                        }
                    }
                    ui.dy(r.h + pad);
                    h += r.h;

                    let r = ui.checkbox("启用硬件加速", &mut self.v_config.hardware_accel);
                    ui.dy(r.h + pad);
                    h += r.h;
                });
                (w, h)
            });
            let dx = width / 2.;
            let mut r = Rect::new(pad, ui.top * 2. - h + pad, dx - pad * 2., h - pad * 2.);
            if ui.button("preview", r, "预览") {
                let info = self.edit.info.clone();
                let config = self.config.clone();
                let fs = self.fs.clone_box();
                let edit = self.edit.clone();
                self.future_to_loading = Some(Box::pin(async move {
                    LoadingScene::new(info, config, Box::new(PatchedFileSystem(fs, edit.to_patches().await?)), None, None).await
                }));
            }
            r.x += dx;
            if ui.button("render", r, "渲染") {
                *INFO_EDIT.lock().unwrap() = Some(self.edit.clone());
                *VIDEO_CONFIG.lock().unwrap() = Some(self.v_config.clone());
                self.next_scene = Some(NextScene::Exit);
            }
        });
        Ok(())
    }

    fn next_scene(&mut self, _tm: &mut TimeManager) -> NextScene {
        self.next_scene.take().unwrap_or_default()
    }
}