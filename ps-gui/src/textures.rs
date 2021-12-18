//! Image handling in egui is not particularly nice. We can only load them
//! when we have access to the `Frame` - so not deeper in the ui.
//! And then, loaded images can only be referenced by the texture id, not their
//! name. So loading images in `setup` and accessing them later only works if
//! the texture-id is accessible later on, so we need to forward the info.
//! This is complicated by the fact that the image in question is only
//! necessary on macOS and I'd rather not load it on Windows or Linux systems.

use eframe::{self, egui, epi};

#[cfg(target_os = "macos")]
use image;

/// Pre-loaded textures
pub struct Textures {
    #[cfg(target_os = "macos")]
    pub missing_permissions_image: (eframe::egui::Vec2, eframe::egui::TextureId),
}

impl Textures {
    #[allow(unused)]
    pub fn populated(frame: &mut epi::Frame<'_>) -> Textures {
        #[cfg(target_os = "macos")]
        {
            let missing_permissions_image = install_missing_permission_image(
                include_bytes!("resources/add_permissions.png"),
                frame,
            );
            Textures {
                missing_permissions_image,
            }
        }

        #[cfg(not(target_os = "macos"))]
        Textures {}
    }
}

/// Load the permission image
// via: https://github.com/emilk/egui/blob/master/eframe/examples/image.rs
#[allow(unused)]
#[cfg(target_os = "macos")]
fn install_missing_permission_image(
    image_data: &[u8],
    frame: &mut epi::Frame<'_>,
) -> (egui::Vec2, egui::TextureId) {
    use image::GenericImageView;
    let image = image::load_from_memory(image_data).expect("Failed to load image");
    let image_buffer = image.to_rgba8();
    let size = (image.width() as usize, image.height() as usize);
    let pixels = image_buffer.into_vec();
    assert_eq!(size.0 * size.1 * 4, pixels.len());
    let pixels: Vec<_> = pixels
        .chunks_exact(4)
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    // Allocate a texture:
    let texture = frame
        .tex_allocator()
        .alloc_srgba_premultiplied(size, &pixels);
    let size = egui::Vec2::new(size.0 as f32, size.1 as f32);
    (size, texture)
}
