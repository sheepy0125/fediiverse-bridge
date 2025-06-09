//! Token scan screen

use crate::AppState;

use cabbage::{F_TOP_HEIGHT, F_TOP_WIDTH};
use citro2d::prelude::*;
use fruit::prelude::*;

/// Scan for a QR every n frames
pub const SCAN_FRAMES: usize = 3;

pub struct QrScan {
    pub text: Text<'static>,
    pub preview: Option<Image>,
    /// Only scan every SCAN_FRAMES frames
    pub scan_frame: usize,
}
impl Default for QrScan {
    fn default() -> Self {
        let text = Text::new(
            "Searching for token QR code",
            (F_TOP_WIDTH / 2., F_TOP_HEIGHT - 15.),
            TextDrawStyle::default()
                .with_alignment(HorizAlignment::Center, VertAlignment::Center)
                .with_color((0xff, 0xff, 0xff, 0xff)),
        );
        Self {
            scan_frame: 0,
            preview: None,
            text,
        }
    }
}

impl Lifecycle<AppState<'_>> for QrScan {
    fn update_state(
        &mut self,
        _handles: &cabbage::Handles,
        state: &mut AppState<'_>,
    ) -> anyhow::Result<()> {
        // Update scan
        state.camera.capture()?;
        state.camera.convert(&mut state.scanner.image);
        self.scan_frame += 1;
        if self.scan_frame > SCAN_FRAMES {
            if let Some(frame) = state.scanner.scan()? {
                state.token.replace(frame);
            };

            self.scan_frame = 0;
        }

        // Update preview texture
        drop(self.preview.take());
        let camera_buf_rgba8 = &*state.scanner.image;
        let preview = Image::new(
            BoundingBox::with_top_left((0., 0.), (F_TOP_WIDTH, F_TOP_HEIGHT)),
            bytemuck::cast_slice(camera_buf_rgba8),
        );
        self.preview.replace(preview);

        Ok(())
    }
}
impl Blit for QrScan {
    fn blit(&self) -> anyhow::Result<()> {
        if let Some(ref preview) = self.preview {
            preview.blit()?;
        }
        // Textbox
        Rectangle::new(
            BoundingBox::with_corners((0., F_TOP_HEIGHT - 30.), (F_TOP_WIDTH, F_TOP_HEIGHT)),
            RectangleDrawStyle::default().with_fill((0, 0, 0, 127)),
        )
        .blit()?;
        self.text.blit()?;
        Ok(())
    }
}
