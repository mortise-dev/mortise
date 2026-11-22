//! The two bundled families: Asap for the interface, Inconsolata for code.
//!
//! Both are under the SIL Open Font License 1.1; each folder in `assets/fonts`
//! holds its `OFL.txt`. The files are the static weights of the specification,
//! section 11, unchanged.

use std::sync::Arc;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

use crate::theme::Face;

fn bytes(face: Face) -> &'static [u8] {
    match face {
        Face::Sans400 => include_bytes!("../../../assets/fonts/asap/Asap-Regular.ttf"),
        Face::Sans500 => include_bytes!("../../../assets/fonts/asap/Asap-Medium.ttf"),
        Face::Sans600 => include_bytes!("../../../assets/fonts/asap/Asap-SemiBold.ttf"),
        Face::Sans700 => include_bytes!("../../../assets/fonts/asap/Asap-Bold.ttf"),
        Face::Mono400 => {
            include_bytes!("../../../assets/fonts/inconsolata/Inconsolata-Regular.ttf")
        }
        Face::Mono500 => include_bytes!("../../../assets/fonts/inconsolata/Inconsolata-Medium.ttf"),
        Face::Mono600 => {
            include_bytes!("../../../assets/fonts/inconsolata/Inconsolata-SemiBold.ttf")
        }
    }
}

/// Registers every face as its own family, and makes Asap and Inconsolata the
/// defaults for anything egui draws by itself. No other font ships with Mortise.
pub fn install(ctx: &Context) {
    let mut fonts = FontDefinitions::empty();
    for face in Face::ALL {
        let name = face.family_name().to_owned();
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_static(bytes(face))));
        fonts.families.insert(face.family(), vec![name]);
    }
    fonts.families.insert(
        FontFamily::Proportional,
        vec![Face::Sans400.family_name().to_owned()],
    );
    fonts.families.insert(
        FontFamily::Monospace,
        vec![Face::Mono400.family_name().to_owned()],
    );
    ctx.set_fonts(fonts);
}
