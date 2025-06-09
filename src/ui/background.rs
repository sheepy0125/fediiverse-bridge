use crate::AppState;

use cabbage::{F_TOP_HEIGHT, F_TOP_WIDTH};
use citro2d::prelude::*;
use fruit::prelude::*;

#[derive(Default)]
pub struct Background {
    //
}

impl Lifecycle<AppState> for Background {}
impl Blit for Background {
    fn blit(&self) -> anyhow::Result<()> {
        let style = RectangleDrawStyle::default().with_fill((0xfb, 0xdb, 0x65, 0xff));
        Rectangle::new(
            BoundingBox::with_top_left((0., 0.), (F_TOP_WIDTH, F_TOP_HEIGHT)),
            style,
        )
        .blit()?;
        Ok(())
    }
}
