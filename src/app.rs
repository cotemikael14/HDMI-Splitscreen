//! Interface : flux côte à côte, menu, paramètres, à propos.

use crate::capture::{self, Capture};
use crate::config::{self, Config, StreamConfig};
use eframe::egui::{
    self, Align2, Color32, ColorImage, FontId, Key, Pos2, Rect, TextureHandle, TextureOptions,
    Vec2, ViewportCommand,
};

struct Live {
    cfg: StreamConfig,
    cap: Capture,
    tex: Option<TextureHandle>,
}

pub struct App {
    config: Config,
    live: Vec<Live>,
    logo: Option<TextureHandle>,
    fullscreen: bool,
    show_about: bool,
    /// Copie de travail pendant qu'on édite les paramètres.
    draft: Option<Config>,
    devices: Vec<(u32, String)>,
    initial_vsync: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config, logo: Option<ColorImage>) -> Self {
        let ctx = &cc.egui_ctx;
        ctx.set_visuals(egui::Visuals::dark());
        let logo = logo.map(|img| ctx.load_texture("logo", img, TextureOptions::LINEAR));
        let mut app = Self {
            initial_vsync: config.vsync,
            config,
            live: Vec::new(),
            logo,
            fullscreen: false,
            show_about: false,
            draft: None,
            devices: Vec::new(),
        };
        app.restart_streams(ctx);
        app
    }

    fn restart_streams(&mut self, ctx: &egui::Context) {
        self.live.clear(); // Drop = arrêt des anciens threads
        for s in self.config.streams.iter().filter(|s| s.enabled) {
            self.live.push(Live {
                cfg: s.clone(),
                cap: Capture::start(s.clone(), ctx.clone()),
                tex: None,
            });
        }
    }

    fn set_fullscreen(&mut self, ctx: &egui::Context, on: bool) {
        self.fullscreen = on;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(on));
    }

    fn open_settings(&mut self) {
        self.devices = capture::list_devices();
        self.draft = Some(self.config.clone());
    }

    fn menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Fichier", |ui| {
                    if ui.button("Paramètres").clicked() {
                        self.open_settings();
                        ui.close_menu();
                    }
                    if ui.button("Quitter").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
                ui.menu_button("Affichage", |ui| {
                    let label = if self.fullscreen { "Quitter le plein écran (Échap)" } else { "Plein écran (F11)" };
                    if ui.button(label).clicked() {
                        let fs = !self.fullscreen;
                        self.set_fullscreen(ctx, fs);
                        ui.close_menu();
                    }
                    let mut changed = ui.checkbox(&mut self.config.keep_aspect, "Garder les proportions").changed();
                    changed |= ui.checkbox(&mut self.config.show_labels, "Afficher les noms").changed();
                    changed |= ui.checkbox(&mut self.config.show_info, "Afficher les infos (résolution, fps)").changed();
                    if changed {
                        config::save(&self.config);
                    }
                });
                ui.menu_button("Aide", |ui| {
                    if ui.button("À propos").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn video(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK))
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let painter = ui.painter();

                if self.live.is_empty() {
                    if let Some(logo) = &self.logo {
                        let size = fit(logo.size_vec2(), area.size() * 0.9);
                        painter.image(logo.id(), Rect::from_center_size(area.center(), size), full_uv(), Color32::WHITE);
                    }
                    return;
                }

                let n = self.live.len() as f32;
                let col_w = area.width() / n;
                let label_h = if self.config.show_labels || self.config.show_info { 22.0 } else { 0.0 };

                for (i, l) in self.live.iter().enumerate() {
                    let col = Rect::from_min_size(
                        Pos2::new(area.left() + col_w * i as f32, area.top()),
                        Vec2::new(col_w, area.height()),
                    );
                    if label_h > 0.0 {
                        let info = if self.config.show_info { l.cap.info() } else { String::new() };
                        let name = if self.config.show_labels { l.cfg.display_name.as_str() } else { "" };
                        let text = match (name.is_empty(), info.is_empty()) {
                            (false, false) => format!("{name}   ·   {info}"),
                            (true, _) => info,
                            (false, true) => name.to_string(),
                        };
                        painter.text(
                            Pos2::new(col.center().x, col.top() + label_h / 2.0),
                            Align2::CENTER_CENTER,
                            text,
                            FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                    }
                    let vid = Rect::from_min_max(Pos2::new(col.left(), col.top() + label_h), col.max);

                    match &l.tex {
                        Some(tex) => {
                            let dest = if self.config.keep_aspect {
                                Rect::from_center_size(vid.center(), fit(tex.size_vec2(), vid.size()))
                            } else {
                                vid
                            };
                            painter.image(tex.id(), dest, full_uv(), Color32::WHITE);
                        }
                        None => {
                            let st = l.cap.status();
                            let msg = if st.is_empty() {
                                format!("Aucun flux vidéo disponible ({})", l.cfg.display_name)
                            } else {
                                format!("Aucun flux vidéo disponible ({})\n{}", l.cfg.display_name, st)
                            };
                            let mut job = egui::text::LayoutJob::simple(
                                msg,
                                FontId::proportional(16.0),
                                Color32::GRAY,
                                (vid.width() - 24.0).max(50.0),
                            );
                            job.halign = egui::Align::Center;
                            let galley = ui.fonts(|f| f.layout_job(job));
                            let pos = Pos2::new(vid.center().x, vid.center().y - galley.size().y / 2.0);
                            painter.galley(pos, galley, Color32::GRAY);
                        }
                    }
                }
            });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.draft.take() else { return };
        let mut open = true;
        let mut save = false;
        let mut cancel = false;

        egui::Window::new("Paramètres")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{} périphérique(s) détecté(s)", self.devices.len()));
                    if ui.button("🔄 Rafraîchir").clicked() {
                        self.devices = capture::list_devices();
                    }
                });
                ui.separator();

                let mut remove = None;
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (i, s) in draft.streams.iter_mut().enumerate() {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let color = if s.enabled { Color32::GREEN } else { Color32::RED };
                                let (r, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                                ui.painter().circle_filled(r.center(), 5.0, color);
                                ui.checkbox(&mut s.enabled, "Activer");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("❌ Supprimer").clicked() {
                                        remove = Some(i);
                                    }
                                });
                            });
                            egui::Grid::new(("stream", i)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                                ui.label("Nom :");
                                ui.text_edit_singleline(&mut s.name);
                                ui.end_row();

                                ui.label("Source :");
                                let current = self
                                    .devices
                                    .iter()
                                    .find(|(idx, _)| *idx == s.source)
                                    .map(|(idx, n)| format!("{n} (ID: {idx})"))
                                    .unwrap_or_else(|| format!("Source {} (non branchée ?)", s.source));
                                egui::ComboBox::from_id_salt(("src", i))
                                    .selected_text(current)
                                    .width(280.0)
                                    .show_ui(ui, |ui| {
                                        for (idx, name) in &self.devices {
                                            if ui.selectable_label(s.source == *idx, format!("{name} (ID: {idx})")).clicked() {
                                                s.source = *idx;
                                                s.device_name = Some(name.clone());
                                            }
                                        }
                                    });
                                ui.end_row();

                                ui.label("Nom à afficher :");
                                ui.text_edit_singleline(&mut s.display_name);
                                ui.end_row();

                                ui.label("Qualité :");
                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_id_salt(("res", i))
                                        .selected_text(format!("{}×{}", s.width, s.height))
                                        .show_ui(ui, |ui| {
                                            for (w, h) in [(1280, 720), (1920, 1080), (640, 480)] {
                                                if ui.selectable_label(s.width == w && s.height == h, format!("{w}×{h}")).clicked() {
                                                    s.width = w;
                                                    s.height = h;
                                                }
                                            }
                                        });
                                    egui::ComboBox::from_id_salt(("fps", i))
                                        .selected_text(format!("{} fps", s.fps))
                                        .show_ui(ui, |ui| {
                                            for f in [60, 30] {
                                                ui.selectable_value(&mut s.fps, f, format!("{f} fps"));
                                            }
                                        });
                                });
                                ui.end_row();
                            });
                        });
                    }
                });
                if let Some(i) = remove {
                    draft.streams.remove(i);
                }

                if ui.button("➕ Ajouter un flux").clicked() {
                    draft.streams.push(StreamConfig {
                        source: draft.streams.len() as u32,
                        ..Default::default()
                    });
                }
                ui.separator();
                ui.checkbox(&mut draft.vsync, "Synchro verticale (V-Sync)")
                    .on_hover_text("Désactivée = moins de lag, mais possible déchirement de l'image.");
                if draft.vsync != self.initial_vsync {
                    ui.colored_label(Color32::YELLOW, "Redémarre l'application pour appliquer le V-Sync.");
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Enregistrer").clicked() {
                        save = true;
                    }
                    if ui.button("Annuler").clicked() {
                        cancel = true;
                    }
                });
            });

        if save {
            let streams_changed = draft.streams != self.config.streams;
            self.config = draft;
            config::save(&self.config);
            if streams_changed {
                self.restart_streams(ctx);
            }
        } else if open && !cancel {
            self.draft = Some(draft);
        }
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        egui::Window::new("À propos")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(format!("HDMI Splitscreen {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Version Rust — faible latence");
                    ui.label("Créateur : Mikaël Côté");
                });
            });
        self.show_about = open;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Raccourcis clavier
        if ctx.input(|i| i.key_pressed(Key::F11)) {
            let fs = !self.fullscreen;
            self.set_fullscreen(ctx, fs);
        }
        if self.fullscreen && ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.set_fullscreen(ctx, false);
        }

        // Envoie les nouvelles images au GPU (le GPU fait le redimensionnement).
        for l in &mut self.live {
            if let Some(img) = l.cap.take_frame() {
                match &mut l.tex {
                    Some(tex) => tex.set(img, TextureOptions::LINEAR),
                    None => l.tex = Some(ctx.load_texture(format!("cap{}", l.cfg.source), img, TextureOptions::LINEAR)),
                }
            }
        }

        // En plein écran, le menu apparaît seulement si la souris est en haut.
        let show_menu = !self.fullscreen
            || self.draft.is_some()
            || ctx.input(|i| i.pointer.hover_pos().is_some_and(|p| p.y < 50.0));
        if show_menu {
            self.menu(ctx);
        }

        self.video(ctx);
        self.settings_window(ctx);
        self.about_window(ctx);

        // Rafraîchit le statut ("Démarrage…", fps) même sans image.
        if self.live.iter().any(|l| l.tex.is_none()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

/// Taille maximale qui rentre dans `max` en gardant les proportions.
fn fit(src: Vec2, max: Vec2) -> Vec2 {
    if src.x <= 0.0 || src.y <= 0.0 {
        return max;
    }
    let s = (max.x / src.x).min(max.y / src.y);
    src * s
}
