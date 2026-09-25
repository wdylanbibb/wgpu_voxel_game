use crate::world::block::BlockId;

pub fn show_selected_block_preview(
    context: &egui::Context,
    texture_id: egui::TextureId,
    block: BlockId,
    size: f32,
) {
    egui::Area::new(egui::Id::new("selected_block_preview"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0))
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(192))
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::ZERO)
                    .show(ui, |ui| {
                        ui.image((texture_id, egui::vec2(size, size)));
                    });
                ui.label(format!("{block:?}"));
            });
        });
}
