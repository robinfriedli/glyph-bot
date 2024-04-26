use bigdecimal::BigDecimal;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serenity::{
    all::{Context, EventHandler, GuildMemberUpdateEvent, Member, Ready},
    async_trait,
};

use crate::{
    acquire_db_connection, error::Error, model::NewAiodeSupporter, schema::aiode_supporter,
    AIODE_SUPPORTER_ROLE_ID, AIODE_SUPPORT_GUILD_ID,
};

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        log::info!("Serenity client connected with data {data_about_bot:?}");
    }

    async fn shards_ready(&self, _ctx: Context, total_shards: u32) {
        log::info!("All {total_shards} shards are ready.");
    }

    async fn guild_member_update(
        &self,
        _ctx: Context,
        old_if_available: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        if AIODE_SUPPORT_GUILD_ID.is_some()
            && AIODE_SUPPORTER_ROLE_ID.is_some()
            && event.guild_id == AIODE_SUPPORT_GUILD_ID.unwrap()
        {
            let user_id = event.user.id;
            log::debug!("Received GuildMemberUpdateEvent for user {} on guild {}. Old: {old_if_available:?}, new: {new:?}, event: {event:?}", user_id, event.guild_id);
            if let Err(e) = handle_member_update(old_if_available, new, event).await {
                log::error!(
                    "An error occurred while handling a GuildMemberUpdateEvent for user {}: {e}",
                    user_id
                );
            }
        }
    }
}

async fn handle_member_update(
    old: Option<Member>,
    new: Option<Member>,
    event: GuildMemberUpdateEvent,
) -> Result<(), Error> {
    if old.is_none() || new.is_none() {
        let mut connection = acquire_db_connection().await?;
        if event
            .roles
            .iter()
            .any(|role| *role == AIODE_SUPPORTER_ROLE_ID.unwrap())
        {
            let res = diesel::insert_into(aiode_supporter::table)
                .values(NewAiodeSupporter {
                    user_id: event.user.id.get().into(),
                })
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .await?;

            if res > 0 {
                log::info!(
                    "User {} has been added to the aiode_supporter table",
                    event.user.id
                );
            }
        } else {
            let res = diesel::delete(aiode_supporter::table)
                .filter(aiode_supporter::user_id.eq::<BigDecimal>(event.user.id.get().into()))
                .execute(&mut connection)
                .await?;

            if res > 0 {
                log::info!(
                    "User {} has been removed from the aiode_supporter table",
                    event.user.id
                );
            }
        }
    } else {
        let old_has_role = old
            .unwrap()
            .roles
            .iter()
            .any(|role| *role == AIODE_SUPPORTER_ROLE_ID.unwrap());
        let new_has_role = new
            .unwrap()
            .roles
            .iter()
            .any(|role| *role == AIODE_SUPPORTER_ROLE_ID.unwrap());

        if old_has_role && !new_has_role {
            let mut connection = acquire_db_connection().await?;
            let res = diesel::delete(aiode_supporter::table)
                .filter(aiode_supporter::user_id.eq::<BigDecimal>(event.user.id.get().into()))
                .execute(&mut connection)
                .await?;

            if res > 0 {
                log::info!(
                    "User {} has been removed from the aiode_supporter table",
                    event.user.id
                );
            }
        } else if !old_has_role && new_has_role {
            let mut connection = acquire_db_connection().await?;
            let res = diesel::insert_into(aiode_supporter::table)
                .values(NewAiodeSupporter {
                    user_id: event.user.id.get().into(),
                })
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .await?;

            if res > 0 {
                log::info!(
                    "User {} has been added to the aiode_supporter table",
                    event.user.id
                );
            }
        }
    }

    Ok(())
}
