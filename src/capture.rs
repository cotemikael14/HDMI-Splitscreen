//! Capture vidéo : un thread par source, qui garde SEULEMENT la dernière image.
//!
//! Analogie : une boîte aux lettres à une seule place. Le facteur (thread de
//! capture) remplace la lettre à chaque nouvelle image ; l'écran prend toujours
//! la plus récente. Jamais de file d'attente = jamais de retard qui s'accumule.

use crate::config::StreamConfig;
use crate::convert::{self, Packed};
use eframe::egui::{self, Color32, ColorImage};
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
};
use nokhwa::Camera;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Formats que l'on sait convertir.
static ALL_FORMATS: [FrameFormat; 6] = [
    FrameFormat::MJPEG,
    FrameFormat::YUYV,
    FrameFormat::NV12,
    FrameFormat::RAWRGB,
    FrameFormat::RAWBGR,
    FrameFormat::GRAY,
];

#[derive(Default)]
struct Shared {
    frame: Mutex<Option<ColorImage>>,
    status: Mutex<String>,
    info: Mutex<String>,
}

pub struct Capture {
    shared: Arc<Shared>,
    stop: Arc<AtomicBool>,
}

impl Capture {
    pub fn start(cfg: StreamConfig, ctx: egui::Context) -> Self {
        let shared = Arc::new(Shared::default());
        *shared.status.lock() = "Démarrage…".into();
        let stop = Arc::new(AtomicBool::new(false));
        let (s, st) = (shared.clone(), stop.clone());
        std::thread::Builder::new()
            .name(format!("capture-{}", cfg.source))
            .spawn(move || run(cfg, s, st, ctx))
            .expect("thread de capture");
        Self { shared, stop }
    }

    /// Prend la dernière image si une nouvelle est arrivée.
    pub fn take_frame(&self) -> Option<ColorImage> {
        self.shared.frame.lock().take()
    }

    pub fn status(&self) -> String {
        self.shared.status.lock().clone()
    }

    /// Ex. "MJPEG 1280×720 · 60 fps"
    pub fn info(&self) -> String {
        self.shared.info.lock().clone()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // Le thread s'arrête de lui-même à la prochaine image (pas de blocage de l'UI).
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Liste des périphériques de capture : (index, nom).
pub fn list_devices() -> Vec<(u32, String)> {
    match nokhwa::query(ApiBackend::Auto) {
        Ok(list) => list
            .into_iter()
            .enumerate()
            .map(|(i, info)| {
                let idx = match info.index() {
                    CameraIndex::Index(n) => *n,
                    CameraIndex::String(_) => i as u32,
                };
                (idx, info.human_name())
            })
            .collect(),
        Err(e) => {
            eprintln!("Liste des périphériques impossible: {e}");
            Vec::new()
        }
    }
}

/// Trouve le bon index : si le nom enregistré ne correspond plus à l'index
/// (ports USB échangés), on cherche le périphérique par son nom.
fn resolve_index(cfg: &StreamConfig) -> u32 {
    let Some(name) = &cfg.device_name else {
        return cfg.source;
    };
    let devices = list_devices();
    if devices.iter().any(|(i, n)| *i == cfg.source && n == name) {
        return cfg.source;
    }
    devices
        .iter()
        .find(|(_, n)| n == name)
        .map(|(i, _)| *i)
        .unwrap_or(cfg.source)
}

/// Choisit le meilleur format : résolution voulue, puis le plus de fps,
/// puis non compressé (YUYV/NV12) avant MJPEG à fps égal.
fn choose_format(formats: &[CameraFormat], w: u32, h: u32, fps: u32) -> Option<CameraFormat> {
    let target_px = (w * h) as i64;
    formats
        .iter()
        .filter(|f| ALL_FORMATS.contains(&f.format()))
        .min_by_key(|f| {
            let r = f.resolution();
            let exact = !(r.width() == w && r.height() == h);
            let below = r.height() < h;
            let diff = ((r.width() * r.height()) as i64 - target_px).abs();
            let fps_score = if f.frame_rate() >= fps {
                0
            } else {
                fps - f.frame_rate()
            };
            let compressed = matches!(f.format(), FrameFormat::MJPEG);
            (exact, below, diff, fps_score, compressed, u32::MAX - f.frame_rate())
        })
        .copied()
}

fn open_camera(cfg: &StreamConfig) -> Result<Camera, String> {
    let idx = resolve_index(cfg);
    let req = RequestedFormat::with_formats(RequestedFormatType::AbsoluteHighestFrameRate, &ALL_FORMATS);
    let mut cam = Camera::new(CameraIndex::Index(idx), req)
        .map_err(|e| format!("Ouverture de la source {idx} impossible : {e}"))?;

    if let Ok(formats) = cam.compatible_camera_formats() {
        if let Some(best) = choose_format(&formats, cfg.width, cfg.height, cfg.fps) {
            let req = RequestedFormat::with_formats(RequestedFormatType::Exact(best), &ALL_FORMATS);
            if let Err(e) = cam.set_camera_requset(req) {
                eprintln!("Format {best} refusé : {e}");
            }
        }
    }
    cam.open_stream()
        .map_err(|e| format!("Démarrage du flux {idx} impossible : {e}"))?;
    Ok(cam)
}

fn to_color_image(w: usize, h: usize, rgba: Vec<u8>) -> ColorImage {
    // Réinterprète le buffer RGBA en pixels egui sans recopier (même alignement).
    let pixels: Vec<Color32> = bytemuck::allocation::try_cast_vec(rgba)
        .unwrap_or_else(|(_, v)| {
            v.chunks_exact(4)
                .map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
                .collect()
        });
    ColorImage { size: [w, h], pixels }
}

fn decode(data: &[u8], fmt: CameraFormat) -> Option<ColorImage> {
    let w = fmt.resolution().width() as usize;
    let h = fmt.resolution().height() as usize;
    let (w, h, rgba) = match fmt.format() {
        FrameFormat::MJPEG => convert::mjpeg_to_rgba(data).ok()?,
        FrameFormat::YUYV => (w, h, convert::yuyv_to_rgba(data, w, h)?),
        FrameFormat::NV12 => (w, h, convert::nv12_to_rgba(data, w, h)?),
        FrameFormat::RAWRGB => (w, h, convert::packed_to_rgba(data, w, h, Packed::Rgb)?),
        FrameFormat::RAWBGR => (w, h, convert::packed_to_rgba(data, w, h, Packed::Bgr)?),
        FrameFormat::GRAY => (w, h, convert::packed_to_rgba(data, w, h, Packed::Gray)?),
    };
    Some(to_color_image(w, h, rgba))
}

fn sleep_unless_stopped(stop: &AtomicBool, total: Duration) {
    let start = Instant::now();
    while start.elapsed() < total && !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(windows)]
fn init_thread_com() {
    // Media Foundation demande COM sur chaque thread qui l'utilise.
    use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED as u32);
    }
}
#[cfg(not(windows))]
fn init_thread_com() {}

fn run(cfg: StreamConfig, shared: Arc<Shared>, stop: Arc<AtomicBool>, ctx: egui::Context) {
    init_thread_com();

    while !stop.load(Ordering::SeqCst) {
        let mut cam = match open_camera(&cfg) {
            Ok(c) => c,
            Err(e) => {
                *shared.status.lock() = e;
                ctx.request_repaint();
                // On réessaie : la source peut être occupée ou débranchée.
                sleep_unless_stopped(&stop, Duration::from_secs(1));
                continue;
            }
        };

        let fmt = cam.camera_format();
        *shared.status.lock() = String::new();
        let mut errors = 0u32;
        let mut count = 0u32;
        let mut tick = Instant::now();

        while !stop.load(Ordering::SeqCst) {
            let image = match cam.frame_raw() {
                Ok(data) => decode(&data, fmt),
                Err(e) => {
                    errors += 1;
                    if errors > 30 {
                        *shared.status.lock() = format!("Flux perdu : {e}");
                        break;
                    }
                    continue;
                }
            };
            errors = 0;
            if let Some(img) = image {
                *shared.frame.lock() = Some(img); // remplace l'ancienne image
                ctx.request_repaint(); // redessine tout de suite = latence minimale
                count += 1;
            }
            let el = tick.elapsed();
            if el >= Duration::from_secs(1) {
                let fps = count as f32 / el.as_secs_f32();
                *shared.info.lock() = format!(
                    "{} {}×{} · {:.0} fps",
                    fmt.format(),
                    fmt.resolution().width(),
                    fmt.resolution().height(),
                    fps
                );
                count = 0;
                tick = Instant::now();
            }
        }
        let _ = cam.stop_stream();
        drop(cam);
        if !stop.load(Ordering::SeqCst) {
            ctx.request_repaint();
            sleep_unless_stopped(&stop, Duration::from_secs(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nokhwa::utils::Resolution;

    fn f(w: u32, h: u32, ff: FrameFormat, fps: u32) -> CameraFormat {
        CameraFormat::new(Resolution::new(w, h), ff, fps)
    }

    #[test]
    fn prefers_720p_mjpeg60_over_yuyv10() {
        let list = [
            f(1920, 1080, FrameFormat::MJPEG, 30),
            f(1280, 720, FrameFormat::YUYV, 10),
            f(1280, 720, FrameFormat::MJPEG, 60),
            f(640, 480, FrameFormat::YUYV, 60),
        ];
        let best = choose_format(&list, 1280, 720, 60).unwrap();
        assert_eq!(best.format(), FrameFormat::MJPEG);
        assert_eq!(best.frame_rate(), 60);
    }

    #[test]
    fn prefers_uncompressed_at_same_fps() {
        let list = [
            f(1280, 720, FrameFormat::MJPEG, 60),
            f(1280, 720, FrameFormat::YUYV, 60),
        ];
        assert_eq!(choose_format(&list, 1280, 720, 60).unwrap().format(), FrameFormat::YUYV);
    }
}
