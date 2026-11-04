use crate::types::{Context, Error};
use crate::utils::embeds::send_embed;
use serenity::model::{
    channel::{PermissionOverwriteType, PermissionOverwrite},
    permissions::Permissions,
};
use serenity::builder::EditChannel;

#[poise::command(slash_command, prefix_command, default_member_permissions = "MANAGE_CHANNELS", required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn lock(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let everyone_role_id = serenity::model::id::RoleId::new(guild_id.get());
    
    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(mut c) = channel {
        let overwrite = PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(everyone_role_id),
        };
        
        let builder = EditChannel::new().permissions(vec![overwrite]);
        if let Err(e) = c.edit(ctx.http(), builder).await {
            send_embed(ctx, "Error", &format!("Failed to lock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, "Admin: Lock", "This channel has been locked.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "MANAGE_CHANNELS", required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn unlock(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let everyone_role_id = serenity::model::id::RoleId::new(guild_id.get());
    
    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(c) = channel {
        if let Err(e) = c.delete_permission(ctx.http(), PermissionOverwriteType::Role(everyone_role_id)).await {
            send_embed(ctx, "Error", &format!("Failed to unlock channel: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    send_embed(ctx, "Admin: Unlock", "This channel has been unlocked.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "MANAGE_CHANNELS", required_permissions = "MANAGE_CHANNELS", category = "Admin")]
pub async fn slowmode(
    ctx: Context<'_>, 
    #[description = "Duration in seconds (0 to disable)"] seconds: u16
) -> Result<(), Error> {
    if seconds > 21600 {
        send_embed(ctx, "Error", "Slowmode cannot exceed 21600 seconds (6 hours).", 0xED4245).await?;
        return Ok(());
    }

    let channel = match ctx.channel_id().to_channel(ctx.http()).await {
        Ok(c) => c.guild(),
        Err(_) => None,
    };
    
    if let Some(mut c) = channel {
        let builder = EditChannel::new().rate_limit_per_user(seconds);
        if let Err(e) = c.edit(ctx.http(), builder).await {
            send_embed(ctx, "Error", &format!("Failed to set slowmode: {}", e), 0xED4245).await?;
            return Ok(());
        }
    }

    let status_text = if seconds == 0 {
        "Slowmode has been disabled.".to_string()
    } else {
        format!("Slowmode set to {} seconds.", seconds)
    };

    send_embed(ctx, "Admin: Slowmode", &status_text, 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin")]
pub async fn chatbot(
    ctx: Context<'_>, 
    #[description = "Action: 'enable' or 'disable'"] action: String
) -> Result<(), Error> {
    let action = action.to_lowercase();
    
    if action == "enable" || action == "on" {
        *ctx.data().chatbot_enabled.write().await = true;
        send_embed(ctx, "Chatbot AI", "Chatbot AI has been **ENABLED**.\nThe bot will now reply to tags and replies.", 0x2b2d31).await?;
    } else if action == "disable" || action == "off" {
        *ctx.data().chatbot_enabled.write().await = false;
        send_embed(ctx, "Chatbot AI", "Chatbot AI has been **DISABLED**.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "Usage: `/chatbot enable` or `/chatbot disable`", 0xED4245).await?;
    }
    
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin")]
pub async fn status(
    ctx: Context<'_>, 
    #[description = "Type: 'playing', 'watching', 'listening', 'competing'"] activity_type: String,
    #[rest]
    #[description = "The status message"] message: String,
) -> Result<(), Error> {
    use serenity::all::ActivityData;
    use serenity::all::OnlineStatus;

    let activity = match activity_type.to_lowercase().as_str() {
        "watching" => Some(ActivityData::watching(&message)),
        "listening" => Some(ActivityData::listening(&message)),
        "competing" => Some(ActivityData::competing(&message)),
        _ => Some(ActivityData::playing(&message)),
    };

    ctx.serenity_context().set_presence(activity, OnlineStatus::Online);
    send_embed(ctx, "Admin: Status", &format!("Status updated successfully."), 0x2b2d31).await?;
    
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin")]
pub async fn sudo(
    ctx: Context<'_>,
    #[description = "Channel tujuan"] channel: serenity::all::ChannelId,
    #[rest]
    #[description = "Pesan yang ingin dikirim"] message: String,
) -> Result<(), Error> {
    if message.trim().is_empty() {
        send_embed(ctx, "Error", "Pesan tidak boleh kosong.", 0xED4245).await?;
        return Ok(());
    }

    if let Err(e) = channel.say(ctx.http(), &message).await {
        send_embed(ctx, "Error", &format!("Gagal mengirim pesan: {}", e), 0xED4245).await?;
    }
    
    Ok(())
}

use sqlx::Row;

#[poise::command(slash_command, prefix_command, category = "Admin", default_member_permissions = "ADMINISTRATOR", required_permissions = "MANAGE_GUILD", check = "crate::utils::checks::is_staff", subcommands("add_autoreply", "list_autoreplies", "remove_autoreply"))]
pub async fn autoreply(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "add")]
pub async fn add_autoreply(
    ctx: Context<'_>,
    #[description = "The text that triggers the auto-reply."] trigger: String,
    #[description = "The response text."] response: Option<String>,
    #[description = "An image to attach to the response."] media: Option<serenity::model::channel::Attachment>,
    #[description = "Use Advanced Components V2 Container style?"] use_container: Option<bool>,
) -> Result<(), Error> {
    if response.is_none() && media.is_none() {
        send_embed(ctx, "Error", "You must provide either a response or a media attachment.", 0xED4245).await?;
        return Ok(());
    }

    let guild_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => {
            send_embed(ctx, "Error", "This command can only be used in a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    
    let media_url = media.map(|m| m.url.clone());
    let use_container = use_container.unwrap_or(false);

    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query(
        "INSERT INTO khivella_autoreplies (guild_id, trigger, response, media_url, use_container)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (guild_id, trigger) DO UPDATE
         SET response = EXCLUDED.response, media_url = EXCLUDED.media_url, use_container = EXCLUDED.use_container"
    )
    .bind(&guild_id)
    .bind(&trigger.to_lowercase())
    .bind(&response)
    .bind(&media_url)
    .bind(&use_container)
    .execute(db_pool)
    .await;

    match res {
        Ok(_) => send_embed(ctx, "Success", &format!("Auto-reply added for trigger: `{}`", trigger), 0x2b2d31).await?,
        Err(e) => send_embed(ctx, "Error", &format!("Failed to save to database: {}", e), 0xED4245).await?,
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list")]
pub async fn list_autoreplies(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => {
            send_embed(ctx, "Error", "This command can only be used in a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    
    let db_pool = &ctx.data().db_pool;

    let replies = sqlx::query(
        "SELECT trigger, response, media_url FROM khivella_autoreplies WHERE guild_id = $1"
    )
    .bind(&guild_id)
    .fetch_all(db_pool)
    .await;

    match replies {
        Ok(rows) => {
            if rows.is_empty() {
                send_embed(ctx, "Auto-replies", "No auto-replies found for this server.", 0x2b2d31).await?;
            } else {
                let mut content = String::new();
                for r in rows {
                    let trigger_text: String = r.get("trigger");
                    let mut details = String::new();
                    
                    if let Ok(resp) = r.try_get::<String, _>("response") {
                        if !resp.is_empty() {
                            details.push_str(&format!("Response: {}\n", resp));
                        }
                    }
                    if let Ok(m) = r.try_get::<String, _>("media_url") {
                        if !m.is_empty() {
                            details.push_str(&format!("Media: {}\n", m));
                        }
                    }
                    content.push_str(&format!("**Trigger:** `{}`\n{}\n", trigger_text, details));
                }
                send_embed(ctx, "Auto-replies", &content, 0x2b2d31).await?;
            }
        }
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to fetch from database: {}", e), 0xED4245).await?;
        }
    }
    
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "remove")]
pub async fn remove_autoreply(
    ctx: Context<'_>,
    #[description = "The trigger content to remove."] trigger: String,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.to_string(),
        None => {
            send_embed(ctx, "Error", "This command can only be used in a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query(
        "DELETE FROM khivella_autoreplies WHERE guild_id = $1 AND trigger = $2"
    )
    .bind(&guild_id)
    .bind(&trigger.to_lowercase())
    .execute(db_pool)
    .await;

    match res {
        Ok(result) => {
            if result.rows_affected() > 0 {
                send_embed(ctx, "Success", &format!("Auto-reply removed for trigger: `{}`", trigger), 0x2b2d31).await?;
            } else {
                send_embed(ctx, "Not Found", &format!("No auto-reply found for trigger: `{}`", trigger), 0xED4245).await?;
            }
        }
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to delete from database: {}", e), 0xED4245).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Admin", default_member_permissions = "ADMINISTRATOR", required_permissions = "MANAGE_GUILD", check = "crate::utils::checks::is_staff", subcommands("booster_background", "booster_channel", "booster_style", "booster_test", "booster_text"))]
pub async fn booster(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "background")]
pub async fn booster_background(
    ctx: Context<'_>,
    #[description = "Direct URL to the background image"] url: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    sqlx::query(
        "INSERT INTO khivella_booster (guild_id, background_url) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET background_url = EXCLUDED.background_url"
    )
    .bind(&guild_id).bind(&url).execute(db_pool).await?;

    send_embed(ctx, "Booster System", "Booster banner background URL updated.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "channel")]
pub async fn booster_channel(
    ctx: Context<'_>,
    #[description = "Channel where booster messages are sent"] channel: serenity::model::channel::Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    sqlx::query(
        "INSERT INTO khivella_booster (guild_id, channel_id) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id"
    )
    .bind(&guild_id).bind(&channel.id().to_string()).execute(db_pool).await?;

    send_embed(ctx, "Booster System", &format!("Booster channel set to <#{}>", channel.id()), 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "style")]
pub async fn booster_style(
    ctx: Context<'_>,
    #[description = "Choose the message style (banner card or plain text)"] style: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    sqlx::query(
        "INSERT INTO khivella_booster (guild_id, style) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET style = EXCLUDED.style"
    )
    .bind(&guild_id).bind(&style).execute(db_pool).await?;

    send_embed(ctx, "Booster System", &format!("Booster style set to: {}", style), 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "text")]
pub async fn booster_text(
    ctx: Context<'_>,
    #[description = "Booster text. Placeholders: {username}, {guildName}, etc."] text: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    sqlx::query(
        "INSERT INTO khivella_booster (guild_id, text) VALUES ($1, $2)
         ON CONFLICT (guild_id) DO UPDATE SET text = EXCLUDED.text"
    )
    .bind(&guild_id).bind(&text).execute(db_pool).await?;

    send_embed(ctx, "Booster System", "Booster message text updated.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "test")]
pub async fn booster_test(
    ctx: Context<'_>,
    #[description = "User to test with"] user: Option<serenity::model::user::User>,
) -> Result<(), Error> {
    let target = user.unwrap_or_else(|| ctx.author().clone());
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    let row = sqlx::query("SELECT channel_id, style, text, background_url FROM khivella_booster WHERE guild_id = $1")
        .bind(&guild_id).fetch_optional(db_pool).await?;

    if let Some(r) = row {
        let channel_id: Option<String> = r.try_get("channel_id").unwrap_or(None);
        let text: Option<String> = r.try_get("text").unwrap_or(None);
        
        let mut msg_text = text.unwrap_or_else(|| "Thank you {username} for boosting the server!".to_string());
        msg_text = msg_text.replace("{username}", &target.name);
        if let Some(guild) = ctx.partial_guild().await {
            msg_text = msg_text.replace("{guildName}", &guild.name);
        }

        if let Some(cid) = channel_id {
            if let Ok(id) = cid.parse::<u64>() {
                let channel = serenity::model::id::ChannelId::new(id);
                let _ = channel.send_message(&ctx.http(), serenity::builder::CreateMessage::new().content(msg_text)).await;
            }
        }
        send_embed(ctx, "Booster System", "Test message sent.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "Booster system not configured yet.", 0xED4245).await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Admin", default_member_permissions = "ADMINISTRATOR", required_permissions = "MANAGE_MESSAGES", check = "crate::utils::checks::is_staff", subcommands("sticky_set", "sticky_remove", "sticky_list"))]
pub async fn sticky(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "set")]
pub async fn sticky_set(
    ctx: Context<'_>,
    #[description = "The content of the sticky message."] message: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let channel_id = ctx.channel_id().to_string();
    let db_pool = &ctx.data().db_pool;

    let sent_msg = ctx.channel_id().send_message(&ctx.http(), serenity::builder::CreateMessage::new().content(&message)).await?;
    let msg_id = sent_msg.id.to_string();

    sqlx::query(
        "INSERT INTO khivella_sticky (guild_id, channel_id, message, last_message_id) VALUES ($1, $2, $3, $4)
         ON CONFLICT (guild_id, channel_id) DO UPDATE SET message = EXCLUDED.message, last_message_id = EXCLUDED.last_message_id"
    )
    .bind(&guild_id).bind(&channel_id).bind(&message).bind(&msg_id).execute(db_pool).await?;

    send_embed(ctx, "Sticky Message", "Sticky message set for this channel.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "remove")]
pub async fn sticky_remove(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let channel_id = ctx.channel_id().to_string();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_sticky WHERE guild_id = $1 AND channel_id = $2")
        .bind(&guild_id).bind(&channel_id).execute(db_pool).await?;

    if res.rows_affected() > 0 {
        send_embed(ctx, "Sticky Message", "Sticky message removed from this channel.", 0x2b2d31).await?;
    } else {
        send_embed(ctx, "Error", "No sticky message found in this channel.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list")]
pub async fn sticky_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query("SELECT channel_id, message FROM khivella_sticky WHERE guild_id = $1")
        .bind(&guild_id).fetch_all(db_pool).await?;

    if rows.is_empty() {
        send_embed(ctx, "Sticky Messages", "No sticky messages in this server.", 0x2b2d31).await?;
    } else {
        let mut content = String::new();
        for r in rows {
            let cid: String = r.get("channel_id");
            let msg: String = r.get("message");
            let display_msg = if msg.len() > 50 {
                format!("{}...", &msg[..47])
            } else {
                msg
            };
            content.push_str(&format!("<#{}>: `{}`\n", cid, display_msg));
        }
        send_embed(ctx, "Sticky Messages", &content, 0x2b2d31).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Admin", default_member_permissions = "ADMINISTRATOR", required_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff")]
pub async fn restart(ctx: Context<'_>) -> Result<(), Error> {
    let msg = "Memulai proses *reboot* sistem secara paksa. Porpan akan offline sejenak dan secara otomatis menyala kembali melalui protokol *auto-recovery* Railway.\n\nHarap tunggu beberapa saat...";
    send_embed(ctx, "System Reboot Initiated", msg, 0xef4444).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    std::process::exit(1);
}

#[derive(serde::Serialize)]
struct RobloxUserRequest {
    userIds: Vec<u64>,
    #[serde(rename = "excludeBannedUsers")]
    exclude_banned_users: bool,
}

#[poise::command(slash_command, prefix_command, category = "Admin", default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", aliases("checkname"))]
pub async fn checknames(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    
    let clan_data_str = ctx.data().clan_data.read().await.clone();
    if clan_data_str.is_empty() {
        send_embed(ctx, "Error", "Clan data is empty or not yet loaded from the website. Please try again later.", 0xED4245).await?;
        return Ok(());
    }
    
    let members: serde_json::Value = match serde_json::from_str(&clan_data_str) {
        Ok(v) => v,
        Err(_) => {
            send_embed(ctx, "Error", "Failed to parse clan data.", 0xED4245).await?;
            return Ok(());
        }
    };
    
    let mut user_map = std::collections::HashMap::new();
    let mut user_ids = Vec::new();
    
    let members_arr = members.as_array().or_else(|| members["members"].as_array());
    
    if let Some(arr) = members_arr {
        for member in arr {
            if let Some(profile_url) = member["robloxProfile"].as_str() {
                if let Some(id_str) = profile_url.split("users/").nth(1).and_then(|s| s.split('/').next()) {
                    if let Ok(id) = id_str.parse::<u64>() {
                        user_ids.push(id);
                        user_map.insert(id, member.clone());
                    }
                }
            }
        }
    }
    
    if user_ids.is_empty() {
        send_embed(ctx, "Check Names", "No Roblox profiles found in clan data.", 0x2b2d31).await?;
        return Ok(());
    }
    
    let client = reqwest::Client::new();
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0;
    
    for chunk in user_ids.chunks(100) {
        let req_body = RobloxUserRequest {
            userIds: chunk.to_vec(),
            exclude_banned_users: false,
        };
        
        match client.post("https://users.roblox.com/v1/users")
            .json(&req_body)
            .send()
            .await {
            Ok(res) => {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(users) = json["data"].as_array() {
                        for user in users {
                            if let Some(roblox_id) = user["id"].as_u64() {
                                if let Some(member) = user_map.get(&roblox_id) {
                                    let roblox_display_name = user["displayName"].as_str().unwrap_or("").to_lowercase();
                                    let roblox_username = user["name"].as_str().unwrap_or("").to_lowercase();
                                    
                                    let json_name = member["name"].as_str().unwrap_or("");
                                    let json_username = member["username"].as_str().unwrap_or("");
                                    
                                    let mut detail = String::new();
                                    if roblox_display_name != json_name.to_lowercase() {
                                        detail = format!("`{}` -> `{}`", json_name, user["displayName"].as_str().unwrap_or(""));
                                    } else if roblox_username != json_username.to_lowercase() {
                                        detail = format!("`@{}` -> `@{}`", json_username, user["name"].as_str().unwrap_or(""));
                                    }
                                    
                                    if !detail.is_empty() {
                                        if let Some(discord_id) = member["socials"]["discordId"].as_str() {
                                            offenders.push(format!("<@{}>: {}", discord_id, detail));
                                        }
                                    }
                                    checked += 1;
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => {
                send_embed(ctx, "Error", "Failed to fetch from Roblox API.", 0xED4245).await?;
                return Ok(());
            }
        }
    }
    
    if offenders.is_empty() {
        send_embed(ctx, "Check Names", &format!("Checked {} accounts. All names are matching!", checked), 0x00FF00).await?;
    } else {
        let mut msg_content = "**UNAUTHORIZED NAME CHANGE DETECTED**\n\nThe following users have changed their Roblox Name without prior notice:\n\n".to_string();
        
        for offender in &offenders {
            msg_content.push_str(offender);
            msg_content.push('\n');
        }
        
        msg_content.push_str("\n**ACTION REQUIRED**\nPlease open a ticket within 24 hours to clarify this name change, or you will be kicked.");
        
        // Send to specific channel 1556213720173125642
        let target_channel_id = serenity::model::id::ChannelId::new(1556213720173125642);
        let _ = target_channel_id.send_message(ctx.http(), serenity::builder::CreateMessage::new().content(&msg_content)).await;
        
        send_embed(ctx, "Check Names", &format!("Found {} mismatches! Warning has been sent to the target channel.", offenders.len()), 0xFFD700).await?;
    }
    
    Ok(())
}

#[poise::command(slash_command, prefix_command, check = "crate::utils::checks::is_staff", category = "Admin", subcommands("autothread_set", "autothread_remove", "autothread_list"), rename = "autothread")]
pub async fn autothread(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "set", category = "Admin")]
pub async fn autothread_set(
    ctx: Context<'_>, 
    #[description = "Target channel"] channel: serenity::model::channel::Channel,
    #[description = "Emojis to react with (separated by space)"] #[rest] emojis: String
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_galleries (guild_id, channel_id, emojis) VALUES ($1, $2, $3) ON CONFLICT (guild_id, channel_id) DO UPDATE SET emojis = $3")
        .bind(guild_id.to_string())
        .bind(channel_id.to_string())
        .bind(emojis.clone())
        .execute(db_pool)
        .await;

    match res {
        Ok(_) => {
            send_embed(ctx, "Auto-Thread Configured", &format!("Channel <#{}> has been set up for auto-threads.\n**Auto-Reacts:** {}", channel_id, emojis), 0x2ecc71).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to configure auto-thread: {}", e), 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "remove", category = "Admin")]
pub async fn autothread_remove(
    ctx: Context<'_>, 
    #[description = "Target channel"] channel: serenity::model::channel::Channel
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_galleries WHERE guild_id = $1 AND channel_id = $2")
        .bind(guild_id.to_string())
        .bind(channel_id.to_string())
        .execute(db_pool)
        .await;

    match res {
        Ok(_) => {
            send_embed(ctx, "Auto-Thread Removed", &format!("Channel <#{}> is no longer using auto-threads.", channel_id), 0x2ecc71).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to remove auto-thread: {}", e), 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list", category = "Admin")]
pub async fn autothread_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query("SELECT channel_id, emojis FROM khivella_galleries WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .fetch_all(db_pool)
        .await;

    match rows {
        Ok(results) => {
            if results.is_empty() {
                send_embed(ctx, "Configured Auto-Threads", "No auto-thread channels have been set up yet.", 0x3498db).await?;
                return Ok(());
            }

            let mut desc = String::new();
            use sqlx::Row;
            for r in results {
                let cid: String = r.get("channel_id");
                let emojis: String = r.get("emojis");
                desc.push_str(&format!("• <#{}> => {}\n", cid, emojis));
            }
            send_embed(ctx, "Configured Auto-Threads", &desc, 0x3498db).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to fetch auto-threads: {}", e), 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin", subcommands("absen_manage_open", "absen_manage_close", "absen_manage_list", "absen_manage_check", "absen_manage_clear", "absen_manage_setchannel", "absen_manage_remove", "absen_manage_reset_guests"), rename = "absen_manage")]
pub async fn absen_manage(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "setchannel", category = "Admin")]
pub async fn absen_manage_setchannel(
    ctx: Context<'_>, 
    #[description = "Target channel"] channel: serenity::model::channel::Channel
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_absen_state (guild_id, channel_id) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET channel_id = $2")
        .bind(guild_id.to_string())
        .bind(channel_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Attendance Channel Set", &format!("Attendance command is now restricted to <#{}>.", channel_id), 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to set attendance channel.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "open", category = "Admin")]
pub async fn absen_manage_open(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_absen_state (guild_id, is_open) VALUES ($1, TRUE) ON CONFLICT (guild_id) DO UPDATE SET is_open = TRUE")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Attendance Opened", "The monthly attendance is now **OPEN**. Members can now use the `/absen` command.", 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to open the attendance system.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "close", category = "Admin")]
pub async fn absen_manage_close(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_absen_state (guild_id, is_open) VALUES ($1, FALSE) ON CONFLICT (guild_id) DO UPDATE SET is_open = FALSE")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Attendance Closed", "The monthly attendance is now **CLOSED**. Members can no longer record their attendance.", 0xED4245).await?;
    } else {
        send_embed(ctx, "Error", "Failed to close the attendance system.", 0xED4245).await?;
    }
    Ok(())
}

pub async fn show_absen_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query(
        "SELECT discord_id, discord_username, roblox_name, roblox_username, roblox_display_name, role_id, status, timestamp 
         FROM khivella_absen_records 
         WHERE guild_id = $1 
         ORDER BY timestamp ASC"
    )
    .bind(guild_id.to_string())
    .fetch_all(db_pool)
    .await;

    match rows {
        Ok(results) => {
            if results.is_empty() {
                send_embed(ctx, "Monthly Attendance Records", "No members have recorded attendance for this period yet.", 0x3498db).await?;
                return Ok(());
            }

            let mut desc = String::new();
            use sqlx::Row;
            for (i, r) in results.iter().enumerate() {
                let discord_id: String = r.get("discord_id");
                let discord_username: String = r.get("discord_username");
                let roblox_name: String = r.get("roblox_name");
                let roblox_username: Option<String> = r.try_get("roblox_username").unwrap_or(None);
                let roblox_display_name: Option<String> = r.try_get("roblox_display_name").unwrap_or(None);
                
                let disp = roblox_display_name.unwrap_or(roblox_name);
                let u_name = roblox_username.unwrap_or_else(|| disp.clone());

                let line = format!(
                    "{}. **{}** (`@{}`) - Discord: **{}** (`{}`)\n",
                    i + 1, disp, u_name, discord_username, discord_id
                );
                
                // Keep it under embed limits
                if desc.len() + line.len() > 3900 {
                    desc.push_str("...and additional records truncated due to embed character limit.");
                    break;
                }
                desc.push_str(&line);
            }
            send_embed(ctx, format!("Monthly Attendance Summary (Total: {} Members)", results.len()).as_str(), &desc, 0x2ecc71).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to retrieve attendance data: {}", e), 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list", category = "Admin")]
pub async fn absen_manage_list(ctx: Context<'_>) -> Result<(), Error> {
    show_absen_list(ctx).await
}

async fn fetch_audit_data(
    ctx: Context<'_>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(Vec<serenity::model::guild::Member>, usize, usize), Error> {
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query("SELECT discord_id FROM khivella_absen_records WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .fetch_all(db_pool)
        .await?;

    let mut recorded_ids = std::collections::HashSet::new();
    for r in rows {
        use sqlx::Row;
        let did: String = r.get("discord_id");
        recorded_ids.insert(did);
    }

    let mut all_members = Vec::new();
    let mut after: Option<u64> = None;
    loop {
        let chunk = ctx.http().get_guild_members(guild_id, Some(1000), after).await?;
        if chunk.is_empty() {
            break;
        }
        if let Some(last) = chunk.last() {
            after = Some(last.user.id.get());
        }
        let count = chunk.len();
        all_members.extend(chunk);
        if count < 1000 {
            break;
        }
    }

    let mut unrecorded_members = Vec::new();
    let mut attended_count = 0;
    let mut exempt_count = 0;

    for m in all_members {
        if m.user.bot {
            continue;
        }
        let has_member = m.roles.iter().any(|r| r.get() == crate::types::ROLE_MEMBER_ID);
        let has_veteran = m.roles.iter().any(|r| r.get() == crate::types::ROLE_VETERAN_ID);

        if !has_member {
            continue;
        }

        if has_veteran {
            exempt_count += 1;
            continue;
        }

        if recorded_ids.contains(&m.user.id.to_string()) {
            attended_count += 1;
        } else {
            unrecorded_members.push(m);
        }
    }

    Ok((unrecorded_members, attended_count, exempt_count))
}

fn build_audit_payload(
    unrecorded_members: &[serenity::model::guild::Member],
    attended_count: usize,
    exempt_count: usize,
    select_menu_id: &str,
    demote_btn_id: &str,
    refresh_btn_id: &str,
    cancel_btn_id: &str,
) -> (serenity::builder::CreateEmbed, Vec<serenity::builder::CreateActionRow>) {
    let mut desc = format!(
        "The following **{}** member(s) hold the Member role (<@&{}>) without Veteran exemption (<@&{}>), and have **not** recorded their monthly attendance.\n\n\
        **Audit Statistics:**\n\
        • Attended Members: **{}**\n\
        • Exempt Veterans: **{}**\n\
        • Unrecorded Members: **{}**\n\n\
        **Actions:**\n\
        • Select a member from the **dropdown menu** below to purge **1-by-1**.\n\
        • Click **Purge All to Guest** to demote all unrecorded members at once.\n\
        *(Purging removes all roles, assigns Guest <@&{}>, and resets nicknames)*\n\n\
        **Unrecorded Members List:**\n",
        unrecorded_members.len(),
        crate::types::ROLE_MEMBER_ID,
        crate::types::ROLE_VETERAN_ID,
        attended_count,
        exempt_count,
        unrecorded_members.len(),
        crate::types::ROLE_GUEST_ID,
    );

    let mut truncated_count = 0;
    for (i, m) in unrecorded_members.iter().enumerate() {
        let nick_info = if let Some(ref nick) = m.nick {
            format!(" (Nick: `{}`)", nick)
        } else {
            String::new()
        };
        let line = format!("{}. <@{}> (`@{}`){}\n", i + 1, m.user.id, m.user.name, nick_info);
        if desc.len() + line.len() > 3800 {
            truncated_count = unrecorded_members.len() - i;
            break;
        }
        desc.push_str(&line);
    }

    if truncated_count > 0 {
        desc.push_str(&format!("\n*...and {} more members (use dropdown or purge all to process).*\n", truncated_count));
    }

    let embed = serenity::builder::CreateEmbed::new()
        .title("Attendance Audit - Unrecorded Members Review")
        .description(desc)
        .color(0xe67e22)
        .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • Monthly Audit"));

    let select_options: Vec<serenity::builder::CreateSelectMenuOption> = unrecorded_members
        .iter()
        .take(25)
        .enumerate()
        .map(|(i, m)| {
            let label = format!("{}. {}", i + 1, m.user.name);
            let label_clean = if label.len() > 100 { &label[..100] } else { &label };
            let nick_str = m.nick.as_deref().unwrap_or("Default");
            let desc_opt = format!("Nick: {} | ID: {}", nick_str, m.user.id);
            let desc_clean = if desc_opt.len() > 100 { &desc_opt[..100] } else { &desc_opt };
            serenity::builder::CreateSelectMenuOption::new(label_clean, m.user.id.to_string())
                .description(desc_clean)
        })
        .collect();

    let mut rows = Vec::new();
    if !select_options.is_empty() {
        let select_menu = serenity::builder::CreateSelectMenu::new(
            select_menu_id,
            serenity::builder::CreateSelectMenuKind::String { options: select_options },
        )
        .placeholder(format!("Select a member to purge 1-by-1 (Showing 1-{} of {})...", unrecorded_members.len().min(25), unrecorded_members.len()));

        rows.push(serenity::builder::CreateActionRow::SelectMenu(select_menu));
    }

    let demote_button = serenity::builder::CreateButton::new(demote_btn_id)
        .label(format!("Purge All to Guest ({})", unrecorded_members.len()))
        .style(serenity::all::ButtonStyle::Danger);

    let refresh_button = serenity::builder::CreateButton::new(refresh_btn_id)
        .label("Refresh")
        .style(serenity::all::ButtonStyle::Secondary);

    let cancel_button = serenity::builder::CreateButton::new(cancel_btn_id)
        .label("Cancel")
        .style(serenity::all::ButtonStyle::Secondary);

    rows.push(serenity::builder::CreateActionRow::Buttons(vec![
        demote_button,
        refresh_button,
        cancel_button,
    ]));

    (embed, rows)
}

pub async fn purge_single_user(ctx: Context<'_>, target_user: serenity::model::user::User) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            send_embed(ctx, "Error", "This command can only be used within a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    let db_pool = &ctx.data().db_pool;

    let member = match guild_id.member(ctx.http(), target_user.id).await {
        Ok(m) => m,
        Err(_) => {
            send_embed(ctx, "Member Not Found", &format!("Could not find <@{}> in this server.", target_user.id), 0xED4245).await?;
            return Ok(());
        }
    };

    // Check if target has Veteran role (exempt from attendance purge!)
    if member.roles.iter().any(|r| r.get() == crate::types::ROLE_VETERAN_ID) {
        send_embed(
            ctx,
            "Protected Member",
            &format!("Cannot purge <@{}>: This member holds the Veteran role (<@&{}>) and is exempt from attendance requirements.", target_user.id, crate::types::ROLE_VETERAN_ID),
            0xED4245
        ).await?;
        return Ok(());
    }

    let guest_role_id = serenity::model::id::RoleId::new(crate::types::ROLE_GUEST_ID);
    let edit_builder = serenity::builder::EditMember::new()
        .nickname("")
        .roles(vec![guest_role_id]);

    match guild_id.edit_member(ctx.http(), target_user.id, edit_builder).await {
        Ok(_) => {
            let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_purge_single', $1, $2, $3)")
                .bind(ctx.author().id.to_string())
                .bind(&ctx.author().name)
                .bind(format!("Purged user {} ({}) to Guest role and reset nickname", target_user.name, target_user.id))
                .execute(db_pool)
                .await;

            let embed = serenity::builder::CreateEmbed::new()
                .title("Member Purged to Guest")
                .description(format!("Successfully demoted <@{}> to Guest role and reset their server display name.", target_user.id))
                .color(0x2ecc71)
                .field("Demoted Member", format!("<@{}> (`@{}`)", target_user.id, target_user.name), true)
                .field("Assigned Role", format!("<@&{}>", crate::types::ROLE_GUEST_ID), true)
                .field("Nickname", "Reset to Discord default", false)
                .field("Action By", format!("<@{}>", ctx.author().id), false)
                .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            send_embed(ctx, "Demotion Failed", &format!("Failed to demote <@{}>: {}\nMake sure the bot's role is positioned higher in Server Settings > Roles.", target_user.id, e), 0xED4245).await?;
        }
    }

    Ok(())
}

pub async fn show_absen_audit(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            send_embed(ctx, "Error", "This command can only be used within a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    let db_pool = &ctx.data().db_pool;

    let (mut unrecorded_members, mut attended_count, mut exempt_count) = fetch_audit_data(ctx, guild_id).await?;

    if unrecorded_members.is_empty() {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Audit - All Clear")
            .description("All clan members holding the Member role have recorded attendance, or hold Veteran exemption.")
            .color(0x2ecc71)
            .field("Attended Members", format!("{} members", attended_count), true)
            .field("Exempt Veterans", format!("{} members", exempt_count), true)
            .field("Unrecorded Members", "0 members", true)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance Audit"));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let demote_btn_id = format!("absen_demote_{}", ctx.id());
    let refresh_btn_id = format!("absen_refresh_{}", ctx.id());
    let cancel_btn_id = format!("absen_cancel_{}", ctx.id());
    let select_menu_id = format!("absen_purge_sel_{}", ctx.id());

    let (embed, components) = build_audit_payload(
        &unrecorded_members,
        attended_count,
        exempt_count,
        &select_menu_id,
        &demote_btn_id,
        &refresh_btn_id,
        &cancel_btn_id,
    );

    let reply = ctx.send(poise::CreateReply::default().embed(embed).components(components)).await?;
    let msg = reply.message().await?;

    use serenity::futures::StreamExt;
    let mut collector = msg.await_component_interactions(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(300))
        .stream();

    while let Some(interaction) = collector.next().await {
        // Enforce staff permissions on interactions
        if interaction.user.id != ctx.author().id {
            let is_staff = crate::utils::checks::is_staff(ctx).await.unwrap_or(false);
            if !is_staff {
                let _ = interaction.create_response(ctx, serenity::builder::CreateInteractionResponse::Message(
                    serenity::builder::CreateInteractionResponseMessage::new()
                        .content("Only staff members can use these attendance audit controls.")
                        .ephemeral(true)
                )).await;
                continue;
            }
        }

        let _ = interaction.create_response(ctx, serenity::builder::CreateInteractionResponse::Acknowledge).await;

        if interaction.data.custom_id == cancel_btn_id {
            let cancel_embed = serenity::builder::CreateEmbed::new()
                .title("Attendance Audit - Cancelled")
                .description(format!("Audit session closed by <@{}>. No further modifications were made.", interaction.user.id))
                .color(0x7f8c8d);
            let _ = reply.edit(ctx, poise::CreateReply::default().embed(cancel_embed).components(vec![])).await;
            return Ok(());
        }

        if interaction.data.custom_id == refresh_btn_id {
            if let Ok((new_unrecorded, new_attended, new_exempt)) = fetch_audit_data(ctx, guild_id).await {
                unrecorded_members = new_unrecorded;
                attended_count = new_attended;
                exempt_count = new_exempt;

                if unrecorded_members.is_empty() {
                    let clear_embed = serenity::builder::CreateEmbed::new()
                        .title("Attendance Audit - All Clear")
                        .description("All clan members holding the Member role have recorded attendance, or hold Veteran exemption.")
                        .color(0x2ecc71)
                        .field("Attended Members", format!("{} members", attended_count), true)
                        .field("Exempt Veterans", format!("{} members", exempt_count), true)
                        .field("Unrecorded Members", "0 members", true)
                        .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance Audit"));

                    let _ = reply.edit(ctx, poise::CreateReply::default().embed(clear_embed).components(vec![])).await;
                    return Ok(());
                }

                let (new_embed, new_components) = build_audit_payload(
                    &unrecorded_members,
                    attended_count,
                    exempt_count,
                    &select_menu_id,
                    &demote_btn_id,
                    &refresh_btn_id,
                    &cancel_btn_id,
                );
                let _ = reply.edit(ctx, poise::CreateReply::default().embed(new_embed).components(new_components)).await;
            }
            continue;
        }

        if interaction.data.custom_id == demote_btn_id {
            let progress_embed = serenity::builder::CreateEmbed::new()
                .title("Attendance Audit - Processing Bulk Demotions...")
                .description(format!("Demoting {} unrecorded member(s), assigning Guest role, and resetting nicknames. Please wait...", unrecorded_members.len()))
                .color(0x3498db);
            let _ = reply.edit(ctx, poise::CreateReply::default().embed(progress_embed).components(vec![])).await;

            let mut success_count = 0;
            let mut fail_count = 0;
            let guest_role_id = serenity::model::id::RoleId::new(crate::types::ROLE_GUEST_ID);

            for m in &unrecorded_members {
                let edit_builder = serenity::builder::EditMember::new()
                    .nickname("")
                    .roles(vec![guest_role_id]);

                match guild_id.edit_member(ctx.http(), m.user.id, edit_builder).await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        tracing::warn!("Failed to demote member {}: {:?}", m.user.id, e);
                        fail_count += 1;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }

            let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_audit_demote_all', $1, $2, $3)")
                .bind(interaction.user.id.to_string())
                .bind(&interaction.user.name)
                .bind(format!("Demoted {} unrecorded members to Guest (failed: {})", success_count, fail_count))
                .execute(db_pool)
                .await;

            let finished_embed = serenity::builder::CreateEmbed::new()
                .title("Attendance Audit - Demotion Completed")
                .description("Role removal and demotion process has finished for all unrecorded members.")
                .color(0x2ecc71)
                .field("Successfully Demoted", format!("{} members", success_count), true)
                .field("Failed / Hierarchy Shield", format!("{} members", fail_count), true)
                .field("Assigned Role", format!("<@&{}>", crate::types::ROLE_GUEST_ID), false)
                .field("Nicknames", "Reset to Discord default", false)
                .field("Exempt Veterans Preserved", format!("{} members", exempt_count), false)
                .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance Audit"));

            let _ = reply.edit(ctx, poise::CreateReply::default().embed(finished_embed).components(vec![])).await;
            return Ok(());
        }

        if interaction.data.custom_id == select_menu_id {
            let selected_uid_str = match &interaction.data.kind {
                serenity::model::application::ComponentInteractionDataKind::StringSelect { values } => values.first().cloned(),
                _ => None,
            };

            if let Some(uid_str) = selected_uid_str {
                if let Ok(uid_num) = uid_str.parse::<u64>() {
                    let target_id = serenity::model::id::UserId::new(uid_num);
                    let guest_role_id = serenity::model::id::RoleId::new(crate::types::ROLE_GUEST_ID);
                    let edit_builder = serenity::builder::EditMember::new()
                        .nickname("")
                        .roles(vec![guest_role_id]);

                    let _ = guild_id.edit_member(ctx.http(), target_id, edit_builder).await;

                    let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_audit_demote_single', $1, $2, $3)")
                        .bind(interaction.user.id.to_string())
                        .bind(&interaction.user.name)
                        .bind(format!("1-by-1 Purge: Demoted user <@{}> to Guest role and reset nickname", uid_num))
                        .execute(db_pool)
                        .await;

                    unrecorded_members.retain(|m| m.user.id.get() != uid_num);

                    if unrecorded_members.is_empty() {
                        let clear_embed = serenity::builder::CreateEmbed::new()
                            .title("Attendance Audit - All Clear")
                            .description("All unrecorded members have been purged to the Guest role.")
                            .color(0x2ecc71)
                            .field("Attended Members", format!("{} members", attended_count), true)
                            .field("Exempt Veterans", format!("{} members", exempt_count), true)
                            .field("Unrecorded Members", "0 members", true)
                            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance Audit"));

                        let _ = reply.edit(ctx, poise::CreateReply::default().embed(clear_embed).components(vec![])).await;
                        return Ok(());
                    } else {
                        let (new_embed, new_components) = build_audit_payload(
                            &unrecorded_members,
                            attended_count,
                            exempt_count,
                            &select_menu_id,
                            &demote_btn_id,
                            &refresh_btn_id,
                            &cancel_btn_id,
                        );
                        let _ = reply.edit(ctx, poise::CreateReply::default().embed(new_embed).components(new_components)).await;
                    }
                }
            }
        }
    }

    // When interaction collector times out after 5 minutes
    let disabled_demote = serenity::builder::CreateButton::new(&demote_btn_id)
        .label(format!("Purge All to Guest ({})", unrecorded_members.len()))
        .style(serenity::all::ButtonStyle::Danger)
        .disabled(true);
    let disabled_refresh = serenity::builder::CreateButton::new(&refresh_btn_id)
        .label("Refresh")
        .style(serenity::all::ButtonStyle::Secondary)
        .disabled(true);
    let disabled_cancel = serenity::builder::CreateButton::new(&cancel_btn_id)
        .label("Cancel")
        .style(serenity::all::ButtonStyle::Secondary)
        .disabled(true);

    let disabled_row = serenity::builder::CreateActionRow::Buttons(vec![
        disabled_demote,
        disabled_refresh,
        disabled_cancel,
    ]);

    let _ = reply.edit(ctx, poise::CreateReply::default().components(vec![disabled_row])).await;

    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "check", category = "Admin")]
pub async fn absen_manage_check(
    ctx: Context<'_>,
    #[description = "Purge a specific member directly (leave empty to open unrecorded review panel)"]
    user: Option<serenity::model::user::User>,
) -> Result<(), Error> {
    if let Some(target_user) = user {
        purge_single_user(ctx, target_user).await
    } else {
        show_absen_audit(ctx).await
    }
}

#[poise::command(slash_command, prefix_command, rename = "clear", category = "Admin")]
pub async fn absen_manage_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_absen_records WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Attendance Cleared", "All attendance data for this month has been successfully cleared! The system is ready for the next month.", 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to clear attendance data.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "remove", category = "Admin")]
pub async fn absen_manage_remove(
    ctx: Context<'_>,
    #[description = "User to remove from attendance"] user: serenity::model::user::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_absen_records WHERE guild_id = $1 AND discord_id = $2")
        .bind(guild_id.to_string())
        .bind(user.id.to_string())
        .execute(db_pool)
        .await;

    if let Ok(result) = res {
        if result.rows_affected() > 0 {
            send_embed(ctx, "Attendance Removed", &format!("Successfully removed {} from the attendance list.", user.name), 0x2ecc71).await?;
        } else {
            send_embed(ctx, "Not Found", &format!("{} hasn't recorded their attendance yet.", user.name), 0xED4245).await?;
        }
    } else {
        send_embed(ctx, "Error", "Failed to remove the attendance record.", 0xED4245).await?;
    }
    Ok(())
}

pub async fn execute_reset_guest_nicks(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            send_embed(ctx, "Error", "This command can only be used within a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    let db_pool = &ctx.data().db_pool;

    // 1. Fetch all guild members via pagination
    let mut all_members = Vec::new();
    let mut after: Option<u64> = None;
    loop {
        let chunk = ctx.http().get_guild_members(guild_id, Some(1000), after).await?;
        if chunk.is_empty() {
            break;
        }
        if let Some(last) = chunk.last() {
            after = Some(last.user.id.get());
        }
        let count = chunk.len();
        all_members.extend(chunk);
        if count < 1000 {
            break;
        }
    }

    // 2. Filter for non-bot members with Guest role who currently have a custom nickname
    let guests_with_nick: Vec<serenity::model::guild::Member> = all_members
        .into_iter()
        .filter(|m| {
            !m.user.bot
                && m.roles.iter().any(|r| r.get() == crate::types::ROLE_GUEST_ID)
                && m.nick.is_some()
        })
        .collect();

    if guests_with_nick.is_empty() {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Guest Nicknames - All Clean")
            .description(format!(
                "No members with the Guest role (<@&{}>) have custom nicknames set. All guest display names are already at their default Discord names.",
                crate::types::ROLE_GUEST_ID
            ))
            .color(0x2ecc71)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Member Management"));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // 3. Create Confirmation UI with safety prompt
    let confirm_btn_id = format!("guest_nick_confirm_{}", ctx.id());
    let cancel_btn_id = format!("guest_nick_cancel_{}", ctx.id());

    let confirm_btn = serenity::builder::CreateButton::new(&confirm_btn_id)
        .label(format!("Reset All Guest Nicknames ({})", guests_with_nick.len()))
        .style(serenity::all::ButtonStyle::Danger);

    let cancel_btn = serenity::builder::CreateButton::new(&cancel_btn_id)
        .label("Cancel")
        .style(serenity::all::ButtonStyle::Secondary);

    let row = serenity::builder::CreateActionRow::Buttons(vec![confirm_btn.clone(), cancel_btn.clone()]);

    let embed = serenity::builder::CreateEmbed::new()
        .title("Reset Guest Display Names")
        .description(format!(
            "Found **{}** member(s) holding the Guest role (<@&{}>) with custom nicknames set (e.g. `GUEST ...`).\n\n\
            Click **Reset All Guest Nicknames** below to clear their server display names back to their original Discord names.\n\n\
            *• A safe rate-limit delay of 400ms per user will be applied.*\n\
            *• Members whose names are already default will be skipped automatically.*",
            guests_with_nick.len(),
            crate::types::ROLE_GUEST_ID
        ))
        .color(0xe67e22)
        .field("Target Role", format!("<@&{}>", crate::types::ROLE_GUEST_ID), true)
        .field("Affected Members", format!("{} members", guests_with_nick.len()), true)
        .footer(serenity::builder::CreateEmbedFooter::new("This action will reset nicknames to Discord default."));

    let reply = ctx.send(poise::CreateReply::default().embed(embed).components(vec![row])).await?;
    let msg = reply.message().await?;

    let interaction = msg
        .await_component_interaction(ctx)
        .timeout(std::time::Duration::from_secs(120))
        .await;

    let interaction = match interaction {
        Some(i) => i,
        None => {
            let disabled_row = serenity::builder::CreateActionRow::Buttons(vec![
                confirm_btn.disabled(true),
                cancel_btn.disabled(true),
            ]);
            let _ = reply.edit(ctx, poise::CreateReply::default().components(vec![disabled_row])).await;
            return Ok(());
        }
    };

    if interaction.user.id != ctx.author().id {
        let is_staff = crate::utils::checks::is_staff(ctx).await.unwrap_or(false);
        if !is_staff {
            let _ = interaction.create_response(ctx, serenity::builder::CreateInteractionResponse::Message(
                serenity::builder::CreateInteractionResponseMessage::new()
                    .content("Only staff members can authorize this action.")
                    .ephemeral(true)
            )).await;
            return Ok(());
        }
    }

    let _ = interaction.create_response(ctx, serenity::builder::CreateInteractionResponse::Acknowledge).await;

    if interaction.data.custom_id == cancel_btn_id {
        let cancel_embed = serenity::builder::CreateEmbed::new()
            .title("Reset Guest Nicknames - Cancelled")
            .description(format!("Action was cancelled by <@{}>. No nicknames were modified.", interaction.user.id))
            .color(0x7f8c8d);
        let _ = reply.edit(ctx, poise::CreateReply::default().embed(cancel_embed).components(vec![])).await;
        return Ok(());
    }

    if interaction.data.custom_id == confirm_btn_id {
        let progress_embed = serenity::builder::CreateEmbed::new()
            .title("Resetting Guest Display Names...")
            .description(format!("Clearing custom nicknames for **{}** member(s). Please wait...", guests_with_nick.len()))
            .color(0x3498db);
        let _ = reply.edit(ctx, poise::CreateReply::default().embed(progress_embed).components(vec![])).await;

        let mut success_count = 0;
        let mut fail_count = 0;

        for m in &guests_with_nick {
            let edit_builder = serenity::builder::EditMember::new().nickname("");
            match guild_id.edit_member(ctx.http(), m.user.id, edit_builder).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    tracing::warn!("Failed to reset nickname for guest {}: {:?}", m.user.id, e);
                    fail_count += 1;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }

        let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('guest_nicks_reset_all', $1, $2, $3)")
            .bind(interaction.user.id.to_string())
            .bind(&interaction.user.name)
            .bind(format!("Reset nicknames for {} guest members (failed: {})", success_count, fail_count))
            .execute(db_pool)
            .await;

        let finished_embed = serenity::builder::CreateEmbed::new()
            .title("Guest Nicknames Reset Complete")
            .description("Successfully cleared custom server nicknames for all members with the Guest role back to their default Discord names.")
            .color(0x2ecc71)
            .field("Successfully Reset", format!("{} members", success_count), true)
            .field("Failed / Shielded", format!("{} members", fail_count), true)
            .field("Target Role", format!("<@&{}>", crate::types::ROLE_GUEST_ID), false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Member Management"));

        let _ = reply.edit(ctx, poise::CreateReply::default().embed(finished_embed).components(vec![])).await;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin", rename = "resetguestnicks", aliases("reset_guest_nicks", "clearguestnicks", "resetguests"))]
pub async fn resetguestnicks(ctx: Context<'_>) -> Result<(), Error> {
    execute_reset_guest_nicks(ctx).await
}

#[poise::command(slash_command, prefix_command, rename = "reset_guests", category = "Admin")]
pub async fn absen_manage_reset_guests(ctx: Context<'_>) -> Result<(), Error> {
    execute_reset_guest_nicks(ctx).await
}

#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", check = "crate::utils::checks::is_staff", category = "Admin", subcommands("bday_manage_setchannel"), rename = "bday_manage")]
pub async fn bday_manage(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "setchannel", category = "Admin")]
pub async fn bday_manage_setchannel(
    ctx: Context<'_>, 
    #[description = "Target channel"] channel: serenity::model::channel::Channel
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_bday_config (guild_id, channel_id) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET channel_id = $2")
        .bind(guild_id.to_string())
        .bind(channel_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Birthday Channel Set", &format!("Birthday announcements will now be sent to <#{}>.", channel_id), 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to set birthday channel.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(
    slash_command, 
    prefix_command, 
    rename = "ffevent_manage", 
    category = "Admin",
    default_member_permissions = "ADMINISTRATOR",
    check = "crate::utils::checks::is_staff",
    subcommands("ffevent_manage_open", "ffevent_manage_close", "ffevent_manage_list", "ffevent_manage_clear", "ffevent_manage_setchannel", "ffevent_manage_remove")
)]
pub async fn ffevent_manage(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "setchannel", category = "Admin")]
pub async fn ffevent_manage_setchannel(
    ctx: Context<'_>, 
    #[description = "Target channel"] channel: serenity::model::channel::Channel
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = channel.id();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_ffevent_state (guild_id, channel_id) VALUES ($1, $2) ON CONFLICT (guild_id) DO UPDATE SET channel_id = $2")
        .bind(guild_id.to_string())
        .bind(channel_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Event Channel Set", &format!("Event join command is now restricted to <#{}>.", channel_id), 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to set event channel.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "open", category = "Admin")]
pub async fn ffevent_manage_open(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_ffevent_state (guild_id, is_open) VALUES ($1, TRUE) ON CONFLICT (guild_id) DO UPDATE SET is_open = TRUE")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Event Registration Opened", "The event registration is now **OPEN**. Members can now use the `/ffevent join` command.", 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to open the event system.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "close", category = "Admin")]
pub async fn ffevent_manage_close(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("INSERT INTO khivella_ffevent_state (guild_id, is_open) VALUES ($1, FALSE) ON CONFLICT (guild_id) DO UPDATE SET is_open = FALSE")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Event Registration Closed", "The event registration is now **CLOSED**. Members can no longer join the event.", 0xED4245).await?;
    } else {
        send_embed(ctx, "Error", "Failed to close the event system.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list", category = "Admin")]
pub async fn ffevent_manage_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query("SELECT discord_username, roblox_name, timestamp FROM khivella_ffevent_records WHERE guild_id = $1 ORDER BY timestamp ASC")
        .bind(guild_id.to_string())
        .fetch_all(db_pool)
        .await;

    match rows {
        Ok(results) => {
            if results.is_empty() {
                send_embed(ctx, "Event Data", "No members have registered for the event yet.", 0x3498db).await?;
                return Ok(());
            }

            let mut desc = String::new();
            use sqlx::Row;
            for (i, r) in results.iter().enumerate() {
                let discord_username: String = r.get("discord_username");
                let roblox_name: String = r.get("roblox_name");
                
                let line = format!("{}. **{}** (`@{}`)\n", i + 1, roblox_name, discord_username);
                
                // Keep it under embed limits
                if desc.len() + line.len() > 3900 {
                    desc.push_str("...and more.");
                    break;
                }
                desc.push_str(&line);
            }
            send_embed(ctx, format!("Event Registration Data (Total: {})", results.len()).as_str(), &desc, 0x3498db).await?;
        },
        Err(e) => {
            send_embed(ctx, "Error", &format!("Failed to fetch event data: {}", e), 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "clear", category = "Admin")]
pub async fn ffevent_manage_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_ffevent_records WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        send_embed(ctx, "Event Data Cleared", "All event registration data has been successfully cleared.", 0x2ecc71).await?;
    } else {
        send_embed(ctx, "Error", "Failed to clear event data.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "remove", category = "Admin")]
pub async fn ffevent_manage_remove(
    ctx: Context<'_>, 
    #[description = "User to remove from event"] user: serenity::model::user::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let user_id = user.id;
    let db_pool = &ctx.data().db_pool;

    let res = sqlx::query("DELETE FROM khivella_ffevent_records WHERE guild_id = $1 AND discord_id = $2")
        .bind(guild_id.to_string())
        .bind(user_id.to_string())
        .execute(db_pool)
        .await;

    if let Ok(result) = res {
        if result.rows_affected() > 0 {
            send_embed(ctx, "User Removed", &format!("Successfully removed <@{}> from the event registration.", user_id), 0x2ecc71).await?;
        } else {
            send_embed(ctx, "Not Found", &format!("<@{}> hasn't registered for the event yet.", user_id), 0xED4245).await?;
        }
    } else {
        send_embed(ctx, "Error", "Failed to remove user from event data.", 0xED4245).await?;
    }
    Ok(())
}

// ─── VOICE TRACKING (fftrack) ────────────────────────────────────────────────

/// Voice tracking commands. Use subcommand `on` or `off`.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "track",
    category = "Utility",
    guild_only,
    subcommands("fftrack_on", "fftrack_off")
)]
pub async fn fftrack(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enable voice tracking — auto-detects your current voice channel.
#[poise::command(slash_command, prefix_command, rename = "on", category = "Utility", guild_only)]
pub async fn fftrack_on(
    ctx: Context<'_>,
    #[description = "Text channel for logs (defaults to current channel)"] log_channel: Option<serenity::all::ChannelId>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;
    let db_pool = &ctx.data().db_pool;

    // If no log channel specified, use the current text channel
    let log_channel = log_channel.unwrap_or(ctx.channel_id());

    // Extract channel_id from cache synchronously so CacheRef is dropped before any .await
    let voice_channel_id: Option<serenity::model::id::ChannelId> = ctx
        .cache()
        .guild(guild_id)
        .and_then(|g| g.voice_states.get(&author_id).and_then(|vs| vs.channel_id));

    let voice_channel = match voice_channel_id {
        Some(ch) => ch,
        None => {
            send_embed(
                ctx,
                "Not in a Voice Channel",
                "You must be inside a voice channel to enable tracking.\nJoin a voice channel first, then run this command.",
                0xED4245,
            )
            .await?;
            return Ok(());
        }
    };

    let res = sqlx::query(
        "INSERT INTO khivella_vc_tracking (guild_id, voice_channel_id, log_channel_id, is_active)
         VALUES ($1, $2, $3, TRUE)
         ON CONFLICT (guild_id, voice_channel_id) DO UPDATE
         SET log_channel_id = EXCLUDED.log_channel_id, is_active = TRUE"
    )
    .bind(guild_id.to_string())
    .bind(voice_channel.to_string())
    .bind(log_channel.to_string())
    .execute(db_pool)
    .await;

    if res.is_ok() {
        send_embed(
            ctx,
            "Voice Tracking: ON",
            "Voice tracking has been **enabled**.",
            0x10B981,
        )
        .await?;
    } else {
        send_embed(ctx, "Error", "Failed to enable voice tracking.", 0xED4245).await?;
    }
    Ok(())
}

/// Disable voice tracking — auto-detects your current voice channel.
#[poise::command(slash_command, prefix_command, rename = "off", category = "Utility", guild_only)]
pub async fn fftrack_off(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;
    let db_pool = &ctx.data().db_pool;

    // Extract channel_id from cache synchronously so CacheRef is dropped before any .await
    let voice_channel_id: Option<serenity::model::id::ChannelId> = ctx
        .cache()
        .guild(guild_id)
        .and_then(|g| g.voice_states.get(&author_id).and_then(|vs| vs.channel_id));

    let voice_channel = match voice_channel_id {
        Some(ch) => ch,
        None => {
            send_embed(
                ctx,
                "Not in a Voice Channel",
                "You must be inside a voice channel to disable tracking.\nJoin the tracked voice channel first, then run this command.",
                0xED4245,
            )
            .await?;
            return Ok(());
        }
    };

    let res = sqlx::query(
        "UPDATE khivella_vc_tracking SET is_active = FALSE WHERE guild_id = $1 AND voice_channel_id = $2"
    )
    .bind(guild_id.to_string())
    .bind(voice_channel.to_string())
    .execute(db_pool)
    .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => {
            send_embed(
                ctx,
                "Voice Tracking: OFF",
                "Voice tracking has been **disabled**.",
                0xED4245,
            )
            .await?;
        }
        Ok(_) => {
            send_embed(
                ctx,
                "Not Found",
                "No active tracking found for this voice channel.",
                0xf39c12,
            )
            .await?;
        }
        Err(_) => {
            send_embed(ctx, "Error", "Failed to disable voice tracking.", 0xED4245).await?;
        }
    }
    Ok(())
}
