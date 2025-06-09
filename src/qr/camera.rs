use ctru::services::cam::{Cam, Camera, FrameRate, OutputFormat, Trimming, ViewSize, WhiteBalance};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(5);

pub struct CameraState<'a> {
    camera: &'a mut dyn Camera,
    rgb565_camera_buf: Vec<u8>,
}
impl<'a> CameraState<'a> {
    pub fn new(cam: &'a mut Cam) -> anyhow::Result<Self> {
        // N.B. `outer_left_cam` and `outer_right_cam` by themselves won't work --
        // the capture timeout is reached -- only using `both_outer_cams` works.
        let camera = &mut cam.both_outer_cams;
        camera.set_view_size(ViewSize::TopLCD)?;
        camera.set_output_format(OutputFormat::Rgb565)?;
        camera.set_noise_filter(true)?;
        camera.set_auto_exposure(true)?;
        camera.set_white_balance(WhiteBalance::Auto)?;
        camera.set_trimming(Trimming::new_centered_with_view(ViewSize::TopLCD))?;
        camera.set_frame_rate(FrameRate::Fps30)?;
        let rgb565_camera_buf = Vec::<u8>::with_capacity(camera.final_byte_length());

        let s = Self {
            camera,
            rgb565_camera_buf,
        };
        Ok(s)
    }

    pub fn capture(&mut self) -> anyhow::Result<()> {
        self.rgb565_camera_buf
            .resize(self.camera.final_byte_length(), 0);
        self.camera
            .take_picture(&mut self.rgb565_camera_buf, TIMEOUT)?;
        Ok(())
    }

    /// Convert the RGB565 buffer into an RGBA8 buffer
    pub fn convert(&mut self, rgba8_camera_buf: &mut [u8]) {
        let mut raw_buf_idx = 0;
        let mut converted_buf_idx = 0;

        while raw_buf_idx < self.rgb565_camera_buf.len()
            && converted_buf_idx < rgba8_camera_buf.len()
        {
            let source = &mut self.rgb565_camera_buf[raw_buf_idx..raw_buf_idx + 2]; // 16-bit
            let dest = &mut rgba8_camera_buf[converted_buf_idx..converted_buf_idx + 4]; // 32-bit

            // N.B. little endian, 0x1F == (1 << 5) - 1 and 0x3F == (1 << 6) - 1
            let rgb565 = (source[1] as u16) << 8 | (source[0] as u16);
            let r5 = (rgb565 >> 11) & 0x1F;
            let g6 = (rgb565 >> 5) & 0x3F;
            let b5 = rgb565 & 0x1F;

            let r8 = ((r5 * 255) / 0x1F) as u8;
            let g8 = ((g6 * 255) / 0x3F) as u8;
            let b8 = ((b5 * 255) / 0x1F) as u8;

            dest.copy_from_slice(&[r8, g8, b8, 0xff]);

            raw_buf_idx += 2; // 16-bit
            converted_buf_idx += 4; // 32-bit
        }
    }
}
