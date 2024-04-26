# glyph-bot

Assistant bot for the Discord servers of all my projects

environment variables:

`DISCORD_TOKEN` (string, required): Discord token
`GLYPH_DATABASE_URL` (string, required): URL for the postgres database
`GLYPH_PG_ENABLE_SSL` (boolean, optional): Whether to enable ssl for postgres connection
`GLYPH_PG_SSL_CERT_PATH` (string, optional): Path to the SSL certificate used for postgres connections, used in addition to the native certificate store. Meaningless if `GLYPH_PG_ENABLE_SSL` is not enabled.
`GLYPH_AIODE_SUPPORT_GUILD_ID` (u64, optional): ID of the aiode support discord server
`GLYPH_AIODE_SUPPORTER_ROLE_ID` (u64, optional): ID of the role rewarded to aiode supporters
`GLYPH_TASK_POOL_WORKER_COUNT` (usize, optional): number of threads in the worker pool used for cron task execution
