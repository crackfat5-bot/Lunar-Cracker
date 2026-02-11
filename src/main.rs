#![windows_subsystem = "windows"]
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{Utc, Duration};
use rand::Rng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

#[derive(PartialEq, Clone, Copy)]
enum Language {
    English,
    Russian,
}

struct Translations {
    title: &'static str,
    subtitle: &'static str,
    nickname_label: &'static str,
    nickname_hint: &'static str,
    path_label: &'static str,
    path_hint: &'static str,
    crack_button: &'static str,
    processing: &'static str,
    info_title: &'static str,
    version: &'static str,
    features: &'static str,
    feature_token: &'static str,
    feature_token_desc: &'static str,
    feature_autoclose: &'static str,
    feature_custom_path: &'static str,
    feature_secure: &'static str,
    support: &'static str,
    telegram_button: &'static str,
    warning: &'static str,
    hint_chars: &'static str,
    success: &'static str,
    error_empty: &'static str,
    error_short: &'static str,
    error_long: &'static str,
    error_chars: &'static str,
    error_not_found: &'static str,
    processing_msg: &'static str,
    closed_msg: &'static str,
}

const EN: Translations = Translations {
    title: "LUNAR",
    subtitle: "CRACKER",
    nickname_label: "Nickname",
    nickname_hint: "Enter your nickname",
    path_label: "Lunar Client Path",
    path_hint: "Path to .lunarclient/settings/game",
    crack_button: "CRACK NOW",
    processing: "PROCESSING",
    info_title: "Information",
    version: "Version",
    features: "Features",
    feature_token: "• Random token lifetime",
    feature_token_desc: "  (10 min - 2 days)",
    feature_autoclose: "• Auto-close Lunar Client",
    feature_custom_path: "• Custom path support",
    feature_secure: "• Secure token generation",
    support: "Support",
    telegram_button: "Telegram Channel",
    warning: "⚠ Use at your own risk. This tool modifies Lunar Client configuration files.",
    hint_chars: "3-16 characters • A-Z, 0-9, _",
    success: "✓ Successfully cracked!",
    error_empty: "Nickname cannot be empty",
    error_short: "Nickname must be at least 3 characters",
    error_long: "Nickname cannot be longer than 16 characters",
    error_chars: "Only English letters, numbers and underscores",
    error_not_found: "accounts.json not found in the specified path.",
    processing_msg: "Processing...",
    closed_msg: "Lunar Client closed. Patching...",
};

const RU: Translations = Translations {
    title: "LUNAR",
    subtitle: "КРЯКЕР",
    nickname_label: "Никнейм",
    nickname_hint: "Введите ваш никнейм",
    path_label: "Путь к Lunar Client",
    path_hint: "Путь к .lunarclient/settings/game",
    crack_button: "КРЯКНУТЬ",
    processing: "ОБРАБОТКА",
    info_title: "Информация",
    version: "Версия",
    features: "Возможности",
    feature_token: "• Случайное время жизни токена",
    feature_token_desc: "  (10 мин - 2 дня)",
    feature_autoclose: "• Авто-закрытие Lunar Client",
    feature_custom_path: "• Поддержка своего пути",
    feature_secure: "• Безопасная генерация токена",
    support: "Поддержка",
    telegram_button: "Telegram Канал",
    warning: "⚠ Используйте на свой риск. Этот инструмент изменяет конфигурационные файлы Lunar Client.",
    hint_chars: "3-16 символов • A-Z, 0-9, _",
    success: "✓ Успешно взломано!",
    error_empty: "Никнейм не может быть пустым",
    error_short: "Никнейм должен быть минимум 3 символа",
    error_long: "Никнейм не может быть длиннее 16 символов",
    error_chars: "Только английские буквы, цифры и подчеркивание",
    error_not_found: "accounts.json не найден по указанному пути.",
    processing_msg: "Обработка...",
    closed_msg: "Lunar Client закрыт. Патчинг...",
};

#[derive(Serialize, Deserialize, Clone)]
struct MinecraftProfile {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Account {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "accessTokenExpiresAt")]
    access_token_expires_at: String,
    #[serde(rename = "eligibleForMigration")]
    eligible_for_migration: bool,
    #[serde(rename = "hasMultipleProfiles")]
    has_multiple_profiles: bool,
    legacy: bool,
    persistent: bool,
    #[serde(rename = "localId")]
    local_id: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "minecraftProfile")]
    minecraft_profile: MinecraftProfile,
    #[serde(rename = "remoteId")]
    remote_id: String,
    #[serde(rename = "type")]
    account_type: String,
    username: String,
    #[serde(rename = "userProperties")]
    user_properties: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct AccountsData {
    #[serde(rename = "activeAccountLocalId")]
    active_account_local_id: String,
    accounts: std::collections::HashMap<String, Account>,
}

struct LunarCrackedApp {
    nickname: String,
    status_message: String,
    is_processing: bool,
    nickname_error: String,
    show_info_panel: bool,
    custom_path: String,
    telegram_icon: Option<egui::TextureHandle>,
    us_flag: Option<egui::TextureHandle>,
    ru_flag: Option<egui::TextureHandle>,
    animation_progress: f32,
    info_panel_progress: f32,
    language: Language,
}

impl Default for LunarCrackedApp {
    fn default() -> Self {
        Self {
            nickname: String::new(),
            status_message: String::new(),
            is_processing: false,
            nickname_error: String::new(),
            show_info_panel: false,
            custom_path: Self::get_default_path(),
            telegram_icon: None,
            us_flag: None,
            ru_flag: None,
            animation_progress: 0.0,
            info_panel_progress: 0.0,
            language: Language::English,
        }
    }
}

impl LunarCrackedApp {
    fn get_translation(&self) -> &Translations {
        match self.language {
            Language::English => &EN,
            Language::Russian => &RU,
        }
    }

    fn get_default_path() -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(username) = std::env::var("USERNAME") {
                return format!("C:\\Users\\{}\\.lunarclient\\settings\\game", username);
            }
        }
        String::new()
    }

    fn validate_nickname(&self) -> Result<(), String> {
        let t = self.get_translation();
        
        if self.nickname.is_empty() {
            return Err(t.error_empty.to_string());
        }
        
        if self.nickname.len() < 3 {
            return Err(t.error_short.to_string());
        }
        
        if self.nickname.len() > 16 {
            return Err(t.error_long.to_string());
        }
        
        if !self.nickname.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(t.error_chars.to_string());
        }
        
        Ok(())
    }

    fn load_icons(&mut self, ctx: &egui::Context) {
        if self.telegram_icon.is_none() {
            let possible_paths = vec!["telegram_icon.png", "images.png"];
            
            for path in possible_paths {
                if let Ok(image_data) = fs::read(path) {
                    if let Ok(image) = image::load_from_memory(&image_data) {
                        let rgba_image = image.to_rgba8();
                        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                        let pixels: Vec<egui::Color32> = rgba_image
                            .pixels()
                            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                            .collect();
                        
                        let color_image = egui::ColorImage { size, pixels };
                        self.telegram_icon = Some(ctx.load_texture("telegram_icon", color_image, Default::default()));
                        break;
                    }
                }
            }
        }

        if self.us_flag.is_none() {
            let size = [24, 16];
            let mut pixels = vec![egui::Color32::from_rgb(255, 255, 255); size[0] * size[1]];
            
            for y in 0..size[1] {
                let stripe = y * 13 / size[1];
                let color = if stripe % 2 == 0 {
                    egui::Color32::from_rgb(178, 34, 52)
                } else {
                    egui::Color32::from_rgb(255, 255, 255)
                };
                
                for x in 0..size[0] {
                    pixels[y * size[0] + x] = color;
                }
            }
            
            for y in 0..(size[1] * 7 / 13) {
                for x in 0..(size[0] * 2 / 5) {
                    pixels[y * size[0] + x] = egui::Color32::from_rgb(60, 59, 110);
                }
            }
            
            let image = egui::ColorImage { size, pixels };
            self.us_flag = Some(ctx.load_texture("us_flag", image, Default::default()));
        }

        if self.ru_flag.is_none() {
            let size = [24, 16];
            let mut pixels = vec![egui::Color32::from_rgb(255, 255, 255); size[0] * size[1]];
            
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let color = if y < size[1] / 3 {
                        egui::Color32::from_rgb(255, 255, 255)
                    } else if y < size[1] * 2 / 3 {
                        egui::Color32::from_rgb(0, 57, 166)
                    } else {
                        egui::Color32::from_rgb(213, 43, 30)
                    };
                    
                    pixels[y * size[0] + x] = color;
                }
            }
            
            let image = egui::ColorImage { size, pixels };
            self.ru_flag = Some(ctx.load_texture("ru_flag", image, Default::default()));
        }
    }

    fn generate_random_token_with_expiry() -> (String, String) {
        let mut rng = rand::thread_rng();
        
        let min_minutes = 10;
        let max_minutes = 2 * 24 * 60;
        let random_minutes = rng.gen_range(min_minutes..=max_minutes);
        
        let now = Utc::now();
        let exp = now + Duration::minutes(random_minutes);
        
        let header = r#"{"kid":"049181","alg":"RS256"}"#;
        let header_b64 = URL_SAFE_NO_PAD.encode(header);
        
        let uuid1 = format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", 
            rng.gen::<u32>(), rng.gen::<u16>(), rng.gen::<u16>(), 
            rng.gen::<u16>(), rng.gen::<u64>() & 0xFFFFFFFFFFFF);
        let uuid2 = format!("{:032x}", rng.gen::<u128>());
        
        let payload = format!(
            r#"{{"xuid":"{}","agg":"Adulu","sub":"{}","auth":"XBOX","ns":"default","roles":[],"iss":"authentication","flags":["orders_2022","msamigration_stage4","twofactorauth","multiplaxUr"],"profiles":{{"mc":"{}"}}, "platform":"PC_LAUNCHER","pfd":[{{"type":"mc","id":"{}","name":"player"}}],"nbf":{},"exp":{},"iat":{},"aid":"00000000-0000-0000-0000-{:012x}"}}"#,
            rng.gen_range(1000000000000000u64..9999999999999999u64),
            uuid1,
            uuid2,
            uuid2,
            now.timestamp(),
            exp.timestamp(),
            now.timestamp(),
            rng.gen::<u64>() & 0xFFFFFFFFFFFF
        );
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload);
        
        let signature: String = (0..342)
            .map(|_| {
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                chars.chars().nth(rng.gen_range(0..chars.len())).unwrap()
            })
            .collect();
        
        let token = format!("{}.{}.{}", header_b64, payload_b64, signature);
        let exp_date = exp.to_rfc3339();
        
        (token, exp_date)
    }

    fn generate_random_refresh_token() -> String {
        let mut rng = rand::thread_rng();
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!*-_.";
        (0..380)
            .map(|_| chars.chars().nth(rng.gen_range(0..chars.len())).unwrap())
            .collect()
    }

    fn create_default_account(nickname: &str) -> (String, Account) {
        let mut rng = rand::thread_rng();
        let local_id = format!("{:032x}", rng.gen::<u128>());
        let mc_uuid = format!("{:032x}", rng.gen::<u128>());
        let remote_id = rng.gen_range(1000000000000000u64..9999999999999999u64).to_string();
        
        let (access_token, expires_at) = Self::generate_random_token_with_expiry();
        let refresh_token = Self::generate_random_refresh_token();
        
        let account = Account {
            access_token,
            access_token_expires_at: expires_at,
            eligible_for_migration: false,
            has_multiple_profiles: false,
            legacy: false,
            persistent: true,
            local_id: local_id.clone(),
            refresh_token,
            minecraft_profile: MinecraftProfile {
                id: mc_uuid.clone(),
                name: nickname.to_string(),
            },
            remote_id,
            account_type: "Xbox".to_string(),
            username: nickname.to_string(),
            user_properties: None,
        };
        
        (local_id, account)
    }

    fn close_lunar_client() -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            
            match Command::new("taskkill")
                .args(&["/F", "/IM", "Lunar Client.exe"])
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                Err(e) => Err(format!("Error executing taskkill: {}", e)),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(false)
        }
    }

    fn get_accounts_path(&self) -> Result<PathBuf, String> {
        let t = self.get_translation();
        let path = PathBuf::from(&self.custom_path).join("accounts.json");
        
        if !path.exists() {
            return Err(t.error_not_found.to_string());
        }
        
        Ok(path)
    }

    fn patch_accounts(&self) -> Result<(), String> {
        let path = self.get_accounts_path()?;
        
        let (local_id, account) = Self::create_default_account(&self.nickname);
        
        let mut accounts = std::collections::HashMap::new();
        accounts.insert(local_id.clone(), account);
        
        let data = AccountsData {
            active_account_local_id: local_id,
            accounts,
        };
        
        let new_content = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Error serializing JSON: {}", e))?;
        
        fs::write(&path, new_content)
            .map_err(|e| format!("Error writing file: {}", e))?;
        
        Ok(())
    }

    fn run_patch(&mut self) {
        if let Err(e) = self.validate_nickname() {
            self.nickname_error = e;
            return;
        }
        
        let processing_msg = self.get_translation().processing_msg.to_string();
        let closed_msg = self.get_translation().closed_msg.to_string();
        let success_msg = self.get_translation().success.to_string();
        
        self.nickname_error.clear();
        self.is_processing = true;
        self.status_message = processing_msg;
        
        match Self::close_lunar_client() {
            Ok(was_running) => {
                if was_running {
                    self.status_message = closed_msg;
                    std::thread::sleep(std::time::Duration::from_millis(800));
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.is_processing = false;
                return;
            }
        }
        
        match self.patch_accounts() {
            Ok(_) => {
                self.status_message = success_msg;
            }
            Err(e) => {
                self.status_message = format!("✕ Error: {}", e);
            }
        }
        
        self.is_processing = false;
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(&self.custom_path)
            .pick_folder()
        {
            if let Some(path_str) = path.to_str() {
                self.custom_path = path_str.to_string();
            }
        }
    }
}

impl eframe::App for LunarCrackedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.load_icons(ctx);
        
        if self.is_processing {
            self.animation_progress += 0.05;
            if self.animation_progress > 1.0 {
                self.animation_progress = 0.0;
            }
        }
        
        let target_panel = if self.show_info_panel { 1.0 } else { 0.0 };
        self.info_panel_progress += (target_panel - self.info_panel_progress) * 0.2;
        
        ctx.request_repaint();
        
        let language = self.language;
        let animation_progress = self.animation_progress;
        let info_panel_progress = self.info_panel_progress;
        let is_processing = self.is_processing;
        
        let tr = self.get_translation();
        let title = tr.title;
        let subtitle = tr.subtitle;
        let nickname_label = tr.nickname_label;
        let nickname_hint = tr.nickname_hint;
        let path_label = tr.path_label;
        let path_hint = tr.path_hint;
        let crack_button = tr.crack_button;
        let processing = tr.processing;
        let info_title = tr.info_title;
        let version = tr.version;
        let features = tr.features;
        let feature_token = tr.feature_token;
        let feature_token_desc = tr.feature_token_desc;
        let feature_autoclose = tr.feature_autoclose;
        let feature_custom_path = tr.feature_custom_path;
        let feature_secure = tr.feature_secure;
        let support = tr.support;
        let telegram_button = tr.telegram_button;
        let warning = tr.warning;
        let hint_chars = tr.hint_chars;
        
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals {
            dark_mode: true,
            override_text_color: Some(egui::Color32::from_rgb(230, 230, 240)),
            window_fill: egui::Color32::from_rgb(16, 16, 20),
            panel_fill: egui::Color32::from_rgb(16, 16, 20),
            extreme_bg_color: egui::Color32::from_rgb(10, 10, 14),
            ..egui::Visuals::dark()
        };
        
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(26, 26, 32);
        style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(200, 200, 220);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 32, 42);
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(210, 210, 230);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 90, 240);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 70, 200);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(100, 100, 255, 120);
        
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        
        ctx.set_style(style);

        egui::TopBottomPanel::top("language_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(16, 16, 20)).inner_margin(egui::Margin {
                left: 15.0,
                right: 15.0,
                top: 12.0,
                bottom: 12.0,
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(flag) = &self.us_flag {
                        let img = egui::Image::new(flag).max_size(egui::vec2(24.0, 16.0));
                        let button = egui::ImageButton::new(img)
                            .frame(language == Language::English);
                        
                        if ui.add(button).clicked() {
                            self.language = Language::English;
                        }
                    }
                    
                    ui.add_space(5.0);
                    
                    if let Some(flag) = &self.ru_flag {
                        let img = egui::Image::new(flag).max_size(egui::vec2(24.0, 16.0));
                        let button = egui::ImageButton::new(img)
                            .frame(language == Language::Russian);
                        
                        if ui.add(button).clicked() {
                            self.language = Language::Russian;
                        }
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(16, 16, 20)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    
                    let wave_offset = (animation_progress * std::f32::consts::PI * 2.0).sin() * 3.0;
                    ui.label(egui::RichText::new(title)
                        .size(48.0)
                        .color(egui::Color32::from_rgb(
                            100, 
                            100, 
                            (255.0 - wave_offset * 10.0) as u8
                        ))
                        .strong());
                    
                    ui.add_space(2.0);
                    
                    ui.label(egui::RichText::new(subtitle)
                        .size(28.0)
                        .color(egui::Color32::from_rgb(160, 160, 200)));
                    
                    ui.add_space(30.0);
                });
                
                ui.horizontal(|ui| {
                    ui.add_space(50.0);
                    
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(nickname_label)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 180)));
                        
                        ui.add_space(8.0);
                        
                        let text_edit = egui::TextEdit::singleline(&mut self.nickname)
                            .hint_text(nickname_hint)
                            .desired_width(550.0)
                            .font(egui::TextStyle::Body);
                        
                        let response = ui.add(text_edit);
                        
                        if response.changed() {
                            self.nickname_error.clear();
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                if !self.nickname_error.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(50.0);
                        ui.label(
                            egui::RichText::new(format!("✕ {}", &self.nickname_error))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(255, 100, 120))
                        );
                    });
                }
                
                ui.add_space(18.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(50.0);
                    
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(path_label)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 180)));
                        
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            let folder_button = egui::Button::new(
                                egui::RichText::new("📁")
                                    .size(18.0)
                            )
                            .fill(egui::Color32::from_rgb(40, 40, 52))
                            .min_size(egui::vec2(44.0, 36.0))
                            .rounding(8.0);
                            
                            if ui.add(folder_button).clicked() {
                                self.open_folder_dialog();
                            }
                            
                            let path_edit = egui::TextEdit::singleline(&mut self.custom_path)
                                .hint_text(path_hint)
                                .desired_width(506.0)
                                .font(egui::TextStyle::Monospace);
                            
                            ui.add(path_edit);
                        });
                    });
                });
                
                ui.add_space(22.0);
                
                ui.vertical_centered(|ui| {
                    let button_enabled = !self.nickname.is_empty() && !is_processing;
                    
                    let pulse = if is_processing {
                        (animation_progress * std::f32::consts::PI * 4.0).sin() * 20.0
                    } else {
                        0.0
                    };
                    
                    let button_color = if button_enabled {
                        egui::Color32::from_rgb(
                            (100.0 + pulse) as u8, 
                            (100.0 + pulse) as u8, 
                            (255.0 - pulse.abs()) as u8
                        )
                    } else {
                        egui::Color32::from_rgb(50, 50, 60)
                    };
                    
                    let button_text = if is_processing { 
                        format!("{}{}", processing, ".".repeat(((animation_progress * 3.0) as usize) + 1))
                    } else { 
                        crack_button.to_string()
                    };
                    
                    let button = egui::Button::new(
                        egui::RichText::new(&button_text)
                            .size(16.0)
                            .color(egui::Color32::WHITE)
                            .strong()
                    )
                    .fill(button_color)
                    .min_size(egui::vec2(240.0, 52.0))
                    .rounding(8.0);
                    
                    if ui.add_enabled(button_enabled, button).clicked() {
                        self.run_patch();
                    }
                });
                
                ui.add_space(22.0);
                
                if !self.status_message.is_empty() {
                    ui.vertical_centered(|ui| {
                        let color = if self.status_message.contains("✓") || self.status_message.contains("Successfully") || self.status_message.contains("Успешно") {
                            egui::Color32::from_rgb(100, 255, 150)
                        } else if self.status_message.contains("Error") || self.status_message.contains("✕") {
                            egui::Color32::from_rgb(255, 100, 120)
                        } else {
                            egui::Color32::from_rgb(180, 180, 200)
                        };
                        
                        ui.label(
                            egui::RichText::new(&self.status_message)
                                .size(14.0)
                                .color(color)
                        );
                    });
                }
                
                ui.add_space(12.0);
                
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(hint_chars)
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 120, 140))
                    );
                });
            });

        egui::Window::new("info_button")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .frame(egui::Frame::none())
            .fixed_pos(egui::pos2(ctx.screen_rect().width() - 75.0, ctx.screen_rect().height() - 75.0))
            .show(ctx, |ui| {
                let glow_intensity = (ctx.input(|i| i.time) as f32 * 2.0).sin() * 0.5 + 0.5;
                let glow_color = egui::Color32::from_rgb(
                    (100.0 + glow_intensity * 50.0) as u8,
                    (100.0 + glow_intensity * 50.0) as u8,
                    (255.0 - glow_intensity * 50.0) as u8
                );
                
                let info_button = egui::Button::new(
                    egui::RichText::new("ⓘ")
                        .size(26.0)
                        .color(glow_color)
                )
                .fill(egui::Color32::from_rgb(30, 30, 40))
                .min_size(egui::vec2(52.0, 52.0))
                .rounding(26.0);
                
                if ui.add(info_button).clicked() {
                    self.show_info_panel = !self.show_info_panel;
                }
            });

        if info_panel_progress > 0.01 {
            let panel_width = 330.0 * info_panel_progress;
            
            egui::SidePanel::right("info_panel")
                .exact_width(panel_width)
                .resizable(false)
                .frame(egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 20, 26))
                    .inner_margin(egui::Margin::same(24.0))
                )
                .show(ctx, |ui| {
                    ui.set_opacity(info_panel_progress);
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(info_title)
                                .size(20.0)
                                .color(egui::Color32::from_rgb(200, 200, 220))
                                .strong());
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_button = egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(16.0)
                                        .color(egui::Color32::from_rgb(180, 180, 200))
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false);
                                
                                if ui.add(close_button).clicked() {
                                    self.show_info_panel = false;
                                }
                            });
                        });
                        
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);
                        
                        ui.label(egui::RichText::new(version)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(140, 140, 160)));
                        ui.label(egui::RichText::new("1.0.0")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(200, 200, 220)));
                        
                        ui.add_space(20.0);
                        
                        ui.label(egui::RichText::new(features)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(140, 140, 160)));
                        ui.add_space(8.0);
                        
                        ui.label(egui::RichText::new(feature_token)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 180, 200)));
                        ui.label(egui::RichText::new(feature_token_desc)
                            .size(12.0)
                            .color(egui::Color32::from_rgb(140, 140, 160)));
                        
                        ui.add_space(8.0);
                        
                        ui.label(egui::RichText::new(feature_autoclose)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 180, 200)));
                        
                        ui.add_space(8.0);
                        
                        ui.label(egui::RichText::new(feature_custom_path)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 180, 200)));
                        
                        ui.add_space(8.0);
                        
                        ui.label(egui::RichText::new(feature_secure)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 180, 200)));
                        
                        ui.add_space(25.0);
                        ui.separator();
                        ui.add_space(30.0);
                        
                        ui.horizontal(|ui| {
                            if let Some(icon) = &self.telegram_icon {
                                ui.add(egui::Image::new(icon).max_width(28.0));
                                ui.add_space(8.0);
                            }
                            
                            let telegram_btn = egui::Button::new(
                                egui::RichText::new(telegram_button)
                                    .size(14.0)
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            )
                            .fill(egui::Color32::from_rgb(50, 150, 255))
                            .min_size(egui::vec2(260.0, 44.0))
                            .rounding(8.0);
                            
                            if ui.add(telegram_btn).clicked() {
                                let _ = open::that("https://t.me/tgkcalifornialove");
                            }
                        });
                        
                        ui.add_space(20.0);
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(warning)
                                .size(10.0)
                                .color(egui::Color32::from_rgb(255, 180, 100))
                        ).wrap());
                    });
                });
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([650.0, 580.0])
            .with_resizable(false)
            .with_title("Lunar Cracker"),
        ..Default::default()
    };
    
    if let Err(e) = eframe::run_native(
        "Lunar Cracker",
        options,
        Box::new(|_cc| Ok(Box::new(LunarCrackedApp::default()))),
    ) {
        eprintln!("Error running application: {}", e);
        std::process::exit(1);
    }
}