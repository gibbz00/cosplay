use cosplay_protocols_wayland::wl_shm::PixelFormat;

pub const fn bytes_per_pixel(format: PixelFormat) -> Option<u8> {
    let n = match format {
        // 8
        PixelFormat::C8 | PixelFormat::Rgb332 | PixelFormat::Bgr233 | PixelFormat::R8 | PixelFormat::Y8 => 1,
        // 16
        PixelFormat::Xrgb4444
        | PixelFormat::Xbgr4444
        | PixelFormat::Rgbx4444
        | PixelFormat::Bgrx4444
        | PixelFormat::Argb4444
        | PixelFormat::Abgr4444
        | PixelFormat::Rgba4444
        | PixelFormat::Bgra4444
        | PixelFormat::Xrgb1555
        | PixelFormat::Xbgr1555
        | PixelFormat::Rgbx5551
        | PixelFormat::Bgrx5551
        | PixelFormat::Argb1555
        | PixelFormat::Abgr1555
        | PixelFormat::Rgba5551
        | PixelFormat::Bgra5551
        | PixelFormat::Rgb565
        | PixelFormat::Bgr565
        | PixelFormat::R16
        | PixelFormat::Rg88
        | PixelFormat::Gr88
        | PixelFormat::R10
        | PixelFormat::R12
        | PixelFormat::R16f => 2,
        // 24
        PixelFormat::Rgb888 | PixelFormat::Bgr888 | PixelFormat::Vuy888 => 3,
        // 32
        PixelFormat::Argb8888
        | PixelFormat::Xrgb8888
        | PixelFormat::Xbgr8888
        | PixelFormat::Rgbx8888
        | PixelFormat::Bgrx8888
        | PixelFormat::Abgr8888
        | PixelFormat::Rgba8888
        | PixelFormat::Bgra8888
        | PixelFormat::Xrgb2101010
        | PixelFormat::Xbgr2101010
        | PixelFormat::Rgbx1010102
        | PixelFormat::Bgrx1010102
        | PixelFormat::Argb2101010
        | PixelFormat::Abgr2101010
        | PixelFormat::Rgba1010102
        | PixelFormat::Bgra1010102
        | PixelFormat::Yuyv
        | PixelFormat::Yvyu
        | PixelFormat::Uyvy
        | PixelFormat::Vyuy
        | PixelFormat::Ayuv
        | PixelFormat::Rg1616
        | PixelFormat::Gr1616
        | PixelFormat::Xyuv8888
        | PixelFormat::Y410
        | PixelFormat::Xvyu2101010
        | PixelFormat::Avuy8888
        | PixelFormat::Xvuy8888
        | PixelFormat::Gr1616f
        | PixelFormat::R32f
        | PixelFormat::Xvuy2101010
        | PixelFormat::Xyyy2101010 => 4,
        // 48
        PixelFormat::Rgb161616 | PixelFormat::Bgr161616 | PixelFormat::Bgr161616f => 6,
        // 64
        PixelFormat::Xrgb16161616f
        | PixelFormat::Xbgr16161616f
        | PixelFormat::Argb16161616f
        | PixelFormat::Abgr16161616f
        | PixelFormat::Y210
        | PixelFormat::Y212
        | PixelFormat::Y216
        | PixelFormat::Y412
        | PixelFormat::Y416
        | PixelFormat::Xvyu1216161616
        | PixelFormat::Xvyu16161616
        | PixelFormat::Y0l0
        | PixelFormat::X0l0
        | PixelFormat::Y0l2
        | PixelFormat::X0l2
        | PixelFormat::Axbxgxrx106106106106
        | PixelFormat::Xrgb16161616
        | PixelFormat::Xbgr16161616
        | PixelFormat::Argb16161616
        | PixelFormat::Abgr16161616
        | PixelFormat::Gr3232f => 8,
        // 96
        PixelFormat::Bgr323232f => 12,
        // 128
        PixelFormat::Abgr32323232f => 16,
        // FIXME: unsure about these
        PixelFormat::Nv12
        | PixelFormat::Nv21
        | PixelFormat::Nv16
        | PixelFormat::Nv61
        | PixelFormat::Yuv410
        | PixelFormat::Yvu410
        | PixelFormat::Yuv411
        | PixelFormat::Yvu411
        | PixelFormat::Yuv420
        | PixelFormat::Yvu420
        | PixelFormat::Yuv422
        | PixelFormat::Yvu422
        | PixelFormat::Yuv444
        | PixelFormat::Yvu444
        | PixelFormat::Vuy101010
        | PixelFormat::Yuv4208bit
        | PixelFormat::Yuv42010bit
        | PixelFormat::Xrgb8888A8
        | PixelFormat::Xbgr8888A8
        | PixelFormat::Rgbx8888A8
        | PixelFormat::Bgrx8888A8
        | PixelFormat::Rgb888A8
        | PixelFormat::Bgr888A8
        | PixelFormat::Rgb565A8
        | PixelFormat::Bgr565A8
        | PixelFormat::Nv24
        | PixelFormat::Nv42
        | PixelFormat::P210
        | PixelFormat::P010
        | PixelFormat::P012
        | PixelFormat::P016
        | PixelFormat::Nv15
        | PixelFormat::Q410
        | PixelFormat::Q401
        | PixelFormat::P030
        | PixelFormat::Nv20
        | PixelFormat::Nv30
        | PixelFormat::S010
        | PixelFormat::S210
        | PixelFormat::S410
        | PixelFormat::S012
        | PixelFormat::S212
        | PixelFormat::S412
        | PixelFormat::S016
        | PixelFormat::S216
        | PixelFormat::S416
        | PixelFormat::P230
        | PixelFormat::T430 => return None,
        // TODO: these are fractional (multiple pixels per byte)
        PixelFormat::C1
        | PixelFormat::C2
        | PixelFormat::C4
        | PixelFormat::D1
        | PixelFormat::D2
        | PixelFormat::D4
        | PixelFormat::D8
        | PixelFormat::R1
        | PixelFormat::R2
        | PixelFormat::R4 => return None,
        // Unknown
        PixelFormat::Other(_) => return None,
    };

    Some(n)
}
