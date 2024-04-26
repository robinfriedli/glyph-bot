use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use rusty_pool::ThreadPool;
use serenity::all::{GuildId, RoleId};
use tokio::runtime::Handle;

use crate::{
    acquire_db_connection, error::Error, model::NewAiodeSupporter, schema::aiode_supporter,
    AIODE_SUPPORTER_ROLE_ID, AIODE_SUPPORT_GUILD_ID, DISCORD_TOKEN,
};

lazy_static! {
    pub static ref TASK_POOL: ThreadPool = {
        let task_pool_worker_count = std::env::var("GLYPH_TASK_POOL_WORKER_COUNT")
            .map(|s| {
                s.parse::<usize>()
                    .expect("GLYPH_TASK_POOL_WORKER_COUNT invalid")
            })
            .unwrap_or(4);
        rusty_pool::Builder::new()
            .core_size(task_pool_worker_count)
            .max_size(task_pool_worker_count)
            .name(String::from("task_pool"))
            .build()
    };
    pub static ref RUNNING_TASK_IDS: flurry::HashSet<&'static str> = flurry::HashSet::new();
}

pub fn submit_task(
    task_id: &'static str,
    task: impl Fn(Handle) -> Result<(), Error> + Send + 'static,
) {
    let tokio_handle = Handle::current();
    ThreadPool::execute(&TASK_POOL, move || {
        let running_task_ids = RUNNING_TASK_IDS.pin();
        // only run task if not already running
        if running_task_ids.insert(task_id) {
            let _sentinel = TaskSentinel {
                task_id,
                running_task_ids,
            };

            log::info!("Starting task {task_id}");
            let now = std::time::Instant::now();
            if let Err(e) = task(tokio_handle) {
                log::error!("Error executing task {task_id}: {}", e);
            }
            log::info!("Finished task {task_id} after {:?}", now.elapsed());
        } else {
            log::warn!("Skipping task {task_id} because it is already running")
        }
    })
}

pub fn refresh_aiode_supporters(tokio_handle: Handle) -> Result<(), Error> {
    if AIODE_SUPPORT_GUILD_ID.is_some() && AIODE_SUPPORTER_ROLE_ID.is_some() {
        tokio_handle.block_on(async {
            let serenity_http = serenity::http::Http::new(&DISCORD_TOKEN);

            let guild_id: GuildId = AIODE_SUPPORT_GUILD_ID.unwrap().into();
            let role_id: RoleId = AIODE_SUPPORTER_ROLE_ID.unwrap().into();

            let mut supporters: Vec<u64> = Vec::new();
            let mut last_member = None;
            let limit = 500;

            loop {
                let members = serenity_http
                    .get_guild_members(guild_id, Some(limit), last_member)
                    .await?;

                last_member = members.last().map(|m| m.user.id.into());

                members
                    .iter()
                    .filter(|member| member.roles.contains(&role_id))
                    .for_each(|member| {
                        supporters.push(member.user.id.into());
                    });

                if members.len() < limit as usize {
                    break;
                }
            }

            let supporters = supporters
                .iter()
                .map(|supporter| NewAiodeSupporter {
                    user_id: supporter.into(),
                })
                .collect::<Vec<_>>();
            if !supporters.is_empty() {
                let mut connection = acquire_db_connection().await?;

                // split items into chunks to avoid hitting the parameter limit
                let mut res = 0;
                for supporter_chunk in supporters.chunks(4096) {
                    res += diesel::insert_into(aiode_supporter::table)
                        .values(supporter_chunk)
                        .on_conflict_do_nothing()
                        .execute(&mut connection)
                        .await?;
                }

                if res > 0 {
                    log::info!("Added {} supporters to the aiode_supporter table", res);
                }
            }

            Ok(())
        })
    } else {
        log::warn!("Cannot perform refresh_aiode_supporters because AIODE_SUPPORT_GUILD_ID or AIODE_SUPPORTER_ROLE_ID is not set");
        Ok(())
    }
}

struct TaskSentinel<'a> {
    task_id: &'static str,
    running_task_ids: flurry::HashSetRef<'a, &'static str>,
}

impl Drop for TaskSentinel<'_> {
    fn drop(&mut self) {
        self.running_task_ids.remove(self.task_id);
    }
}
