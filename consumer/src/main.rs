mod delay;
mod dto;
mod state;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use eventstore::Client;
use horfimbor_eventsource::cache_db::redis::StateDb;
use horfimbor_eventsource::repository::{DtoRepository, StateRepository};
use std::env;
use template_shared::dto::TemplateDto;
use template_state::TemplateState;

type TemplateStateCache = StateDb<TemplateState>;
type TemplateRepository = StateRepository<TemplateState, TemplateStateCache>;
type TemplateDtoCache = StateDb<TemplateDto>;
type TemplateDtoRepository = DtoRepository<TemplateDto, TemplateDtoCache>;

const STREAM_NAME: &str = "template2";
const GROUP_NAME: &str = "t2";

#[derive(ValueEnum, Clone, Debug)]
enum Consumer {
    Delay,
    Dto,
    State,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    consumer: Consumer,
    #[arg(short, long, default_value_t = false)]
    real_env: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if !args.real_env {
        dotenvy::dotenv().context("cannot get env")?;
    }

    let settings = env::var("EVENTSTORE_URI")
        .context("fail to get EVENTSTORE_URI env var")?
        .parse()
        .context("fail to parse the settings")?;

    let redis_client =
        redis::Client::open(env::var("REDIS_URI").context("fail to get REDIS_URI env var")?)?;

    let event_store_db = Client::new(settings).context("fail to connect to eventstore db")?;

    match args.consumer {
        Consumer::Delay => {
            delay::compute_delay(redis_client, event_store_db).await?;
        }
        Consumer::Dto => {
            dto::cache_dto(redis_client, event_store_db).await?;
        }
        Consumer::State => {
            state::cache_state(redis_client, event_store_db).await?;
        }
    }

    Ok(())
}
