use crate::types::{Context, Error};
use serenity::builder::CreateEmbed;

pub fn create_embed(title: &str, description: &str, color: u32) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
}

pub async fn send_embed(ctx: Context<'_>, title: &str, description: &str, color: u32) -> Result<(), Error> {
    let embed = create_embed(title, description, color);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
