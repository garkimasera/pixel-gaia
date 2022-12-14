mod edit_planet;
mod main_menu;
mod orbit;
mod star_system;
mod stat;

use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    math::Rect,
    prelude::*,
};
use bevy_egui::{
    egui::{self, FontData, FontDefinitions, FontFamily, RichText, Ui},
    EguiContext, EguiPlugin, EguiSettings,
};
use std::collections::{HashMap, VecDeque};
use strum::IntoEnumIterator;

use crate::{
    assets::{UiAssets, UiTexture, UiTextures},
    conf::Conf,
    draw::UpdateMap,
    gz::GunzipBin,
    msg::MsgKind,
    overlay::OverlayLayerKind,
    planet::*,
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::ManagePlanet,
    text::Unit,
    GameSpeed, GameState,
};

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin {
    pub edit_planet: bool,
}

#[derive(Clone, Default, Debug, Resource)]
pub struct WindowsOpenState {
    pub build: bool,
    pub orbit: bool,
    pub star_system: bool,
    pub layers: bool,
    pub stat: bool,
    pub message: bool,
    pub game_menu: bool,
    pub edit_planet: bool,
}

#[derive(Clone, Default, Resource)]
pub struct EguiTextures(HashMap<UiTexture, (egui::TextureHandle, egui::Vec2)>);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(WindowsOpenState {
                edit_planet: self.edit_planet,
                message: true,
                ..default()
            })
            .init_resource::<OverlayLayerKind>()
            .add_system_set(
                SystemSet::on_exit(GameState::AssetLoading)
                    .with_system(setup_fonts)
                    .with_system(load_textures),
            )
            .add_system_set(
                SystemSet::on_update(GameState::MainMenu).with_system(main_menu::main_menu),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(panels.label("ui_panels").before("ui_windows"))
                    .with_system(build_window.label("ui_windows"))
                    .with_system(orbit::orbit_window.label("ui_windows"))
                    .with_system(star_system::star_system_window.label("ui_windows"))
                    .with_system(layers_window.label("ui_windows"))
                    .with_system(stat::stat_window.label("ui_windows"))
                    .with_system(msg_window.label("ui_windows"))
                    .with_system(game_menu_window.label("ui_windows"))
                    .with_system(edit_planet::edit_planet_window.label("ui_windows")),
            )
            .add_system(exit_on_esc);
    }
}

fn setup_fonts(
    mut egui_ctx: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    conf: Res<Assets<Conf>>,
    ui_assets: Res<UiAssets>,
    gunzip_bin: Res<Assets<GunzipBin>>,
) {
    let conf = conf.get(&ui_assets.default_conf).unwrap().clone();
    egui_settings.scale_factor = conf.scale_factor.into();

    let font_data = gunzip_bin.get(&ui_assets.font).unwrap().clone();
    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_owned(font_data.0);
    font_data.tweak.scale = conf.font_scale;
    fonts.font_data.insert("m+_font".to_owned(), font_data);
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "m+_font".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("m+_font".to_owned());
    egui_ctx.ctx_mut().set_fonts(fonts);
}

fn exit_on_esc(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ButtonState::Pressed && key_code == KeyCode::Escape {
                app_exit_events.send(bevy::app::AppExit);
            }
        }
    }
}

fn load_textures(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    images: Res<Assets<Image>>,
    ui_textures: Res<UiTextures>,
) {
    let ctx = egui_ctx.ctx_mut();

    let mut egui_textures = HashMap::new();

    for (k, handle) in ui_textures.textures.iter() {
        let image = images.get(handle).unwrap();
        let size = egui::Vec2::new(image.size().x, image.size().y);
        let color_image = egui::ColorImage {
            size: [size.x as usize, size.y as usize],
            pixels: image
                .data
                .windows(4)
                .step_by(4)
                .map(|rgba| {
                    egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
                })
                .collect(),
        };
        let texture_handle =
            ctx.load_texture(k.as_ref(), color_image, egui::TextureOptions::NEAREST);

        egui_textures.insert(*k, (texture_handle, size));
    }

    commands.insert_resource(EguiTextures(egui_textures));
}

fn panels(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    planet: Res<Planet>,
    textures: Res<EguiTextures>,
    conf: Res<Conf>,
) {
    occupied_screen_space.window_rects.clear();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            sidebar(ui, &cursor_mode, &planet, hover_tile.get_single().unwrap());
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(ui, &mut cursor_mode, &mut wos, &mut speed, &textures, &conf);
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.scale_factor;
}

fn sidebar(ui: &mut egui::Ui, cursor_mode: &CursorMode, planet: &Planet, hover_tile: &HoverTile) {
    let mut stock: Vec<_> = planet.res.stock.iter().collect();
    stock.sort_by_key(|&(res, _)| res);
    for (kind, v) in stock.into_iter() {
        ui.horizontal(|ui| {
            ui.label(&format!(
                "{}: {}",
                t!(kind.as_ref()),
                kind.display_with_value(*v)
            ));
            let diff = planet.res.diff[kind];
            let sign = if diff > 0.0 { '+' } else { '-' };
            ui.label(
                egui::RichText::new(format!("({}{})", sign, kind.display_with_value(diff.abs())))
                    .small(),
            );
        });
    }

    ui.separator();

    // Information about selected tool
    ui.label(t!("selected-tool"));
    match cursor_mode {
        CursorMode::Normal => {
            ui.label(t!("none"));
        }
        CursorMode::Build(kind) => {
            ui.label(t!(kind.as_ref()));
        }
        CursorMode::Demolition => {
            ui.label(t!("demolition"));
        }
        CursorMode::EditBiome(biome) => {
            ui.label(format!("biome editing: {}", biome.as_ref()));
        }
    }

    ui.separator();

    // Information about the hovered tile
    if let Some(p) = hover_tile.0 {
        ui.label(format!("{}: [{}, {}]", t!("coordinates"), p.0, p.1));
        let tile = &planet.map[p];

        let (longitude, latitude) = planet.calc_longitude_latitude(p);
        ui.label(format!(
            "{}: {:.0}??, {}: {:.0}??",
            t!("longitude"),
            longitude * 180.0 * std::f32::consts::FRAC_1_PI,
            t!("latitude"),
            latitude * 180.0 * std::f32::consts::FRAC_1_PI,
        ));

        ui.label(format!(
            "{}: {:.1} ??C",
            t!("air-temprature"),
            tile.temp - 273.15
        ));

        let s = match &tile.structure {
            Structure::None => None,
            Structure::Occupied { by } => {
                Some(crate::info::structure_info(&planet.map[*by].structure))
            }
            other => Some(crate::info::structure_info(other)),
        };

        if let Some(s) = s {
            ui.label(s);
        }
    } else {
        ui.label(format!("{}: -", t!("coordinates")));
    };
}

fn toolbar(
    ui: &mut egui::Ui,
    _cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    speed: &mut GameSpeed,
    textures: &EguiTextures,
    conf: &Conf,
) {
    let (handle, size) = textures.0.get(&UiTexture::IconBuild).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("build"))
        .clicked()
    {
        wos.build = !wos.build;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconOrbit).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("orbit"))
        .clicked()
    {
        wos.orbit = !wos.orbit;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconStarSystem).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("star-system"))
        .clicked()
    {
        wos.star_system = !wos.star_system;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconLayers).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("layers"))
        .clicked()
    {
        wos.layers = !wos.layers;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconStat).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("statistics"))
        .clicked()
    {
        wos.stat = !wos.stat;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let texture = if *speed == GameSpeed::Paused {
        UiTexture::IconSpeedPausedSelected
    } else {
        UiTexture::IconSpeedPaused
    };
    let (handle, size) = textures.0.get(&texture).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("speed-paused"))
        .clicked()
    {
        *speed = GameSpeed::Paused;
    }
    let texture = if *speed == GameSpeed::Normal {
        UiTexture::IconSpeedNormalSelected
    } else {
        UiTexture::IconSpeedNormal
    };
    let (handle, size) = textures.0.get(&texture).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("speed-normal"))
        .clicked()
    {
        *speed = GameSpeed::Normal;
    }
    let texture = if *speed == GameSpeed::Fast {
        UiTexture::IconSpeedFastSelected
    } else {
        UiTexture::IconSpeedFast
    };
    let (handle, size) = textures.0.get(&texture).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("speed-fast"))
        .clicked()
    {
        *speed = GameSpeed::Fast;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let (handle, size) = textures.0.get(&UiTexture::IconMessage).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("messages"))
        .clicked()
    {
        wos.message = !wos.message;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconGameMenu).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("menu"))
        .clicked()
    {
        wos.game_menu = !wos.game_menu;
    }
}

fn build_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut cursor_mode: ResMut<CursorMode>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    params: Res<Params>,
) {
    if !wos.build {
        return;
    }

    let rect = egui::Window::new(t!("build"))
        .open(&mut wos.build)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            if ui.button(t!("demolition")).clicked() {
                *cursor_mode = CursorMode::Demolition;
            }
            ui.separator();
            for kind in &planet.player.buildable_structures {
                let s: &str = kind.as_ref();
                if ui
                    .button(t!(s))
                    .on_hover_ui(build_button_tooltip(*kind, &params))
                    .clicked()
                {
                    *cursor_mode = CursorMode::Build(*kind);
                }
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn build_button_tooltip(kind: StructureKind, params: &Params) -> impl FnOnce(&mut Ui) + '_ {
    building_desc_tooltip(&params.structures[&kind].building)
}

fn building_desc_tooltip(attrs: &BuildingAttrs) -> impl FnOnce(&mut Ui) + '_ {
    move |ui| {
        if !attrs.cost.is_empty() {
            ui.label(RichText::new(t!("cost")).strong());
            let mut resources = attrs.cost.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .into_iter()
                .map(|(resource, value)| {
                    format!(
                        "{}: {}",
                        t!(resource.as_ref()),
                        resource.display_with_value(*value)
                    )
                })
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
        if !attrs.upkeep.is_empty() {
            ui.label(RichText::new(t!("upkeep")).strong());
            let mut resources = attrs.upkeep.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .iter()
                .map(|(resource, value)| {
                    format!(
                        "{}: {}",
                        t!(resource.as_ref()),
                        resource.display_with_value(**value)
                    )
                })
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
        if !attrs.produces.is_empty() {
            ui.label(RichText::new(t!("produces")).strong());
            let mut resources = attrs.produces.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .iter()
                .map(|(resource, value)| {
                    format!(
                        "{}: {}",
                        t!(resource.as_ref()),
                        resource.display_with_value(**value)
                    )
                })
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
    }
}

fn layers_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut current_layer: ResMut<OverlayLayerKind>,
    mut update_map: ResMut<UpdateMap>,
    conf: Res<Conf>,
) {
    if !wos.layers {
        return;
    }
    let mut new_layer = *current_layer;

    let rect = egui::Window::new(t!("layers"))
        .open(&mut wos.layers)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            for kind in OverlayLayerKind::iter() {
                ui.radio_value(&mut new_layer, kind, t!(kind.as_ref()));
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));

    if new_layer != *current_layer {
        *current_layer = new_layer;
        update_map.update();
    }
}

fn msg_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut msgs: Local<VecDeque<(MsgKind, String)>>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
) {
    if !wos.message {
        return;
    }

    while let Some(msg) = crate::msg::pop_msg() {
        msgs.push_front(msg);
        if msgs.len() > conf.max_message {
            msgs.pop_back();
        }
    }

    let rect = egui::Window::new(t!("messages"))
        .open(&mut wos.message)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    for (_kind, msg) in msgs.iter() {
                        ui.label(msg);
                        ui.separator();
                    }
                });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn game_menu_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut app_exit_events: EventWriter<AppExit>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    conf: Res<Conf>,
) {
    if !wos.game_menu {
        return;
    }

    let mut close = false;

    let rect = egui::Window::new(t!("menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .open(&mut wos.game_menu)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button(t!("save")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Save("test.planet".into()));
                    close = true;
                }

                if ui.button(t!("load")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Load("test.planet".into()));
                    close = true;
                }
                ui.separator();
                if ui.button(t!("exit")).clicked() {
                    app_exit_events.send(bevy::app::AppExit);
                }
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));

    if close {
        wos.game_menu = false;
    }
}

fn convert_rect(rect: bevy_egui::egui::Rect, scale_factor: f32) -> Rect {
    Rect {
        min: Vec2::new(rect.left() * scale_factor, rect.top() * scale_factor),
        max: Vec2::new(rect.right() * scale_factor, rect.bottom() * scale_factor),
    }
}

impl Conf {
    fn tex_size(&self, size: egui::Vec2) -> egui::Vec2 {
        let factor = 1.0;
        egui::Vec2::new(size.x * factor, size.y * factor)
    }
}
