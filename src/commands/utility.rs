use crate::types::{Context, Error};
use crate::utils::embeds::{create_embed, send_embed};
use serenity::model::user::User;

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    send_embed(ctx, "Pong", "System is online and responsive.", 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Utility")]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = match ctx.http().get_guild_with_counts(guild_id).await {
        Ok(g) => g,
        Err(_) => {
            send_embed(ctx, "Error", "Could not fetch server information.", 0xED4245).await?;
            return Ok(());
        }
    };

    let created_timestamp = guild_id.created_at().unix_timestamp();
    
    let channels = ctx.http().get_channels(guild_id).await.unwrap_or_default();
    let text_channels = channels.iter().filter(|c| c.kind == serenity::all::ChannelType::Text).count();
    let voice_channels = channels.iter().filter(|c| c.kind == serenity::all::ChannelType::Voice).count();
    
    let total_emojis = guild.emojis.len();
    let animated_emojis = guild.emojis.values().filter(|e| e.animated).count();
    let static_emojis = total_emojis - animated_emojis;
    
    let member_count = guild.approximate_member_count.unwrap_or(0);
    let online_count = guild.approximate_presence_count.unwrap_or(0);
    
    let tier_str = match guild.premium_tier {
        serenity::all::PremiumTier::Tier1 => "Level 1",
        serenity::all::PremiumTier::Tier2 => "Level 2",
        serenity::all::PremiumTier::Tier3 => "Level 3",
        _ => "None",
    };

    let description = if let Some(desc) = &guild.description {
        format!("*{}*", desc)
    } else {
        "A community server managed by Porpan.".to_string()
    };

    let mut embed = serenity::builder::CreateEmbed::new()
        .title(&guild.name)
        .description(description)
        .color(0xef4444)
        .field("MEMBER DEMOGRAPHICS", format!("Total Members: **{}**\nOnline Members: **{}**", member_count, online_count), false)
        .field("SERVER ARCHITECTURE", format!("Text Channels: **{}**\nVoice Channels: **{}**\nTotal Roles: **{}**", text_channels, voice_channels, guild.roles.len()), false)
        .field("COMMUNITY ASSETS", format!("Total Emojis: **{}** ({} Static, {} Animated)\nBoost Level: **{}** ({} Boosts)", total_emojis, static_emojis, animated_emojis, tier_str, guild.premium_subscription_count.unwrap_or(0)), false)
        .field("CORE INFORMATION", format!("Server ID: `{}`\nEstablished: <t:{}:D>\nServer Owner: <@{}>", guild.id, created_timestamp, guild.owner_id), false)
        .footer(serenity::builder::CreateEmbedFooter::new("4FUN Server Analytics"));

    if let Some(icon) = guild.icon_url() { 
        embed = embed.thumbnail(icon); 
    }
    if let Some(mut banner) = guild.banner_url() { 
        banner = banner.replace("?size=1024", "?size=4096");
        embed = embed.image(banner); 
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn userinfo(
    ctx: Context<'_>, 
    #[description = "User to inspect"] user: Option<User>
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());
    
    let member = if let Some(guild_id) = ctx.guild_id() {
        guild_id.member(ctx.http(), user.id).await.ok()
    } else {
        None
    };
    
    let created_timestamp = user.id.created_at().unix_timestamp();
    
    let embed = serenity::builder::CreateEmbed::new()
        .title(format!("User Information: {}", user.name))
        .color(0xef4444)
        .field("Global Name", user.global_name.as_deref().unwrap_or("None").to_string(), true)
        .field("Identifier", user.id.to_string(), true)
        .field("Creation Date", format!("<t:{}:F>", created_timestamp), false);

    let embed = if let Some(avatar_url) = user.avatar_url() { embed.thumbnail(avatar_url) } else { embed };
    let mut embed = if let Some(mut banner) = user.banner_url() { 
        banner = banner.replace("?size=1024", "?size=4096");
        embed.image(banner) 
    } else { 
        embed 
    };

    if let Some(m) = member {
        if let Some(joined) = m.joined_at {
            embed = embed.field("Network Join Date", format!("<t:{}:F>", joined.unix_timestamp()), false);
        }
        
        let roles = m.roles;
        if !roles.is_empty() {
            let roles_str: Vec<String> = roles.iter().map(|r| format!("<@&{}>", r)).collect();
            let mut joined = roles_str.join(", ");
            if joined.len() > 1000 {
                joined = format!("{}... and more", &joined[0..980]);
            }
            embed = embed.field(format!("Assigned Roles ({})", roles.len()), joined, false);
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn avatar(
    ctx: Context<'_>, 
    #[description = "User to inspect"] user: Option<User>
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());

    let url = user.face(); 
    let mut embed = create_embed(&format!("Avatar: {}", user.name), "", 0x2b2d31);
    embed = embed.image(url);
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility", track_edits)]
pub async fn help(
    ctx: Context<'_>, 
    #[description = "Type 'all' to see all commands including Admin commands"] scope: Option<String>
) -> Result<(), Error> {
    let commands = &ctx.framework().options().commands;
    let show_all = scope.as_deref() == Some("all");
    
    let mut total_commands = 0;
    let mut categories: std::collections::HashMap<&str, Vec<String>> = std::collections::HashMap::new();
    
    for cmd in commands {
        if cmd.hide_in_help { continue; }
        
        let category = cmd.category.as_deref().unwrap_or("Uncategorized");
        
        // Hide Admin & Moderation unless 'all' is passed
        if !show_all && (category == "Admin" || category == "Moderation") {
            continue;
        }

        let desc = cmd.description.as_deref().unwrap_or("No description provided");
        categories.entry(category).or_default().push(format!("`/{}` - {}", cmd.name, desc));
        total_commands += 1;
        
        for subcmd in &cmd.subcommands {
            let sub_desc = subcmd.description.as_deref().unwrap_or("No description provided");
            categories.entry(category).or_default().push(format!("`/{} {}` - {}", cmd.name, subcmd.name, sub_desc));
            total_commands += 1;
        }
    }
    
    let total_categories = categories.len();

    let description = format!(
        "Welcome to the **Porpan Command Center**!\n\n\
        **✦ Quick Guide:**\n\
        - Explore the categories below to discover what I can do.\n\
        - Use `/` in chat to see Discord's native auto-complete.\n\n\
        **✦ System Stats:**\n\
        - Modules Active: `{}`\n\
        - Commands Loaded: `{}`",
        total_categories, total_commands
    );

    let mut embed = serenity::builder::CreateEmbed::new()
        .title("Porpan Help & Documentation")
        .color(0xef4444)
        .description(description);

    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by_key(|(k, _)| *k);

    for (cat, cmds) in sorted_categories {
        let mut current_value = String::new();
        let mut part = 1;
        
        for cmd_str in cmds {
            if current_value.len() + cmd_str.len() + 1 > 1024 {
                let title = if part == 1 { format!("🔹 {}", cat) } else { format!("🔹 {} (Cont.)", cat) };
                embed = embed.field(title, current_value.clone(), false);
                current_value.clear();
                part += 1;
            }
            if !current_value.is_empty() {
                current_value.push('\n');
            }
            current_value.push_str(&cmd_str);
        }
        
        if !current_value.is_empty() {
            let title = if part == 1 { format!("🔹 {}", cat) } else { format!("🔹 {} (Cont.)", cat) };
            embed = embed.field(title, current_value, false);
        }
    }
    
    embed = embed.footer(serenity::builder::CreateEmbedFooter::new("Porpan OS v1.0.0 | Built for 4FUN Clan"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn report(
    ctx: Context<'_>,
    #[description = "User to report"] user: User,
    #[description = "Reason for the report"] reason: String,
) -> Result<(), Error> {
    send_embed(ctx, "Report", &format!("Successfully reported {} for: {}", user.name, reason), 0x2b2d31).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total_memory = sys.total_memory() / 1_048_576; 
    let used_memory = sys.used_memory() / 1_048_576; 
    let cpu_cores = sys.cpus().len();
    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());

    let guild_count = ctx.cache().guilds().len();
    let user_count = ctx.cache().users().len();
    
    let bot_uptime_secs = ctx.data().start_time.elapsed().as_secs();
    let days = bot_uptime_secs / 86400;
    let hours = (bot_uptime_secs % 86400) / 3600;
    let mins = (bot_uptime_secs % 3600) / 60;
    let uptime_str = format!("{}d {}h {}m", days, hours, mins);
    
    let created = ctx.created_at().timestamp_millis();
    let now = serenity::model::Timestamp::now().timestamp_millis();
    let api_latency = format!("{}ms", (now - created).max(0));

    let db_status = "Online (PostgreSQL)";

    let embed = serenity::builder::CreateEmbed::new()
        .title("Porpan System Diagnostics")
        .color(0xef4444)
        .description("Real-time telemetry and resource usage statistics.")
        .field("Developer Identity", "**Author:** phy0n\n**Organization:** 4FUN Clan", false)
        .field("Network Reach", format!("**Servers:** {}\n**Cached Users:** {}\n**API Latency:** {}", guild_count, user_count, api_latency), true)
        .field("Hardware", format!("**OS:** {}\n**CPU Cores:** {}\n**RAM:** {} MB / {} MB", os_name, cpu_cores, used_memory, total_memory), true)
        .field("Core Systems", format!("**Database:** {}\n**Framework:** Poise (Rust)\n**Engine Version:** v1.0.0\n**Uptime:** {}", db_status, uptime_str), false)
        .footer(serenity::builder::CreateEmbedFooter::new("4FUN Core Engine"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let description = "Hello! I am Mbak Porpan, a core member assigned to manage and maintain the 4FUN Clan ecosystem.\n\n\
    My primary role here is to ensure the server remains organized and welcoming. This includes managing new member administration, tracking attendance, and handling your music requests in the voice channels.\n\n\
    While I operate systematically, I am designed to be flexible and friendly. I am also here to accompany you for casual chats or simply to liven up the atmosphere with other 4FUN members. Feel free to say hi!";
    let bot_id = ctx.cache().current_user().id;

    let mut embed = serenity::builder::CreateEmbed::new()
        .title("Mbak Porpan")
        .color(0xef4444)
        .description(description)
        .field("Identitas", "Core Member", true)
        .field("Lokasi Nongkrong", "Surabaya, Indonesia", true)
        .footer(serenity::builder::CreateEmbedFooter::new("Porpan Core Engine • v1.0.0"));

    if let Ok(user) = bot_id.to_user(ctx.http()).await {
        embed = embed.thumbnail(user.face());
        if let Some(mut banner) = user.banner_url() {
            banner = banner.replace("?size=1024", "?size=4096");
            embed = embed.image(banner);
        }
    } else {
        embed = embed.thumbnail(ctx.cache().current_user().face());
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn profile(
    ctx: Context<'_>,
    #[rest]
    #[description = "Discord tag, ID, or Roblox Name"] query: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    
    let clan_data_str = ctx.data().clan_data.read().await.clone();
    
    let mut found_member = None;
    let mut target_discord_id = String::new();
    
    if let Some(q) = &query {
        let q_lower = q.to_lowercase();
        let maybe_id = if q.starts_with("<@") && q.ends_with('>') {
            q.replace("<@", "").replace("!", "").replace(">", "")
        } else {
            q.clone()
        };
        
        if !clan_data_str.is_empty() {
            if let Ok(members) = serde_json::from_str::<serde_json::Value>(&clan_data_str) {
                let members_arr = members.as_array().or_else(|| members["members"].as_array());
                if let Some(arr) = members_arr {
                    for member in arr {
                        let d_id = member["socials"]["discordId"].as_str().unwrap_or("");
                        let r_name = member["name"].as_str().unwrap_or("").to_lowercase();
                        let r_user = member["username"].as_str().unwrap_or("").to_lowercase();
                        
                        if d_id == maybe_id || r_name == q_lower || r_user == q_lower {
                            found_member = Some(member.clone());
                            target_discord_id = d_id.to_string();
                            break;
                        }
                    }
                }
            }
        }
        
        if found_member.is_none() {
            if let Ok(id) = maybe_id.parse::<u64>() {
                target_discord_id = id.to_string();
            } else {
                send_embed(ctx, "Profile", &format!("Tidak menemukan member 4FUN atau Discord User dengan nama/ID: `{}`.", q), 0xED4245).await?;
                return Ok(());
            }
        }
    } else {
        target_discord_id = ctx.author().id.to_string();
        if !clan_data_str.is_empty() {
            if let Ok(members) = serde_json::from_str::<serde_json::Value>(&clan_data_str) {
                let members_arr = members.as_array().or_else(|| members["members"].as_array());
                if let Some(arr) = members_arr {
                    for member in arr {
                        if member["socials"]["discordId"].as_str().unwrap_or("") == target_discord_id {
                            found_member = Some(member.clone());
                            break;
                        }
                    }
                }
            }
        }
    }
    
    let mut discord_user = None;
    let mut discord_member = None;
    
    if let Ok(id) = target_discord_id.parse::<u64>() {
        let user_id = serenity::model::id::UserId::new(id);
        discord_user = ctx.http().get_user(user_id).await.ok();
        
        if let Some(guild_id) = ctx.guild_id() {
            discord_member = guild_id.member(ctx.http(), user_id).await.ok();
        }
    }
    
    let mut roblox_created = String::new();
    let mut roblox_live_bio = String::new();
    
    if let Some(member) = &found_member {
        if let Some(roblox_url) = member["robloxProfile"].as_str() {
            if let Some(id_str) = roblox_url.split("users/").nth(1).and_then(|s| s.split('/').next()) {
                if let Ok(rbx_id) = id_str.parse::<u64>() {
                    let req_url = format!("https://users.roblox.com/v1/users/{}", rbx_id);
                    if let Ok(res) = reqwest::get(&req_url).await {
                        if let Ok(json) = res.json::<serde_json::Value>().await {
                            if let Some(created) = json["created"].as_str() {
                                if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(created) {
                                    roblox_created = format!("<t:{}:D>", parsed_time.timestamp());
                                }
                            }
                            if let Some(bio) = json["description"].as_str() {
                                roblox_live_bio = bio.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut embed = serenity::builder::CreateEmbed::new().color(0xef4444);
    let discord_name = discord_user.as_ref().map(|u| u.name.clone()).unwrap_or_else(|| "Unknown User".to_string());
    
    if let Some(u) = &discord_user {
        embed = embed.thumbnail(u.face());
    }
    
    if let Some(member) = found_member {
        let name = member["name"].as_str().unwrap_or("Unknown");
        let username = member["username"].as_str().unwrap_or("Unknown");
        let roblox_url = member["robloxProfile"].as_str().unwrap_or("");
        let roles = member["roles"].as_array()
            .map(|arr| arr.iter().filter_map(|r| r.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "MEMBER".to_string());
        
        let id = member["id"].as_i64().unwrap_or(0);
        let priority = member["orderPriority"].as_i64().unwrap_or(0);
        
        embed = embed.title(format!("{}'s 4FUN Profile", discord_name));
        
        // Header Description
        let header = format!("` ID: {} ` • ` Priority: {} ` • ` Role: {} `\n", id, priority, roles);
        embed = embed.description(header);
                     
        // Roblox Section
        let mut rbx_info = format!("**Name:** {} (`@{}`)\n**Profile:** [View Profile]({})\n", name, username, roblox_url);
        if !roblox_created.is_empty() {
            rbx_info.push_str(&format!("**Created:** {}\n", roblox_created));
        }
        if !roblox_live_bio.is_empty() {
            let mut trunc_bio = roblox_live_bio.clone();
            if trunc_bio.len() > 100 {
                trunc_bio.truncate(97);
                trunc_bio.push_str("...");
            }
            rbx_info.push_str(&format!("**Bio:** *{}*\n", trunc_bio));
        }
        embed = embed.field("ROBLOX", rbx_info, false);
        
        // Discord Section
        let mut d_info = String::new();
        let d_joined = discord_member.as_ref().and_then(|m| m.joined_at).map(|t| format!("<t:{}:D>", t.unix_timestamp())).unwrap_or_else(|| "Unknown".to_string());
        d_info.push_str(&format!("**Joined Server:** {}\n", d_joined));
        
        if let Some(m) = &discord_member {
            let mut d_roles: Vec<String> = m.roles.iter().map(|r| format!("<@&{}>", r)).collect();
            if !d_roles.is_empty() {
                if d_roles.len() > 5 {
                    d_roles.truncate(5);
                    d_roles.push("...".to_string());
                }
                d_info.push_str(&format!("**Roles:** {}\n", d_roles.join(", ")));
            }
        }
        embed = embed.field("DISCORD", d_info, false);
        
        // Socials Section
        let mut socials = format!("**Discord:** <@{}>\n", target_discord_id);
        if let Some(tiktok) = member["socials"]["tiktok"].as_str() {
            if !tiktok.is_empty() {
                socials.push_str(&format!("**TikTok:** [@{}](https://tiktok.com/@{})\n", tiktok, tiktok));
            }
        }
        embed = embed.field("SOCIALS", socials, false);
        
        if let Some(bio) = member["description"].as_str() {
            if !bio.is_empty() {
                embed = embed.field("WEBSITE BIO", format!("*\"{}\"*", bio), false);
            }
        }
        
        embed = embed.footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Member"));
    } else {
        embed = embed.title(format!("{}'s Profile", discord_name))
                     .description(format!("**Discord ID:** {}\n*This user is not registered in the 4FUN website data.*", target_discord_id));
                     
        if let Some(m) = &discord_member {
            if let Some(joined) = m.joined_at {
                embed = embed.field("Server Join Date", format!("<t:{}:F>", joined.unix_timestamp()), false);
            }
        }
    }
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility", aliases("member"))]
pub async fn members(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let clan_data_str = ctx.data().clan_data.read().await.clone();
    
    if clan_data_str.is_empty() {
        send_embed(ctx, "Members", "Data clan belum tersedia. Coba lagi nanti.", 0xED4245).await?;
        return Ok(());
    }

    let members_data = match serde_json::from_str::<serde_json::Value>(&clan_data_str) {
        Ok(v) => v,
        Err(_) => {
            send_embed(ctx, "Error", "Gagal memproses data clan.", 0xED4245).await?;
            return Ok(());
        }
    };

    let members_arr = members_data.as_array().or_else(|| members_data["members"].as_array());
    if let Some(arr) = members_arr {
        let mut desc = format!("Total Members: **{}**\n\n", arr.len());

        for member in arr {
            let name = member["name"].as_str().unwrap_or("Unknown");
            let username = member["username"].as_str().unwrap_or("Unknown");
            let discord_id = member["socials"]["discordId"].as_str().unwrap_or("");
            
            let line = if discord_id.is_empty() {
                format!("• **{}** (`@{}`)\n", name, username)
            } else {
                format!("• **{}** (`@{}`) - <@{}>\n", name, username, discord_id)
            };
            
            if desc.len() + line.len() > 3950 {
                desc.push_str("...and additional members.");
                break;
            }
            desc.push_str(&line);
        }

        let embed = serenity::builder::CreateEmbed::new()
            .title("4FUN Clan Members")
            .color(0xef4444)
            .description(desc)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Website • https://www.4funclan.site/members"));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        send_embed(ctx, "Error", "Member data not found.", 0xED4245).await?;
    }
    
    Ok(())
}

async fn fetch_roblox_identity(
    client: &reqwest::Client,
    username: &str,
    roblox_profile_url: Option<&str>,
) -> (String, String, Option<u64>) {
    // 1. Try from roblox_profile_url if available
    if let Some(url) = roblox_profile_url {
        if let Some(id_str) = url.split("users/").nth(1).and_then(|s| s.split('/').next()) {
            if let Ok(rbx_id) = id_str.parse::<u64>() {
                let req_url = format!("https://users.roblox.com/v1/users/{}", rbx_id);
                if let Ok(res) = client.get(&req_url).send().await {
                    if let Ok(json) = res.json::<serde_json::Value>().await {
                        let name = json["name"].as_str().unwrap_or(username).to_string();
                        let display = json["displayName"].as_str().unwrap_or(&name).to_string();
                        return (name, display, Some(rbx_id));
                    }
                }
            }
        }
    }

    // 2. Try POST https://users.roblox.com/v1/usernames/users
    let req_body = serde_json::json!({
        "usernames": [username],
        "excludeBannedUsers": false
    });
    if let Ok(res) = client.post("https://users.roblox.com/v1/usernames/users")
        .json(&req_body)
        .send()
        .await
    {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(user_obj) = json["data"].as_array().and_then(|arr| arr.first()) {
                let name = user_obj["name"].as_str().unwrap_or(username).to_string();
                let display = user_obj["displayName"].as_str().unwrap_or(&name).to_string();
                let id = user_obj["id"].as_u64();
                return (name, display, id);
            }
        }
    }

    // 3. Fallback
    (username.to_string(), username.to_string(), None)
}

#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn absen(
    ctx: Context<'_>,
    #[description = "Your registered Roblox username on the 4FUN website"] roblox_username: Option<String>,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            crate::utils::embeds::send_embed(ctx, "Error", "This command can only be used within a server.", 0xED4245).await?;
            return Ok(());
        }
    };
    let db_pool = &ctx.data().db_pool;
    let author_id = ctx.author().id.to_string();
    let author_username = ctx.author().name.clone();

    // Verify author has the required Member role
    let author_member = match ctx.author_member().await {
        Some(m) => m,
        None => {
            crate::utils::embeds::send_embed(ctx, "Error", "Could not retrieve your server member profile.", 0xED4245).await?;
            return Ok(());
        }
    };

    let has_member_role = author_member.roles.iter().any(|r| r.get() == crate::types::ROLE_MEMBER_ID);
    if !has_member_role {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Denied")
            .description("Only clan members holding the Member role are eligible to record monthly attendance.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Discord Member", format!("<@{}>", author_id), true)
            .field("Required Role", format!("<@&{}>", crate::types::ROLE_MEMBER_ID), true)
            .field("Reason", "You must have the Member role in this server to record attendance.", false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // 1. Check if user ran ffabsen without arguments
    let raw_username = match roblox_username {
        Some(ref u) if !u.trim().is_empty() => u.clone(),
        _ => {
            let existing_row = sqlx::query(
                "SELECT discord_id, discord_username, roblox_name, roblox_username, roblox_display_name, role_id, status, timestamp 
                 FROM khivella_absen_records 
                 WHERE guild_id = $1 AND discord_id = $2"
            )
            .bind(guild_id.to_string())
            .bind(&author_id)
            .fetch_optional(db_pool)
            .await;

            if let Ok(Some(row)) = existing_row {
                use sqlx::Row;
                let r_name: String = row.get("roblox_name");
                let r_user: Option<String> = row.try_get("roblox_username").unwrap_or(None);
                let r_disp: Option<String> = row.try_get("roblox_display_name").unwrap_or(None);
                let r_status: String = row.try_get("status").unwrap_or_else(|_| "ACCEPTED".to_string());
                let r_time: Option<chrono::NaiveDateTime> = row.try_get("timestamp").unwrap_or(None);

                let display_val = r_disp.unwrap_or(r_name);
                let username_val = r_user.unwrap_or_else(|| display_val.clone());
                let time_val = r_time.map(|t| format!("<t:{}:F>", t.and_utc().timestamp())).unwrap_or_else(|| "Recorded".to_string());

                let embed = serenity::builder::CreateEmbed::new()
                    .title("Attendance Status")
                    .description("Your monthly attendance has already been recorded.")
                    .color(0x2ecc71)
                    .field("Status", format!("`{}`", r_status), true)
                    .field("Discord Member", format!("<@{}> (`@{}`)", author_id, author_username), true)
                    .field("Discord ID", format!("`{}`", author_id), true)
                    .field("Roblox Username", format!("`@{}`", username_val), true)
                    .field("Roblox Display Name", format!("`{}`", display_val), true)
                    .field("Recorded At", time_val, false)
                    .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            } else {
                let embed = serenity::builder::CreateEmbed::new()
                    .title("Roblox Username Required")
                    .description("Please provide your Roblox username registered on the 4FUN Clan website.\n\n**Usage:**\n`ffabsen <roblox_username>`\n\n*Example:*\n`ffabsen phy0nn`")
                    .color(0x3498db)
                    .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • https://www.4funclan.site/members"));

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    };

    let clean_input = raw_username.trim().trim_start_matches('@');

    // Shortcuts for staff/admin: ffabsen list / ffabsen check / ffabsen purge
    if clean_input.eq_ignore_ascii_case("list") || clean_input.eq_ignore_ascii_case("rekap") {
        if crate::utils::checks::is_staff(ctx).await.unwrap_or(false) {
            return crate::commands::admin::show_absen_list(ctx).await;
        }
    }
    if clean_input.eq_ignore_ascii_case("check") || clean_input.eq_ignore_ascii_case("audit") || clean_input.eq_ignore_ascii_case("review") {
        if crate::utils::checks::is_staff(ctx).await.unwrap_or(false) {
            return crate::commands::admin::show_absen_audit(ctx).await;
        }
    }
    if clean_input.to_lowercase().starts_with("purge") {
        if crate::utils::checks::is_staff(ctx).await.unwrap_or(false) {
            let remainder = clean_input["purge".len()..].trim();
            let user_id_str = remainder.trim_start_matches("<@").trim_start_matches('!').trim_end_matches('>');
            if let Ok(uid) = user_id_str.parse::<u64>() {
                if let Ok(u) = serenity::model::id::UserId::new(uid).to_user(ctx.http()).await {
                    return crate::commands::admin::purge_single_user(ctx, u).await;
                }
            }
            return crate::commands::admin::show_absen_audit(ctx).await;
        }
    }

    // 2. Check if attendance session is currently open
    let state_row = sqlx::query("SELECT is_open, channel_id FROM khivella_absen_state WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .fetch_optional(db_pool)
        .await;

    let (is_open, target_channel_id) = match state_row {
        Ok(Some(row)) => {
            use sqlx::Row;
            let open = row.get::<bool, _>("is_open");
            let cid: Option<String> = row.try_get("channel_id").unwrap_or(None);
            (open, cid)
        },
        _ => (false, None),
    };

    if !is_open {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Closed")
            .description("The monthly attendance session is currently closed.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Discord Member", format!("<@{}>", author_id), true)
            .field("Reason", "Attendance has not been opened by Clan Staff. Please wait for an official announcement.", false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    if let Some(required_channel) = target_channel_id {
        if ctx.channel_id().to_string() != required_channel {
            let embed = serenity::builder::CreateEmbed::new()
                .title("Invalid Channel")
                .description("Attendance is restricted to a designated channel.")
                .color(0xED4245)
                .field("Status", "`DENIED`", true)
                .field("Discord Member", format!("<@{}>", author_id), true)
                .field("Reason", format!("Attendance commands can only be used in <#{}>.", required_channel), false)
                .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }

    // 3. Load clan data from cache, website API, or local fallback
    let mut clan_data_str = ctx.data().clan_data.read().await.clone();
    if clan_data_str.is_empty() {
        let client = reqwest::Client::new();
        if let Ok(res) = client.get("https://www.4funclan.site/api/clan-data").send().await {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if json["success"].as_bool().unwrap_or(false) {
                    let s = serde_json::to_string(&json["data"]).unwrap_or_default();
                    let mut lock = ctx.data().clan_data.write().await;
                    *lock = s.clone();
                    clan_data_str = s;
                }
            }
        }
        if clan_data_str.is_empty() {
            if let Ok(content) = std::fs::read_to_string("../project-4Fun/src/data/members.json") {
                if let Ok(members_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    clan_data_str = serde_json::json!({ "members": members_json }).to_string();
                }
            }
        }
    }

    let parsed: serde_json::Value = serde_json::from_str(&clan_data_str).unwrap_or(serde_json::Value::Null);
    let members_arr = parsed.as_array()
        .or_else(|| parsed["members"].as_array())
        .or_else(|| parsed["data"]["members"].as_array())
        .or_else(|| parsed["data"].as_array());

    let mut matched_member: Option<serde_json::Value> = None;
    if let Some(arr) = members_arr {
        for member in arr {
            let m_username = member["username"].as_str().unwrap_or("");
            let m_name = member["name"].as_str().unwrap_or("");
            if m_username.eq_ignore_ascii_case(clean_input) || m_name.eq_ignore_ascii_case(clean_input) {
                matched_member = Some(member.clone());
                break;
            }
        }
    }

    // 4. Check if registered on website
    let member_data = match matched_member {
        Some(m) => m,
        None => {
            let embed = serenity::builder::CreateEmbed::new()
                .title("Attendance Denied")
                .description("Account not found in the clan database.")
                .color(0xED4245)
                .field("Status", "`DENIED`", true)
                .field("Discord Member", format!("<@{}>", author_id), true)
                .field("Discord ID", format!("`{}`", author_id), true)
                .field("Input Username", format!("`@{}`", clean_input), true)
                .field("Reason", format!("Roblox username `@{}` is not registered on the 4FUN Clan website (https://www.4funclan.site/members). Please check your spelling or contact Clan Staff.", clean_input), false)
                .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • Website Verification"));

            let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_denied_not_found', $1, $2, $3)")
                .bind(&author_id)
                .bind(&author_username)
                .bind(format!("DENIED: Username '{}' not found in website clan data", clean_input))
                .execute(db_pool)
                .await;

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // 5. Check 1-to-1 linkage: Discord ID match ("gabisa absenin orang lain")
    let reg_discord_id = member_data["socials"]["discordId"]
        .as_str()
        .map(|s| s.trim())
        .unwrap_or("");

    if reg_discord_id.is_empty() {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Denied")
            .description("No Discord account is linked to this Roblox user on the website.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Submitted By", format!("<@{}>", author_id), true)
            .field("Your Discord ID", format!("`{}`", author_id), true)
            .field("Roblox Username", format!("`@{}`", member_data["username"].as_str().unwrap_or(clean_input)), true)
            .field("Reason", "This Roblox account does not have a linked Discord ID in the 4FUN Clan database. Please update your profile on the website or contact Clan Staff.", false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • Discord Link Required"));

        let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_denied_no_discord_link', $1, $2, $3)")
            .bind(&author_id)
            .bind(&author_username)
            .bind(format!("DENIED: Roblox '{}' has no Discord ID linked on website", clean_input))
            .execute(db_pool)
            .await;

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    if reg_discord_id != author_id {
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Denied - Proxy Not Allowed")
            .description("You cannot submit attendance for another member's account.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Submitted By", format!("<@{}>", author_id), true)
            .field("Your Discord ID", format!("`{}`", author_id), true)
            .field("Roblox Username", format!("`@{}`", member_data["username"].as_str().unwrap_or(clean_input)), true)
            .field("Registered Discord", format!("<@{}> (`{}`)", reg_discord_id, reg_discord_id), false)
            .field("Reason", "The Discord ID you are currently using does not match the registered Discord ID for this Roblox account on the 4FUN website. Each member must submit their own attendance.", false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • Anti-Proxy Protection"));

        let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_denied_discord_mismatch', $1, $2, $3)")
            .bind(&author_id)
            .bind(&author_username)
            .bind(format!("DENIED: Anti-Proxy triggered! User <@{}> tried to submit Roblox '{}' belonging to Discord ID '{}'", author_id, clean_input, reg_discord_id))
            .execute(db_pool)
            .await;

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // 6. Check duplicates ("1 orang cuma bisa absen 1 akun, jadi gabisa double")
    // 6a. Check if this Discord user has already recorded attendance
    let author_record = sqlx::query(
        "SELECT roblox_username, roblox_name FROM khivella_absen_records WHERE guild_id = $1 AND discord_id = $2"
    )
    .bind(guild_id.to_string())
    .bind(&author_id)
    .fetch_optional(db_pool)
    .await;

    if let Ok(Some(row)) = author_record {
        use sqlx::Row;
        let prev_roblox: String = row.try_get("roblox_username").unwrap_or_else(|_| row.get("roblox_name"));
        
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Denied")
            .description("You have already submitted your attendance for this monthly cycle.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Discord Member", format!("<@{}>", author_id), true)
            .field("Recorded Roblox", format!("`@{}`", prev_roblox), true)
            .field("Reason", "Each member may only submit attendance once per cycle (duplicate submissions are not allowed). Contact Staff if you need assistance.", false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

        let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_denied_duplicate_user', $1, $2, $3)")
            .bind(&author_id)
            .bind(&author_username)
            .bind(format!("DENIED: User already recorded attendance with Roblox '{}'", prev_roblox))
            .execute(db_pool)
            .await;

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // 6b. Check if this Roblox username was already recorded by someone else
    let website_username = member_data["username"].as_str().unwrap_or(clean_input);
    let roblox_record = sqlx::query(
        "SELECT discord_id, discord_username FROM khivella_absen_records 
         WHERE guild_id = $1 AND (LOWER(roblox_username) = LOWER($2) OR LOWER(roblox_name) = LOWER($2))"
    )
    .bind(guild_id.to_string())
    .bind(website_username)
    .fetch_optional(db_pool)
    .await;

    if let Ok(Some(row)) = roblox_record {
        use sqlx::Row;
        let other_discord_id: String = row.get("discord_id");
        
        let embed = serenity::builder::CreateEmbed::new()
            .title("Attendance Denied")
            .description("This Roblox account has already been recorded.")
            .color(0xED4245)
            .field("Status", "`DENIED`", true)
            .field("Discord Member", format!("<@{}>", author_id), true)
            .field("Roblox Username", format!("`@{}`", website_username), true)
            .field("Reason", format!("Roblox account `@{}` was already recorded by <@{}>. Each Roblox account can only be submitted once.", website_username, other_discord_id), false)
            .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System"));

        let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_denied_duplicate_roblox', $1, $2, $3)")
            .bind(&author_id)
            .bind(&author_username)
            .bind(format!("DENIED: Roblox '{}' already submitted by Discord ID '{}'", website_username, other_discord_id))
            .execute(db_pool)
            .await;

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // 7. Retrieve Live Roblox User Info (Username and Display Name)
    let client = reqwest::Client::new();
    let roblox_profile_url = member_data["robloxProfile"].as_str();
    let (mut rbx_username, mut rbx_display_name, _rbx_id) = fetch_roblox_identity(&client, website_username, roblox_profile_url).await;

    if let Some(clan_name) = member_data["name"].as_str() {
        if !clan_name.is_empty() && (rbx_display_name.is_empty() || rbx_display_name == rbx_username) {
            rbx_display_name = clan_name.to_string();
        }
    }
    if rbx_username.is_empty() {
        rbx_username = website_username.to_string();
    }

    // 8. Retrieve Discord Member and Role ID
    let mut primary_role_id: Option<String> = None;

    if let Some(member) = ctx.author_member().await {
        if let Some(guild) = ctx.guild() {
            let mut highest_pos: Option<u16> = None;
            for rid in &member.roles {
                if let Some(r) = guild.roles.get(rid) {
                    if highest_pos.map_or(true, |pos| r.position > pos) {
                        highest_pos = Some(r.position);
                        primary_role_id = Some(rid.to_string());
                    }
                }
            }
        }
    }

    let clan_roles_str = member_data["roles"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>().join(", "))
        .unwrap_or_else(|| "Member".to_string());

    // 9. Save Record to Database (Status: ACCEPTED)
    let res = sqlx::query(
        "INSERT INTO khivella_absen_records (
            guild_id,
            discord_id,
            discord_username,
            roblox_name,
            roblox_username,
            roblox_display_name,
            role_id,
            status,
            timestamp
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'ACCEPTED', CURRENT_TIMESTAMP)"
    )
    .bind(guild_id.to_string())
    .bind(&author_id)
    .bind(&author_username)
    .bind(&rbx_display_name)
    .bind(&rbx_username)
    .bind(&rbx_display_name)
    .bind(&primary_role_id)
    .execute(db_pool)
    .await;

    if res.is_err() {
        crate::utils::embeds::send_embed(ctx, "Error", "Failed to save attendance record to the database. Please contact an Administrator.", 0xED4245).await?;
        return Ok(());
    }

    // 10. Audit log & Response Embed
    let _ = sqlx::query("INSERT INTO khivella_audit_logs (event_type, user_id, username, details) VALUES ('absen_accepted', $1, $2, $3)")
        .bind(&author_id)
        .bind(&author_username)
        .bind(format!("ACCEPTED: Roblox {} (@{}) | Role ID: {:?}", rbx_display_name, rbx_username, primary_role_id))
        .execute(db_pool)
        .await;

    let now_ts = chrono::Utc::now().timestamp();
    let mut embed = serenity::builder::CreateEmbed::new()
        .title("Attendance Recorded")
        .description("Your monthly attendance has been successfully verified and recorded.")
        .color(0x2ecc71)
        .field("Status", "`ACCEPTED`", true)
        .field("Discord Member", format!("<@{}> (`@{}`)", author_id, author_username), true)
        .field("Discord ID", format!("`{}`", author_id), true)
        .field("Roblox Username", format!("`@{}`", rbx_username), true)
        .field("Roblox Display Name", format!("`{}`", rbx_display_name), true)
        .field("Clan Role", format!("`{}`", clan_roles_str), true)
        .field("Timestamp", format!("<t:{}:F>", now_ts), false)
        .footer(serenity::builder::CreateEmbedFooter::new("4FUN Clan Attendance System • Monthly Attendance"));

    embed = embed.thumbnail(ctx.author().face());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility", subcommands("bday_set", "bday_list"), rename = "bday", aliases("birthday"))]
pub async fn bday(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "set", category = "Utility")]
pub async fn bday_set(
    ctx: Context<'_>,
    #[description = "Your birthday in DD-MM or DD-MM-YYYY format (e.g. 15-08 or 15-08-2005)"] date: String,
) -> Result<(), Error> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 2 && parts.len() != 3 {
        crate::utils::embeds::send_embed(ctx, "Invalid Format", "Please use `DD-MM` or `DD-MM-YYYY` format. Example: `15-08` or `15-08-2005`.", 0xED4245).await?;
        return Ok(());
    }

    let day = parts[0].parse::<u32>().unwrap_or(0);
    let month = parts[1].parse::<u32>().unwrap_or(0);
    let mut year: Option<i32> = None;

    if parts.len() == 3 {
        let parsed_year = parts[2].parse::<i32>().unwrap_or(0);
        if parsed_year > 0 {
            // Handle YY format (e.g. 05 -> 2005, 99 -> 1999)
            year = Some(if parsed_year < 100 {
                if parsed_year > 50 { 1900 + parsed_year } else { 2000 + parsed_year }
            } else {
                parsed_year
            });
        }
    }

    if day == 0 || day > 31 || month == 0 || month > 12 {
        crate::utils::embeds::send_embed(ctx, "Invalid Date", "That date doesn't exist! Please provide a valid day and month.", 0xED4245).await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;
    let discord_id = ctx.author().id.to_string();

    let res = sqlx::query("INSERT INTO khivella_birthdays (guild_id, discord_id, day, month, year) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (guild_id, discord_id) DO UPDATE SET day = $3, month = $4, year = $5")
        .bind(guild_id.to_string())
        .bind(discord_id)
        .bind(day as i32)
        .bind(month as i32)
        .bind(year)
        .execute(db_pool)
        .await;

    if res.is_ok() {
        crate::utils::embeds::send_embed(ctx, "Birthday Set!", &format!("Your birthday has been successfully saved as **{}**! 🎂", date), 0x2ecc71).await?;
    } else {
        crate::utils::embeds::send_embed(ctx, "Error", "Failed to save your birthday. Please tell an admin.", 0xED4245).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "list", category = "Utility")]
pub async fn bday_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let rows = sqlx::query("SELECT discord_id, day, month, year FROM khivella_birthdays WHERE guild_id = $1 ORDER BY month ASC, day ASC")
        .bind(guild_id.to_string())
        .fetch_all(db_pool)
        .await;

    match rows {
        Ok(results) => {
            if results.is_empty() {
                crate::utils::embeds::send_embed(ctx, "Birthdays", "No one has set their birthday yet!", 0x3498db).await?;
                return Ok(());
            }

            let mut desc = String::new();
            use sqlx::Row;
            for r in results {
                let discord_id: String = r.get("discord_id");
                let day: i32 = r.get("day");
                let month: i32 = r.get("month");
                let year: Option<i32> = r.try_get("year").unwrap_or(None);
                
                let month_name = match month {
                    1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
                    7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                    _ => "Unknown",
                };
                
                if let Some(y) = year {
                    desc.push_str(&format!("• <@{}> - `{} {} {}`\n", discord_id, day, month_name, y));
                } else {
                    desc.push_str(&format!("• <@{}> - `{} {}`\n", discord_id, day, month_name));
                }
            }
            crate::utils::embeds::send_embed(ctx, "Clan Birthdays 🎂", &desc, 0xf1c40f).await?;
        },
        Err(_) => {
            crate::utils::embeds::send_embed(ctx, "Error", "Failed to load birthdays.", 0xED4245).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Utility", subcommands("ffevent_join"), rename = "ffevent")]
pub async fn ffevent(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "join", category = "Utility")]
pub async fn ffevent_join(
    ctx: Context<'_>,
    #[description = "Your Roblox Display Name (Optional)"] roblox_name: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db_pool = &ctx.data().db_pool;

    let state_row = sqlx::query("SELECT is_open, channel_id FROM khivella_ffevent_state WHERE guild_id = $1")
        .bind(guild_id.to_string())
        .fetch_optional(db_pool)
        .await;

    let (is_open, target_channel_id) = match state_row {
        Ok(Some(row)) => {
            use sqlx::Row;
            let open = row.get::<bool, _>("is_open");
            let cid: Option<String> = row.try_get("channel_id").unwrap_or(None);
            (open, cid)
        },
        _ => (false, None),
    };

    if !is_open {
        crate::utils::embeds::send_embed(ctx, "Event Closed", "Sorry! The event registration is currently closed. Please wait for an admin to open it.", 0xED4245).await?;
        return Ok(());
    }

    if let Some(required_channel) = target_channel_id {
        if ctx.channel_id().to_string() != required_channel {
            crate::utils::embeds::send_embed(ctx, "Invalid Channel", &format!("You can only register for the event in <#{}>.", required_channel), 0xED4245).await?;
            return Ok(());
        }
    }

    let json_data = ctx.data().clan_data.read().await;
    let parsed: serde_json::Value = serde_json::from_str(&json_data).unwrap_or(serde_json::Value::Null);
    let members_array = parsed.as_array().or_else(|| parsed["members"].as_array());

    let mut found = false;
    let mut verified_name = String::new();
    let author_id = ctx.author().id.to_string();

    if let Some(arr) = members_array {
        for member in arr {
            let member_did = member["socials"]["discordId"].as_str().unwrap_or("");
            let member_name = member["name"].as_str().unwrap_or("");
            
            if let Some(ref r_name) = roblox_name {
                if member_name.to_lowercase() == r_name.to_lowercase() {
                    if member_did == author_id {
                        found = true;
                        verified_name = member_name.to_string();
                        break;
                    } else {
                        crate::utils::embeds::send_embed(ctx, "Verification Failed", &format!("The Roblox name `{}` is not linked to your Discord account in our database!", r_name), 0xED4245).await?;
                        return Ok(());
                    }
                }
            } else {
                if member_did == author_id {
                    found = true;
                    verified_name = member_name.to_string();
                    break;
                }
            }
        }
    }

    if !found {
        if let Some(r_name) = roblox_name {
            crate::utils::embeds::send_embed(ctx, "Registration Failed", &format!("The Roblox name `{}` was not found in the 4FUN Clan database.", r_name), 0xED4245).await?;
        } else {
            crate::utils::embeds::send_embed(ctx, "Registration Failed", "Your Discord account was not found in the 4FUN Clan database. Please enter your Roblox name manually or contact an Admin.", 0xED4245).await?;
        }
        return Ok(());
    }

    let discord_id = ctx.author().id.to_string();
    let discord_username = ctx.author().name.clone();

    let res = sqlx::query("INSERT INTO khivella_ffevent_records (guild_id, discord_id, discord_username, roblox_name) VALUES ($1, $2, $3, $4) ON CONFLICT (guild_id, discord_id) DO UPDATE SET roblox_name = $4, timestamp = CURRENT_TIMESTAMP")
        .bind(guild_id.to_string())
        .bind(discord_id)
        .bind(discord_username)
        .bind(verified_name.clone())
        .execute(db_pool)
        .await;

    if res.is_ok() {
        crate::utils::embeds::send_embed(ctx, "Successfully Joined Event!", &format!("Thank you! Your event registration has been successfully recorded:\n\n**Roblox:** `{}`\n**Discord:** <@{}>", verified_name, ctx.author().id), 0x2ecc71).await?;
    } else {
        crate::utils::embeds::send_embed(ctx, "Error", "Failed to save your registration in the database. Please report this to an Admin.", 0xED4245).await?;
    }

    Ok(())
}
