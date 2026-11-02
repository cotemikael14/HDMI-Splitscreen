//! Conversion des formats bruts de capture vers RGBA (ce que le GPU reçoit).

use zune_jpeg::zune_core::colorspace::ColorSpace;
use zune_jpeg::zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

/// Décode un MJPEG en RGBA. Retourne (largeur, hauteur, pixels).
pub fn mjpeg_to_rgba(data: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    let opts = DecoderOptions::default()
        .jpeg_set_out_colorspace(ColorSpace::RGBA)
        .set_strict_mode(false);
    let mut dec = JpegDecoder::new_with_options(data, opts);
    dec.decode_headers().map_err(|e| format!("JPEG: {e:?}"))?;
    let (w, h) = dec.dimensions().ok_or("JPEG sans dimensions")?;
    let size = dec.output_buffer_size().ok_or("JPEG: taille inconnue")?;
    let mut out = vec![0u8; size];
    dec.decode_into(&mut out).map_err(|e| format!("JPEG: {e:?}"))?;
    Ok((w, h, out))
}

#[inline(always)]
fn clamp(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

#[inline(always)]
fn yuv_px(y: i32, u: i32, v: i32, out: &mut [u8]) {
    // BT.601, plage limitée (standard des cartes de capture), en entiers.
    let c = 298 * (y - 16);
    out[0] = clamp((c + 409 * v + 128) >> 8);
    out[1] = clamp((c - 100 * u - 208 * v + 128) >> 8);
    out[2] = clamp((c + 516 * u + 128) >> 8);
    out[3] = 255;
}

/// YUYV (YUY2) -> RGBA. Pas de décompression = latence minimale.
pub fn yuyv_to_rgba(src: &[u8], w: usize, h: usize) -> Option<Vec<u8>> {
    let px = w * h;
    if src.len() < px * 2 {
        return None;
    }
    let mut out = vec![0u8; px * 4];
    for (s, d) in src[..px * 2].chunks_exact(4).zip(out.chunks_exact_mut(8)) {
        let u = s[1] as i32 - 128;
        let v = s[3] as i32 - 128;
        yuv_px(s[0] as i32, u, v, &mut d[0..4]);
        yuv_px(s[2] as i32, u, v, &mut d[4..8]);
    }
    Some(out)
}

/// NV12 -> RGBA.
pub fn nv12_to_rgba(src: &[u8], w: usize, h: usize) -> Option<Vec<u8>> {
    let px = w * h;
    if src.len() < px + px / 2 {
        return None;
    }
    let (y_plane, uv_plane) = src.split_at(px);
    let mut out = vec![0u8; px * 4];
    for row in 0..h {
        let uv_row = &uv_plane[(row / 2) * w..];
        let y_row = &y_plane[row * w..(row + 1) * w];
        let o_row = &mut out[row * w * 4..(row + 1) * w * 4];
        for col in 0..w {
            let uvi = (col / 2) * 2;
            let u = uv_row[uvi] as i32 - 128;
            let v = uv_row[uvi + 1] as i32 - 128;
            yuv_px(y_row[col] as i32, u, v, &mut o_row[col * 4..col * 4 + 4]);
        }
    }
    Some(out)
}

/// RGB / BGR / gris -> RGBA.
pub fn packed_to_rgba(src: &[u8], w: usize, h: usize, kind: Packed) -> Option<Vec<u8>> {
    let px = w * h;
    let bpp = if matches!(kind, Packed::Gray) { 1 } else { 3 };
    if src.len() < px * bpp {
        return None;
    }
    let mut out = vec![255u8; px * 4];
    match kind {
        Packed::Rgb => {
            for (s, d) in src.chunks_exact(3).zip(out.chunks_exact_mut(4)) {
                d[..3].copy_from_slice(s);
            }
        }
        Packed::Bgr => {
            for (s, d) in src.chunks_exact(3).zip(out.chunks_exact_mut(4)) {
                d[0] = s[2];
                d[1] = s[1];
                d[2] = s[0];
            }
        }
        Packed::Gray => {
            for (s, d) in src.iter().zip(out.chunks_exact_mut(4)) {
                d[0] = *s;
                d[1] = *s;
                d[2] = *s;
            }
        }
    }
    Some(out)
}

#[derive(Clone, Copy)]
pub enum Packed {
    Rgb,
    Bgr,
    Gray,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuyv_white_black() {
        // 2 pixels : blanc (Y=235) et noir (Y=16), sans couleur.
        let src = [235u8, 128, 16, 128];
        let out = yuyv_to_rgba(&src, 2, 1).unwrap();
        assert_eq!(&out[0..4], &[255, 255, 255, 255]);
        assert_eq!(&out[4..8], &[0, 0, 0, 255]);
    }

    #[test]
    fn nv12_size_check() {
        assert!(nv12_to_rgba(&[0u8; 5], 2, 2).is_none());
        assert!(nv12_to_rgba(&[16u8; 6], 2, 2).is_some());
    }
}

#[cfg(test)]
mod jpeg_tests {
    #[test]
    fn mjpeg_720p_decode_speed() {
        let Ok(data) = std::fs::read("/tmp/t.jpg") else { return };
        let t = std::time::Instant::now();
        for _ in 0..60 {
            let (w, h, px) = super::mjpeg_to_rgba(&data).unwrap();
            assert_eq!((w, h, px.len()), (1280, 720, 1280 * 720 * 4));
            assert!((px[0] as i32 - 200).abs() < 4 && px[3] == 255);
        }
        eprintln!("MJPEG 720p : {:.2} ms/image", t.elapsed().as_secs_f64() * 1000.0 / 60.0);
    }
}
