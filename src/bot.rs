use crate::config;

use std::{
    env,
    fs,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use teloxide::{
    prelude::*,
    types::{InputFile, KeyboardButton, KeyboardMarkup, Message},
};

fn main_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("🖥 Статус ПК")],
        vec![
            KeyboardButton::new("🔊 Громкость +"),
            KeyboardButton::new("🔉 Громкость -"),
            KeyboardButton::new("🔇 Без звука"),
        ],
        vec![
            KeyboardButton::new("🔒 Заблокировать"),
            KeyboardButton::new("📸 Скриншот"),
        ],
        vec![
            KeyboardButton::new("📂 Программы"),
            KeyboardButton::new("🌐 Сеть"),
        ],
        vec![
            KeyboardButton::new("🔴 Выключить ПК"),
            KeyboardButton::new("🔄 Перезагрузить ПК"),
        ],
        vec![KeyboardButton::new("❌ Отмена")],
    ])
}

fn run_powershell(script: &str) -> std::io::Result<std::process::Output> {
    Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
}

fn change_volume(key: u16) -> std::io::Result<()> {
    let script = format!(
        r#"
$src = @'
using System;
using System.Runtime.InteropServices;
public class AudioKeys {{
    [DllImport("user32.dll")]
    public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
}}
'@
Add-Type -TypeDefinition $src
[AudioKeys]::keybd_event({}, 0, 0, [UIntPtr]::Zero)
[AudioKeys]::keybd_event({}, 0, 2, [UIntPtr]::Zero)
"#,
        key, key
    );

    let result = run_powershell(&script)?;

    if result.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("Не удалось изменить громкость"))
    }
}

fn take_screenshot() -> std::io::Result<std::path::PathBuf> {
    let path = env::temp_dir().join("toddy_pc_screenshot.png");
    let safe_path = path.to_string_lossy().replace('\'', "''");

    let script = format!(
        r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)

$graphics.CopyFromScreen(
    $bounds.Location,
    [System.Drawing.Point]::Empty,
    $bounds.Size
)

$bitmap.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png)

$graphics.Dispose()
$bitmap.Dispose()
"#,
        safe_path
    );

    let result = run_powershell(&script)?;

    if result.status.success() && path.exists() {
        Ok(path)
    } else {
        Err(std::io::Error::other("Не удалось сделать скриншот"))
    }
}

pub async fn run_bot() {

    let config = config::Config::load()
        .expect("Не удалось загрузить config.json");

    let admin_id = config.admin_id;
    let bot = Bot::new(config.telegram_token);
    let pending = Arc::new(Mutex::new(None::<(&'static str, Instant)>));

    println!("TODDY PC CONTROL is running!");

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let pending = pending.clone();

        async move {
            let user_id = msg.from.as_ref().map(|u| u.id.0);

            if user_id != Some(admin_id) || !msg.chat.is_private() {
                return Ok(());
            }

            let text = msg.text().unwrap_or("");

            match text {
                "/start" | "⬅️ Главное меню" => {
                    bot.send_message(
                        msg.chat.id,
                        "🖥 TODDY PC CONTROL\n\nПанель управления Windows готова.",
                    )
                    .reply_markup(main_menu())
                    .await?;
                }

                "/id" => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Твой ID: {}", admin_id),
                    )
                    .await?;
                }

                "🖥 Статус ПК" => {
                    bot.send_message(
                        msg.chat.id,
                        "🟢 Компьютер включён.\nБот работает.",
                    )
                    .reply_markup(main_menu())
                    .await?;
                }

                "🔊 Громкость +" => {
                    let result = change_volume(0xAF);

                    let response = if result.is_ok() {
                        "🔊 Громкость увеличена."
                    } else {
                        "❌ Не удалось изменить громкость."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🔉 Громкость -" => {
                    let result = change_volume(0xAE);

                    let response = if result.is_ok() {
                        "🔉 Громкость уменьшена."
                    } else {
                        "❌ Не удалось изменить громкость."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🔇 Без звука" => {
                    let result = change_volume(0xAD);

                    let response = if result.is_ok() {
                        "🔇 Команда отключения/включения звука отправлена."
                    } else {
                        "❌ Не удалось изменить звук."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🔒 Заблокировать" => {
                    let result = Command::new("rundll32.exe")
                        .args(["user32.dll,LockWorkStation"])
                        .spawn();

                    let response = if result.is_ok() {
                        "🔒 Команда блокировки Windows отправлена."
                    } else {
                        "❌ Не удалось заблокировать Windows."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "📸 Скриншот" => {
                    bot.send_message(
                        msg.chat.id,
                        "📸 Делаю скриншот рабочего стола...",
                    ).await?;

                    match take_screenshot() {
                        Ok(path) => {
                            bot.send_photo(
                                msg.chat.id,
                                InputFile::file(path.clone()),
                            )
                            .await?;

                            let _ = fs::remove_file(path);
                        }

                        Err(_) => {
                            bot.send_message(
                                msg.chat.id,
                                "❌ Не удалось сделать скриншот.",
                            ).await?;
                        }
                    }
                }

                "📂 Программы" => {
                    let keyboard = KeyboardMarkup::new(vec![
                        vec![KeyboardButton::new("📝 Блокнот")],
                        vec![KeyboardButton::new("🧮 Калькулятор")],
                        vec![KeyboardButton::new("📁 Проводник")],
                        vec![KeyboardButton::new("⬅️ Главное меню")],
                    ]);

                    bot.send_message(
                        msg.chat.id,
                        "📂 Выбери программу для запуска:",
                    )
                    .reply_markup(keyboard)
                    .await?;
                }

                "📝 Блокнот" => {
                    let result = Command::new("notepad.exe").spawn();

                    let response = if result.is_ok() {
                        "✅ Блокнот запущен."
                    } else {
                        "❌ Не удалось запустить Блокнот."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🧮 Калькулятор" => {
                    let result = Command::new("calc.exe").spawn();

                    let response = if result.is_ok() {
                        "✅ Калькулятор запущен."
                    } else {
                        "❌ Не удалось запустить Калькулятор."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "📁 Проводник" => {
                    let result = Command::new("explorer.exe").spawn();

                    let response = if result.is_ok() {
                        "✅ Проводник запущен."
                    } else {
                        "❌ Не удалось запустить Проводник."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🌐 Сеть" => {
                    let script = r#"
Get-NetIPAddress -AddressFamily IPv4 |
Where-Object {
    $_.IPAddress -ne '127.0.0.1' -and
    $_.IPAddress -notlike '169.254*'
} |
Select-Object -ExpandProperty IPAddress
"#;

                    let response = match run_powershell(script) {
                        Ok(output) if output.status.success() => {
                            let ips = String::from_utf8_lossy(&output.stdout);

                            if ips.trim().is_empty() {
                                "Не удалось определить локальный IP.".to_string()
                            } else {
                                format!("🌐 Локальные IP-адреса:\n{}", ips.trim())
                            }
                        }
                        _ => "❌ Не удалось получить информацию о сети.".to_string(),
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                "🔴 Выключить ПК" => {
                    *pending.lock().unwrap() = Some(("shutdown", Instant::now()));

                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Выключить компьютер через 30 секунд?",
                    )
                    .reply_markup(KeyboardMarkup::new(vec![
                        vec![KeyboardButton::new("\u{041F}\u{041E}\u{0414}\u{0422}\u{0412}\u{0415}\u{0420}\u{0414}\u{0418}\u{0422}\u{042C}")],
                        vec![KeyboardButton::new("❌ Отмена")],
                    ]))
                    .await?;
                }

                "🔄 Перезагрузить ПК" => {
                    *pending.lock().unwrap() = Some(("restart", Instant::now()));

                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Перезагрузить компьютер через 30 секунд?",
                    )
                    .reply_markup(KeyboardMarkup::new(vec![
                        vec![KeyboardButton::new("\u{041F}\u{041E}\u{0414}\u{0422}\u{0412}\u{0415}\u{0420}\u{0414}\u{0418}\u{0422}\u{042C}")],
                        vec![KeyboardButton::new("❌ Отмена")],
                    ]))
                    .await?;
                }

                "\u{041F}\u{041E}\u{0414}\u{0422}\u{0412}\u{0415}\u{0420}\u{0414}\u{0418}\u{0422}\u{042C}" => {
                    let action = pending.lock().unwrap().take();

                    let action = match action {
                        Some((name, created_at))
                            if created_at.elapsed() <= Duration::from_secs(60) => name,

                        Some(_) => {
                            bot.send_message(
                                msg.chat.id,
                                "Confirmation expired. Please request it again.",
                            ).await?;
                            return Ok(());
                        }

                        None => {
                            bot.send_message(
                                msg.chat.id,
                                "No pending action.",
                            ).await?;
                            return Ok(());
                        }
                    };

                    let result = match action {
                        "shutdown" => {
                            Command::new("shutdown")
                                .args(["/s", "/t", "30"])
                                .spawn()
                        }

                        "restart" => {
                            Command::new("shutdown")
                                .args(["/r", "/t", "30"])
                                .spawn()
                        }

                        _ => {
                            bot.send_message(
                                msg.chat.id,
                                "Unknown action.",
                            ).await?;
                            return Ok(());
                        }
                    };

                    let response = if result.is_ok() {
                        "Command accepted by Windows. Execution in 30 seconds."
                    } else {
                        "Failed to execute Windows command."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }
                "❌ Отмена" => {
                    let action = pending.lock().unwrap().take();

                    let response = if action.is_some() {
                        "Действие отменено."
                    } else {
                        "Нет ожидающего действия."
                    };

                    bot.send_message(msg.chat.id, response).await?;
                }

                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "РСЃРїРѕР»СЊР·СѓР№ РєРЅРѕРїРєРё РјРµРЅСЋ.",
                    )
                    .reply_markup(main_menu())
                    .await?;
                }
            }

            Ok(())
        }
    })
    .await;
}
