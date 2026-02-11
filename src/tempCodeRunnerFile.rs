// Cargo.toml остается тот же

// src/main.rs
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use sysinfo::{ProcessExt, System, SystemExt};
use chrono::{Utc, Duration};
use rand::Rng;

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

struct LunarPatcherApp {
    nickname: String,
    status_message: String,
    is_processing: bool,
}

impl Default for LunarPatcherApp {
    fn default() -> Self {
        Self {
            nickname: String::new(),
            status_message: String::new(),
            is_processing: false,
        }
    }
}

impl LunarPatcherApp {
    fn generate_random_token_with_expiry() -> (String, String) {
        let mut rng = rand::thread_rng();
        
        // Генерируем случайное время от 10 минут до 2 дней
        let min_minutes = 10;
        let max_minutes = 2 * 24 * 60; // 2 дня в минутах
        let random_minutes = rng.gen_range(min_minutes..=max_minutes);
        
        let now = Utc::now();
        let exp = now + Duration::minutes(random_minutes);
        
        // JWT Header
        let header = r#"{"kid":"049181","alg":"RS256"}"#;
        let header_b64 = base64::encode_config(header, base64::URL_SAFE_NO_PAD);
        
        // JWT Payload with random dates
        let payload = format!(
            r#"{{"xuid":"{}","agg":"Adulu","sub":"bbaab714-24a2-4a23-b4ac-b10cd424{}","auth":"XBOX","ns":"default","roles":[],"iss":"authentication","flags":["orders_2022","msamigration_stage4","twofactorauth","multiplaxUr"],"profiles":{{"mc":"{}"}}, "platform":"PC_LAUNCHER","pfd":[{{"type":"mc","id":"{}","name":"player"}}],"nbf":{},"exp":{},"iat":{},"aid":"00000000-0000-0000-0000-00004{}"}}"#,
            rng.gen_range(1000000000000000u64..9999999999999999u64),
            rng.gen_range(1000..9999),
            format!("{:032x}", rng.gen::<u128>()),
            format!("{:032x}", rng.gen::<u128>()),
            now.timestamp(),
            exp.timestamp(),
            now.timestamp(),
            rng.gen_range(100000..999999)
        );
        let payload_b64 = base64::encode_config(payload, base64::URL_SAFE_NO_PAD);
        
        // Random signature
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

    fn close_lunar_client() -> Result<bool, String> {
        let mut system = System::new_all();
        system.refresh_all();
        
        let mut found = false;
        for (pid, process) in system.processes() {
            let process_name = process.name().to_lowercase();
            if process_name.contains("lunarclient") || process_name.contains("lunar") {
                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/PID", &pid.to_string()])
                        .output();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    process.kill();
                }
                found = true;
            }
        }
        
        Ok(found)
    }

    fn get_accounts_path() -> Result<PathBuf, String> {
        let username = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .map_err(|_| "Не удалось получить имя пользователя".to_string())?;
        
        let path = PathBuf::from(format!(
            "C:\\Users\\{}\\.lunarclient\\settings\\game\\accounts.json",
            username
        ));
        
        if !path.exists() {
            return Err("Файл accounts.json не найден".to_string());
        }
        
        Ok(path)
    }

    fn patch_accounts(&self) -> Result<(), String> {
        let path = Self::get_accounts_path()?;
        
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Ошибка чтения файла: {}", e))?;
        
        let mut data: AccountsData = serde_json::from_str(&content)
            .map_err(|e| format!("Ошибка парсинга JSON: {}", e))?;
        
        // Generate new token with random expiration (10 min to 2 days)
        let (new_token, exp_date) = Self::generate_random_token_with_expiry();
        
        // Update all accounts
        for (_, account) in data.accounts.iter_mut() {
            account.access_token = new_token.clone();
            account.access_token_expires_at = exp_date.clone();
            account.minecraft_profile.name = self.nickname.clone();
            account.username = self.nickname.clone();
        }
        
        let new_content = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Ошибка сериализации JSON: {}", e))?;
        
        fs::write(&path, new_content)
            .map_err(|e| format!("Ошибка записи файла: {}", e))?;
        
        Ok(())
    }

    fn run_patch(&mut self) {
        self.is_processing = true;
        self.status_message = "Обработка...".to_string();
        
        // Close Lunar Client if running
        match Self::close_lunar_client() {
            Ok(was_running) => {
                if was_running {
                    self.status_message = "Lunar Client закрыт. Патчинг...".to_string();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
            Err(e) => {
                self.status_message = format!("Ошибка при закрытии процесса: {}", e);
                self.is_processing = false;
                return;
            }
        }
        
        // Patch accounts
        match self.patch_accounts() {
            Ok(_) => {
                self.status_message = "✓ Успешно! Токен и ник обновлены.".to_string();
            }
            Err(e) => {
                self.status_message = format!("✗ Ошибка: {}", e);
            }
        }
        
        self.is_processing = false;
    }
}

impl eframe::App for LunarPatcherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                
                ui.heading(egui::RichText::new("🌙 Lunar Client Patcher")
                    .size(28.0)
                    .color(egui::Color32::from_rgb(100, 150, 255)));
                
                ui.add_space(20.0);
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Никнейм:").size(16.0));
                ui.add_space(10.0);
                
                let text_edit = egui::TextEdit::singleline(&mut self.nickname)
                    .hint_text("Введите ваш ник")
                    .desired_width(250.0)
                    .font(egui::TextStyle::Body);
                ui.add(text_edit);
            });
            
            ui.add_space(20.0);
            
            ui.vertical_centered(|ui| {
                let button_enabled = !self.nickname.is_empty() && !self.is_processing;
                
                if ui.add_enabled(
                    button_enabled,
                    egui::Button::new(
                        egui::RichText::new("🔧 Patch")
                            .size(18.0)
                    )
                    .min_size(egui::vec2(150.0, 40.0))
                ).clicked() {
                    self.run_patch();
                }
            });
            
            ui.add_space(20.0);
            
            if !self.status_message.is_empty() {
                ui.vertical_centered(|ui| {
                    let color = if self.status_message.contains("Успешно") {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else if self.status_message.contains("Ошибка") {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        egui::Color32::from_rgb(200, 200, 200)
                    };
                    
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(16.0)
                            .color(color)
                    );
                });
            }
            
            ui.add_space(30.0);
            
            ui.separator();
            
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("ℹ Токен генерируется на случайный срок от 10 минут до 2 дней")
                        .size(12.0)
                        .color(egui::Color32::GRAY)
                );
                ui.label(
                    egui::RichText::new("Приложение автоматически закроет Lunar Client перед патчингом")
                        .size(12.0)
                        .color(egui::Color32::GRAY)
                );
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 380.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Lunar Client Patcher",
        options,
        Box::new(|_cc| Ok(Box::new(LunarPatcherApp::default()))),
    )
}