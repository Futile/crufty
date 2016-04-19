use util::TextureInfo;
use components::SpriteInfo;

#[derive(Clone, Debug)]
pub struct Animation {
    pub start_info: TextureInfo,
    pub num_frames: u8,
    pub frame_durations: Vec<f32>, // in seconds
    pub width: f32,
    pub height: f32,
}

impl Animation {
    pub fn create_sprite_info(&self, frame_idx: u8) -> SpriteInfo {
        let mut new_tex_info = self.start_info.clone();
        new_tex_info.idx += frame_idx as f32;

        SpriteInfo {
            width: self.width,
            height: self.height,
            texture_info: new_tex_info,
        }
    }
}