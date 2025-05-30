use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use anyhow::{bail, Context as _, Result};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc};
use photon_rs::native::save_image;
use photon_rs::PhotonImage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::local::{Db, RocksDb},
    sql::Datetime,
    RecordId, Surreal,
};
use tauri::{
    http::{self, request::Request, response::Response, StatusCode},
    AppHandle, Manager, State,
};
use tokio::sync::{mpsc, oneshot};

use super::agent::AgentValue;
use super::{agent::AgentData, tokenize::tokenize_text};
use crate::mnemnk::settings::{data_dir, CoreSettings};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Event {
    kind: String,
    time: Datetime,
    agent: String,
    local_offset: i64,
    local_y: i64,
    local_ym: i64,
    local_ymd: i64,
    value: serde_json::Value,
    text: Option<String>,
    text_tokens: Option<String>,
    // metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: RecordId,
}

// Event types that can be sent to the store event loop
#[derive(Debug)]
enum StoreEvent {
    Backup {
        response: oneshot::Sender<Result<()>>,
    },
    CreateEvent {
        event: Event,
        // response: std::sync::mpsc::Sender<Result<()>>,
    },
    DailyStats {
        response: oneshot::Sender<Result<Vec<DailyStats>>>,
    },
    Delete {
        database: String,
        table: String,
        key: String,
        return_before: bool,
        response: std::sync::mpsc::Sender<Result<Option<serde_json::Value>>>,
    },
    ExportEvents {
        path: String,
        response: oneshot::Sender<Result<()>>,
    },
    FindEventsByYmd {
        year: i32,
        month: i32,
        day: i32,
        response: oneshot::Sender<Result<Vec<EventRecordInternal>>>,
    },
    ImportEvents {
        path: String,
        response: oneshot::Sender<Result<()>>,
    },
    Insert {
        database: String,
        table: String,
        key: String,
        value: serde_json::Value,
        response: std::sync::mpsc::Sender<Result<()>>,
    },
    Query {
        database: String,
        query: String,
        bindings: Option<serde_json::Value>,
        response: std::sync::mpsc::Sender<Result<Vec<serde_json::Value>>>,
    },
    ReindexText {
        response: oneshot::Sender<Result<()>>,
    },
    ReindexYmd {
        day_start_hour: u32,
        response: oneshot::Sender<Result<()>>,
    },
    SearchEvents {
        query: String,
        response: oneshot::Sender<Result<Vec<EventRecordInternal>>>,
    },
    Select {
        database: String,
        table: String,
        key: String,
        response: std::sync::mpsc::Sender<Result<Option<serde_json::Value>>>,
    },
    Shutdown {
        completion: oneshot::Sender<()>,
    },
    Update {
        database: String,
        table: String,
        key: String,
        value: serde_json::Value,
        response: std::sync::mpsc::Sender<Result<()>>,
    },
    UpdateMerge {
        database: String,
        table: String,
        key: String,
        value: serde_json::Value,
        return_after: bool,
        response: std::sync::mpsc::Sender<Result<Option<serde_json::Value>>>,
    },
    Upsert {
        database: String,
        table: String,
        key: String,
        value: serde_json::Value,
        response: std::sync::mpsc::Sender<Result<()>>,
    },
    UpsertMerge {
        database: String,
        table: String,
        key: String,
        value: serde_json::Value,
        return_after: bool,
        response: std::sync::mpsc::Sender<Result<Option<serde_json::Value>>>,
    },
}

pub struct MnemnkDatabase {
    db: Surreal<Db>,
    event_tx: mpsc::Sender<StoreEvent>,
}

const MNEMNK_NS: &str = "mnemnk";
const MNEMNK_DB: &str = "mnemnk";

pub async fn init(app: &AppHandle) -> Result<()> {
    let data_dir = data_dir(app).context("data_dir is not set")?;
    let db_path = PathBuf::from(&data_dir).join("store.db");

    if !db_path.exists() {
        // Check if the restore file exists
        let restore_path = data_dir.join("restore.surql");
        if restore_path.exists() {
            log::info!("Found restore file: {}", restore_path.display());
            restore_from_backup(app, &restore_path).await?;
            std::process::exit(0);
        }
    }

    let db = try_init_database(&db_path).await?;

    let event_tx = start_event_loop(app);

    app.manage(MnemnkDatabase { db, event_tx });

    start_auto_backup(app).await?;

    Ok(())
}

async fn try_init_database(path: &Path) -> Result<Surreal<Db>> {
    let db = Surreal::new::<RocksDb>(path).await?;
    db.use_ns(MNEMNK_NS).use_db(MNEMNK_DB).await?;

    log::info!("store::init: initializing tables");
    let mut resp = db
        .query("DEFINE TABLE IF NOT EXISTS event SCHEMAFULL")
        // fields
        .query("DEFINE FIELD IF NOT EXISTS kind ON TABLE event TYPE string")
        .query("DEFINE FIELD IF NOT EXISTS time ON TABLE event TYPE datetime")
        .query("DEFINE FIELD IF NOT EXISTS local_offset ON TABLE event TYPE int")
        .query("DEFINE FIELD IF NOT EXISTS local_y ON TABLE event TYPE int")
        .query("DEFINE FIELD IF NOT EXISTS local_ym ON TABLE event TYPE int")
        .query("DEFINE FIELD IF NOT EXISTS local_ymd ON TABLE event TYPE int")
        .query("DEFINE FIELD IF NOT EXISTS agent ON TABLE event TYPE string")
        .query("DEFINE FIELD IF NOT EXISTS value ON TABLE event FLEXIBLE TYPE object")
        // index
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYIndex ON TABLE event COLUMNS local_y")
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYmIndex ON TABLE event COLUMNS local_ym")
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYmdIndex ON TABLE event COLUMNS local_ymd")
        .query("DEFINE INDEX IF NOT EXISTS eventKindTimeIndex ON TABLE event COLUMNS kind,time")
        // text
        .query("DEFINE FIELD IF NOT EXISTS text ON TABLE event TYPE option<string>")
        .query("DEFINE FIELD IF NOT EXISTS text_tokens ON TABLE event TYPE option<string>")
        .query("DEFINE ANALYZER IF NOT EXISTS eventTextTokensAnalyzer TOKENIZERS blank")
        .query("DEFINE INDEX IF NOT EXISTS eventTextTokensIndex ON TABLE event COLUMNS text_tokens SEARCH ANALYZER eventTextTokensAnalyzer")
        .await?;
    log::info!("store::init: {:?}", resp);
    if resp.take_errors().len() > 0 {
        bail!("Failed to initialize database.");
    }

    Ok(db)
}

async fn restore_from_backup(app: &AppHandle, restore_file: &Path) -> Result<Surreal<Db>> {
    let data_dir = data_dir(app).context("data_dir is not set")?;
    let db_path = PathBuf::from(&data_dir).join("store.db");

    log::info!("Create a new database at: {}", db_path.display());
    let db = create_new_database(&db_path).await?;

    log::info!("Importing data from: {}", restore_file.display());
    db.import(restore_file).await?;

    log::info!("Database restored successfully");

    Ok(db)
}

async fn create_new_database(db_path: &Path) -> Result<Surreal<Db>> {
    let db = Surreal::new::<RocksDb>(db_path).await?;
    db.use_ns(MNEMNK_NS).use_db(MNEMNK_DB).await?;
    Ok(db)
}

fn start_event_loop(app: &AppHandle) -> mpsc::Sender<StoreEvent> {
    // TODO: the number of events should be configurable
    let (event_tx, mut event_rx) = mpsc::channel::<StoreEvent>(1000);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Store event loop started");

        while let Some(event) = event_rx.recv().await {
            // log::debug!("Store event: {:?}", event);

            match event {
                StoreEvent::Backup { response } => {
                    let result = process_backup(&app_clone).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send backup result");
                    });
                }
                StoreEvent::CreateEvent { event } => {
                    let result = process_create_event(&app_clone, event).await;
                    // response.send(result).unwrap_or_else(|_| {
                    //     log::error!("Failed to send create_event result");
                    // });
                    result.unwrap_or_else(|e| {
                        log::error!("Failed to process_create_event: {}", e);
                    });
                }
                StoreEvent::DailyStats { response } => {
                    let result = process_daily_stats(&app_clone).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send daily stats");
                    });
                }
                StoreEvent::Delete {
                    database,
                    table,
                    key,
                    return_before,
                    response,
                } => {
                    let result =
                        process_delete(&app_clone, database, table, key, return_before).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send delete result: {}", e);
                    });
                }
                StoreEvent::ExportEvents { path, response } => {
                    let result = process_export_events(&app_clone, path).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send export events");
                    });
                }
                StoreEvent::FindEventsByYmd {
                    year,
                    month,
                    day,
                    response,
                } => {
                    let result = process_find_events_by_ymd(&app_clone, year, month, day).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send find events by ymd");
                    });
                }
                StoreEvent::ImportEvents { path, response } => {
                    let result = process_import_events(&app_clone, path).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send import events");
                    });
                }
                StoreEvent::Insert {
                    database,
                    table,
                    key,
                    value,
                    response,
                } => {
                    let result = process_insert(&app_clone, database, table, key, value).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send insert result: {}", e);
                    });
                }
                StoreEvent::Query {
                    database,
                    query,
                    bindings,
                    response,
                } => {
                    let result = process_query(&app_clone, database, query, bindings).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send query result: {}", e);
                    });
                }
                StoreEvent::ReindexText { response } => {
                    let result = process_reindex_text(&app_clone).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send reindex text");
                    });
                }
                StoreEvent::ReindexYmd {
                    day_start_hour,
                    response,
                } => {
                    let result = process_reindex_ymd(&app_clone, day_start_hour).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send reindex ymd");
                    });
                }
                StoreEvent::SearchEvents { query, response } => {
                    let result = process_search_events(&app_clone, query).await;
                    response.send(result).unwrap_or_else(|_| {
                        log::error!("Failed to send search events");
                    });
                }
                StoreEvent::Select {
                    database,
                    table,
                    key,
                    response,
                } => {
                    let result = process_select(&app_clone, database, table, key).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send select result: {}", e);
                    });
                }
                StoreEvent::Shutdown { completion } => {
                    log::info!("Store event loop shutting down");
                    completion.send(()).unwrap_or_else(|_| {
                        log::error!("Failed to send shutdown completion");
                    });
                    break;
                }
                StoreEvent::Update {
                    database,
                    table,
                    key,
                    value,
                    response,
                } => {
                    let result = process_update(&app_clone, database, table, key, value).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send update result: {}", e);
                    });
                }
                StoreEvent::UpdateMerge {
                    database,
                    table,
                    key,
                    value,
                    return_after,
                    response,
                } => {
                    let result =
                        process_update_merge(&app_clone, database, table, key, value, return_after)
                            .await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send update_merge result: {}", e);
                    });
                }
                StoreEvent::Upsert {
                    database,
                    table,
                    key,
                    value,
                    response,
                } => {
                    let result = process_upsert(&app_clone, database, table, key, value).await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send upsert result: {}", e);
                    });
                }
                StoreEvent::UpsertMerge {
                    database,
                    table,
                    key,
                    value,
                    return_after,
                    response,
                } => {
                    let result =
                        process_upsert_merge(&app_clone, database, table, key, value, return_after)
                            .await;
                    response.send(result).unwrap_or_else(|e| {
                        log::error!("Failed to send upsert_merge result: {}", e);
                    });
                }
            }
        }
        log::info!("Store event loop terminated");
    });

    event_tx
}

pub async fn quit(app: &AppHandle) {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    if let Err(e) = state
        .event_tx
        .send(StoreEvent::Shutdown { completion: tx })
        .await
    {
        log::error!("Failed to send shutdown event to store: {}", e);
    }

    // Wait for the event loop to complete (with a timeout)
    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
        Ok(Ok(())) => {
            log::info!("Store shutdown complete");
        }
        Ok(Err(e)) => {
            log::error!("Store shutdown failed: {}", e);
        }
        Err(_) => {
            log::warn!("Store shutdown did not complete within 5 seconds");
        }
    }
}

pub fn delete(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    return_before: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Delete {
        database,
        table,
        key,
        return_before,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive delete result")?
}

async fn process_delete(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    return_before: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;

    if return_before {
        let mut groups = db
            .query("DELETE FROM type::thing($table, $key) RETURN BEFORE")
            .bind(("table", table))
            .bind(("key", key))
            .await?;

        let data: surrealdb::Value = groups.take(0)?;
        let data = data.into_inner().into_json();
        let Some(records) = data.as_array() else {
            return Ok(None);
        };
        let Some(record) = records.get(0) else {
            return Ok(None);
        };

        Ok(Some(serde_json::to_value(record)?))
    } else {
        let _: Option<Record> = db.delete((table, key)).await?;
        Ok(None)
    }
}

pub fn insert(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Insert {
        database,
        table,
        key,
        value,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive insert result")?
}

async fn process_insert(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;
    let _: Option<Record> = db.insert((table, key)).content(value).await?;
    Ok(())
}

pub fn query(
    app: &AppHandle,
    database: String,
    query: String,
    bindings: Option<serde_json::Value>,
) -> Result<Vec<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Query {
        database,
        query,
        bindings,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive query result")?
}

async fn process_query(
    app: &AppHandle,
    database: String,
    query: String,
    bindings: Option<serde_json::Value>,
) -> Result<Vec<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    // use the database
    let db = &state.db;
    db.use_db(database).await?;

    // build a query
    let mut query = db.query(query);
    if let Some(bindings) = bindings {
        for (key, value) in bindings.as_object().context("bindings is not an object")? {
            query = query.bind((key.clone(), value.clone()));
        }
    }

    // execute the query
    let mut groups = query.await?;

    let mut result = Vec::with_capacity(groups.num_statements());
    for i in 0..groups.num_statements() {
        let value: surrealdb::Value = groups.take(i)?;
        let value = value.into_inner().into_json();
        result.push(value);
    }

    Ok(result)
}

pub fn select(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Select {
        database,
        table,
        key,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive select result")?
}

async fn process_select(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;

    // Deserialize the result into a Value doesn't work in SurrealDB 2.x
    // https://github.com/surrealdb/surrealdb/issues/4921
    //
    // let Some(result): Option<surrealdb::sql::Value> = db.select((table, key)).await? else {
    //     return Ok(None);
    // };
    // let value = serde_json::to_value(result)?;
    // Ok(Some(value))

    let mut groups = db
        .query("select * from type::thing($table, $key)")
        .bind(("table", table))
        .bind(("key", key))
        .await?;

    let data: surrealdb::Value = groups.take(0)?;
    let data = data.into_inner().into_json();
    let Some(records) = data.as_array() else {
        return Ok(None);
    };
    let Some(record) = records.get(0) else {
        return Ok(None);
    };

    Ok(Some(serde_json::to_value(record)?))
}

pub fn update(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Update {
        database,
        table,
        key,
        value,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive update result")?
}

async fn process_update(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;
    let _: Option<Record> = db.update((table, key)).content(value).await?;
    Ok(())
}

pub fn update_merge(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
    return_after: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::UpdateMerge {
        database,
        table,
        key,
        value,
        return_after,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive update_merge result")?
}

async fn process_update_merge(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
    return_after: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;

    if return_after {
        let mut groups = db
            .query("UPDATE type::thing($table, $key) MERGE $value RETURN AFTER")
            .bind(("table", table))
            .bind(("key", key))
            .bind(("value", value))
            .await?;

        let data: surrealdb::Value = groups.take(0)?;
        let data = data.into_inner().into_json();
        let Some(records) = data.as_array() else {
            return Ok(None);
        };
        let Some(record) = records.get(0) else {
            return Ok(None);
        };

        Ok(Some(serde_json::to_value(record)?))
    } else {
        let _: Option<Record> = db.update((table, key)).merge(value).await?;
        Ok(None)
    }
}

pub fn upsert(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::Upsert {
        database,
        table,
        key,
        value,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive upsert result")?
}

async fn process_upsert(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;
    let _: Option<Record> = db.upsert((table, key)).content(value).await?;
    Ok(())
}

pub fn upsert_merge(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
    return_after: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = std::sync::mpsc::channel();
    let event = StoreEvent::UpsertMerge {
        database,
        table,
        key,
        value,
        return_after,
        response: tx,
    };

    state.event_tx.try_send(event)?;

    rx.recv().context("Failed to receive upsert_merge result")?
}

async fn process_upsert_merge(
    app: &AppHandle,
    database: String,
    table: String,
    key: String,
    value: serde_json::Value,
    return_after: bool,
) -> Result<Option<serde_json::Value>> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(database).await?;

    if return_after {
        let mut groups = db
            .query("UPSERT type::thing($table, $key) MERGE $value RETURN AFTER")
            .bind(("table", table))
            .bind(("key", key))
            .bind(("value", value))
            .await?;

        let data: surrealdb::Value = groups.take(0)?;
        let data = data.into_inner().into_json();
        let Some(records) = data.as_array() else {
            return Ok(None);
        };
        let Some(record) = records.get(0) else {
            return Ok(None);
        };

        Ok(Some(serde_json::to_value(record)?))
    } else {
        let _: Option<Record> = db.upsert((table, key)).merge(value).await?;
        Ok(None)
    }
}

pub fn create_event(app: &AppHandle, data: AgentData) -> Result<()> {
    let kind = data.kind;
    let Some(mut map) = data.value.as_object().cloned() else {
        return Err(anyhow::anyhow!("store: data is not an object"));
    };

    // extract timestamp from the value if it exists
    let timestamp = if let Some(t) = map.get("t").cloned() {
        // remove timestamp from the value
        map.remove("t");
        t.as_i64().context("wrong timestamp type")?
    } else {
        Utc::now().timestamp_millis()
    };

    let Some(utc_dt) = Utc.timestamp_millis_opt(timestamp).single() else {
        bail!("Failed to parse timestamp: {}", timestamp);
    };
    let local_dt: DateTime<Local> = DateTime::from(utc_dt);
    let local_offset = local_dt.offset().local_minus_utc() as i64;

    let day_start_hour: u32 = {
        let settings = app.state::<Mutex<CoreSettings>>();
        let settings = settings.lock().unwrap();
        settings.day_start_hour.unwrap_or(0)
    };
    let (local_y, local_ym, local_ymd) = adjust_local_ymd(local_dt, day_start_hour);

    // extract text from the value if it exists
    let text = if let Some(t) = map.get("text").cloned() {
        // remove the text from the value
        map.remove("text");
        t.as_str().map(|s| s.to_string())
    } else {
        None
    };
    let text_tokens = text.as_ref().map(|t| tokenize_text(t));

    // extract image from the value if it exists
    if let Some(image) = map.get("image").cloned() {
        // remove image from the value. it's too big to store into the database.
        map.remove("image");

        if let Some(image_id) = map.get("image_id").cloned() {
            let image_id = image_id
                .as_str()
                .context("wrong image_id type")?
                .to_string();

            let app = app.clone();
            let kind = kind.clone();
            tauri::async_runtime::spawn(async move {
                save_image_value(&app, kind, image_id, image)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to save image: {}", e);
                    });
            });
        }
    };

    let value = serde_json::to_value(map).context("Failed to convert value to JSON")?;

    let event = Event {
        kind,
        time: utc_dt.into(),
        agent: "".to_string(),
        local_offset,
        local_y,
        local_ym,
        local_ymd,
        value,
        text,
        text_tokens,
    };

    // log::debug!("store: create_event: {:?}", event);

    let state = app.state::<MnemnkDatabase>();
    // let (tx, rx) = std::sync::mpsc::channel();
    let create_event = StoreEvent::CreateEvent {
        event,
        // response: tx,
    };

    // log::debug!("store: create_event: sending event");

    state.event_tx.try_send(create_event)?;

    // log::debug!("store: create_event: waiting for result");

    // let result = rx.recv()?;

    // log::debug!("store: create_event: result: {:?}", result);

    // result

    Ok(())
}

async fn process_create_event(app: &AppHandle, event: Event) -> Result<()> {
    // log::debug!("store: process_create_event: {:?}", event);

    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;

    db.use_db(MNEMNK_DB)
        .await
        .context("Failed to use database")?;

    let _: Option<Record> = db.create("event").content(event).await?;

    // log::debug!("store: process_create_event: event created");

    Ok(())
}

// Image
async fn save_image_value(
    app: &AppHandle,
    kind: String,
    image_id: String,
    value: AgentValue,
) -> Result<()> {
    let image_dir = image_dir(app, &kind)?;

    let mut image: Option<PhotonImage> = None;
    if value.is_string() {
        let image_str = value
            .as_str()
            .context("image is not a string")?
            .trim_start_matches("data:image/png;base64,");
        image = Some(PhotonImage::new_from_base64(image_str));
    } else if value.is_image() {
        image = value.as_image().cloned();
    }

    let image = image.context("image is not a valid image")?;

    // TODO: check if the image_id is valid
    let ymd = &image_id[0..8];
    let ymd_dir = image_dir.join(ymd);
    if !ymd_dir.exists() {
        std::fs::create_dir(&ymd_dir).context("Failed to create ymd directory")?;
    }

    let filename = &image_id[9..];

    let settings = app.state::<Mutex<CoreSettings>>();
    let thumbnail_width;
    let thumbnail_height;
    {
        let settings = settings.lock().unwrap();
        thumbnail_width = settings.thumbnail_width.clone();
        thumbnail_height = settings.thumbnail_height.clone();
    }
    let thumbnail = make_thumbnail(&image, thumbnail_width, thumbnail_height);

    save_image(image, ymd_dir.join(filename).with_extension("png"))?;
    save_image(thumbnail, ymd_dir.join(filename).with_extension("t.png"))?;

    Ok(())
}

fn image_dir(app: &AppHandle, kind: &str) -> Result<PathBuf> {
    if let Some(data_dir) = data_dir(app) {
        let image_dir = PathBuf::from(data_dir).join(kind).join("image");
        if !image_dir.exists() {
            std::fs::create_dir_all(&image_dir).context("Failed to create image directory")?;
        }
        Ok(image_dir)
    } else {
        Err(anyhow::anyhow!("data_dir is not set"))
    }
}

fn make_thumbnail(image: &PhotonImage, width: Option<u32>, height: Option<u32>) -> PhotonImage {
    let (width, height) = thumbnail_size(image, width, height);

    photon_rs::transform::resize(
        image,
        width,
        height,
        photon_rs::transform::SamplingFilter::Nearest,
    )
}

fn thumbnail_size(image: &PhotonImage, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
    static DEFAULT_THUMBNAIL_HEIGHT: u32 = 36;

    let mut height = height;
    if width.is_some() && height.is_some() {
        return (width.unwrap(), height.unwrap());
    }
    if width.is_none() && height.is_none() {
        height = Some(DEFAULT_THUMBNAIL_HEIGHT);
    }
    if let Some(height) = height {
        let ratio = height as f32 / image.get_height() as f32;
        let width = (image.get_width() as f32 * ratio) as u32;
        return (width, height);
    }
    if let Some(width) = width {
        if let Some(height) = height {
            return (width, height);
        }
        let ratio = width as f32 / image.get_width() as f32;
        let height = (image.get_height() as f32 * ratio) as u32;
        return (width, height);
    }
    // never reach here
    (64, 36)
}

pub fn handle_mimg_protocol(app: &AppHandle, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let uri = request.uri();
    if !check_mimg_path(uri.path()) {
        log::error!("Invalid path: {}", uri.path());
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid uri".into())
            .unwrap();
    }

    // split the uri into type, date directory and filename
    let mut parts = uri.path().split("/").skip(1);
    let kind = parts.next().unwrap_or("");
    let image_id = parts.next().unwrap_or("");
    let date = &image_id[0..8];
    let filename = &image_id[9..];

    if kind.is_empty() || date.is_empty() || filename.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid uri".into())
            .unwrap();
    }

    let screen_dir = image_dir(app, &kind).unwrap(); // TODO: handle error

    let path = screen_dir.join(date).join(format!("{}.png", filename));
    if path.exists() {
        if let Ok(data) = fs::read(path) {
            Response::builder()
                .status(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "image/png")
                .body(data)
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Vec::new())
                .unwrap()
        }
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Invalid uri".into())
            .unwrap()
    }
}

fn check_mimg_path(path: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^/[-_A-Za-z0-9]+/\d{8}-\d{6}(-[A-Za-z0-9]+)?(\.t)?$").unwrap()
    });
    if RE.is_match(path) {
        return true;
    }
    false
}

// #[tauri::command]
// pub async fn load_image_command(app: tauri::AppHandle, event: String, image_id: String) -> Result<Vec<u8>, String> {
//     let rgba_image = load_image(&app, event, image_id).map_err(|e| e.to_string())?;
//     let mut png_data = Vec::new();
//     image::png::PNGEncoder::new(&mut png_data).encode(&rgba_image, rgba_image.width(), rgba_image.height(), image::ColorType::Rgba8).unwrap();
//     Ok(png_data)
// }

// daily stats

#[derive(Debug, Deserialize, Serialize)]
pub struct DailyStats {
    date: i32,
    count: i32,
}

async fn daily_stats(app: &AppHandle) -> Result<Vec<DailyStats>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::DailyStats { response: tx })
        .await?;

    rx.await.context("Failed to receive daily stats")?
}

async fn process_daily_stats(app: &AppHandle) -> Result<Vec<DailyStats>> {
    let sql = r#"
        SELECT
            local_ymd AS date,
            COUNT() AS count
        FROM
            event
        GROUP BY
            date
        ;
    "#;

    let state = app.state::<MnemnkDatabase>();
    let db = state.db.clone();
    db.use_db(MNEMNK_DB).await?;
    let mut result = db.query(sql).await?;
    let daily_stats: Vec<DailyStats> = result.take(0)?;
    Ok(daily_stats)
}

#[tauri::command]
pub async fn daily_stats_cmd(app: AppHandle) -> Result<Vec<DailyStats>, String> {
    let result = daily_stats(&app).await.map_err(|e| e.to_string())?;
    Ok(result)
}

// find events by ymd

#[derive(Debug, Deserialize)]
struct EventRecordInternal {
    id: RecordId,
    kind: String,
    time: Datetime,
    local_offset: i64,
    local_ymd: i64,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventRecord {
    id: String,
    kind: String,
    time: Datetime,
    local_offset: i64,
    local_ymd: i64,
    value: serde_json::Value,
}

async fn find_events_by_ymd(
    app: &AppHandle,
    year: i32,
    month: i32,
    day: i32,
) -> Result<Vec<EventRecordInternal>> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::FindEventsByYmd {
            year,
            month,
            day,
            response: tx,
        })
        .await?;

    rx.await.context("Failed to receive find_events_by_ymd")?
}

async fn process_find_events_by_ymd(
    app: &AppHandle,
    year: i32,
    month: i32,
    day: i32,
) -> Result<Vec<EventRecordInternal>> {
    let sql = r#"
        SELECT 
            id,
            kind,
            time,
            local_offset,
            local_ymd,
            value
        FROM
            event
        WHERE
            local_ymd = $local_ymd
        ORDER BY
            time ASC
        ;
        "#;
    let local_ymd = year * 10000 + month * 100 + day;
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    let mut result = db.query(sql).bind(("local_ymd", local_ymd)).await?;
    let events: Vec<EventRecordInternal> = result.take(0)?;
    Ok(events)
}

#[tauri::command]
pub async fn find_events_by_ymd_cmd(
    app: AppHandle,
    year: i32,
    month: i32,
    day: i32,
) -> Result<Vec<EventRecord>, String> {
    let result = find_events_by_ymd(&app, year, month, day)
        .await
        .map_err(|e| e.to_string())?;
    let events = result
        .iter()
        .map(|e| EventRecord {
            id: e.id.to_string().replace(':', "-"),
            kind: e.kind.clone(),
            time: e.time.clone(),
            local_offset: e.local_offset,
            local_ymd: e.local_ymd,
            value: e.value.clone(),
        })
        .collect();
    Ok(events)
}

// reindex ymd

#[tauri::command]
pub async fn reindex_ymd_cmd(
    app: AppHandle,
    settings: State<'_, Mutex<CoreSettings>>,
) -> Result<(), String> {
    let day_start_hour;
    {
        let settings = settings.lock().unwrap();
        day_start_hour = settings.day_start_hour.unwrap_or(0);
    }
    reindex_ymd(&app, day_start_hour)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct TimestampRecord {
    id: RecordId,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct LocalYmd {
    local_y: i64,
    local_ym: i64,
    local_ymd: i64,
}

async fn reindex_ymd(app: &AppHandle, day_start_hour: u32) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::ReindexYmd {
            day_start_hour,
            response: tx,
        })
        .await?;

    rx.await.context("Failed to receive reindex_ymd")?
}

async fn process_reindex_ymd(app: &AppHandle, day_start_hour: u32) -> Result<()> {
    log::info!("store: reindexing local_ymd...");
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    let mut result = db
        .query("SELECT id, time::unix(time) + local_offset AS timestamp FROM event;")
        .await?;
    let events = result.take::<Vec<TimestampRecord>>(0)?;
    let num_events = events.len();
    let mut i = 0;
    for rec in events {
        i += 1;
        if i % 100 == 0 {
            log::info!("store: indexed {} / {}", i, num_events);
        }
        let dt = DateTime::from_timestamp(rec.timestamp, 0).unwrap_or_default();
        let (local_y, local_ym, local_ymd) = adjust_local_ymd(dt, day_start_hour);
        db.update::<Option<Event>>(rec.id)
            .merge(LocalYmd {
                local_y,
                local_ym,
                local_ymd,
            })
            .await?;
    }
    log::info!("store: reindexed local_ymd");
    Ok(())
}

fn adjust_local_ymd(dt: DateTime<impl TimeZone>, day_start_hour: u32) -> (i64, i64, i64) {
    let adjusted_dt = if dt.hour() < day_start_hour {
        dt - chrono::Duration::days(1)
    } else {
        dt
    };
    let local_y = adjusted_dt.year() as i64;
    let local_ym = local_y * 100 + (adjusted_dt.month() as i64);
    let local_ymd = local_ym * 100 + (adjusted_dt.day() as i64);
    (local_y, local_ym, local_ymd)
}

// search

#[derive(Debug, Deserialize)]
struct TextRecord {
    id: RecordId,
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct TextTokens {
    text_tokens: Option<String>,
}

async fn reindex_text(app: &AppHandle) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::ReindexText { response: tx })
        .await?;

    rx.await.context("Failed to receive reindex_text")?
}

async fn process_reindex_text(app: &AppHandle) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    log::info!("store::init: reindexing text");
    let mut result = db.query("SELECT id, text FROM event").await?;
    let texts = result.take::<Vec<TextRecord>>(0)?;
    let num_texts = texts.len();
    let mut i = 0;
    for rec in texts {
        i += 1;
        if i % 100 == 0 {
            log::info!("store::init: indexed {} / {}", i, num_texts);
        }
        if rec.text.is_none() {
            continue;
        }
        let tokenized_text = tokenize_text(&rec.text.unwrap());
        db.update::<Option<Event>>(rec.id)
            .merge(TextTokens {
                text_tokens: Some(tokenized_text),
            })
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reindex_text_cmd(app: AppHandle) -> Result<(), String> {
    reindex_text(&app).await.map_err(|e| e.to_string())?;
    Ok(())
}

// search events

async fn search_events(app: &AppHandle, query: String) -> Result<Vec<EventRecordInternal>> {
    let tokenized_query = tokenize_text(&query);
    if tokenized_query.is_empty() {
        return Ok(Vec::new());
    }

    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::SearchEvents {
            query: tokenized_query,
            response: tx,
        })
        .await?;

    rx.await.context("Failed to receive search_events")?
}

async fn process_search_events(app: &AppHandle, query: String) -> Result<Vec<EventRecordInternal>> {
    let sql = r#"
        SELECT 
            id,
            kind,
            time,
            local_offset,
            local_ymd,
            value
        FROM
            event
        WHERE
            text_tokens @@ $query
        ORDER BY
            time ASC
        ;
        "#;

    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    let mut result = db.query(sql).bind(("query", query)).await?;
    let events: Vec<EventRecordInternal> = result.take(0)?;
    Ok(events)
}

#[tauri::command]
pub async fn search_events_cmd(app: AppHandle, query: String) -> Result<Vec<EventRecord>, String> {
    let result = search_events(&app, query)
        .await
        .map_err(|e| e.to_string())?;
    let events = result
        .iter()
        .map(|e| EventRecord {
            id: e.id.to_string().replace(':', "-"),
            kind: e.kind.clone(),
            time: e.time.clone(),
            local_offset: e.local_offset,
            local_ymd: e.local_ymd,
            value: e.value.clone(),
        })
        .collect();
    Ok(events)
}

// Export

pub async fn export_events(app: &AppHandle, path: &str) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::ExportEvents {
            path: path.to_string(),
            response: tx,
        })
        .await?;

    rx.await.context("Failed to receive export_events")?
}

pub async fn process_export_events(app: &AppHandle, path: String) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    log::info!("Exporting events to {}", path);

    // Query to get all events with necessary fields (excluding local_y, local_ym, local_ymd, text_tokens)
    let sql = r#"
        SELECT 
            kind,
            time,
            agent,
            local_offset,
            value,
            text
        FROM
            event
        ORDER BY
            time ASC
        ;
    "#;

    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    let mut result = db.query(sql).await?;
    let events: Vec<ExportEventRecord> = result.take(0)?;

    log::info!("Found {} events to export", events.len());

    // Open file for writing
    let file = fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);

    // Write version and timestamp as first line
    #[derive(Serialize)]
    struct Header {
        version: String,
        timestamp: i64,
    }

    let header = Header {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let header_json = serde_json::to_string(&header)?;
    writeln!(writer, "{}", header_json)?;

    // Write each event as a separate line
    for event in events {
        let event_json = serde_json::to_string(&event)?;
        writeln!(writer, "{}", event_json)?;
    }

    writer.flush()?;

    log::info!("Events exported successfully");

    Ok(())
}

// Structure for export
#[derive(Serialize, Deserialize)]
struct ExportEventRecord {
    kind: String,
    time: Datetime,
    agent: String,
    local_offset: i64,
    value: serde_json::Value,
    text: Option<String>,
}

#[tauri::command]
pub async fn export_events_cmd(app: AppHandle, path: String) -> Result<(), String> {
    export_events(&app, &path).await.map_err(|e| e.to_string())
}

// import

pub async fn import_events(app: &AppHandle, path: &str) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    let (tx, rx) = oneshot::channel();
    state
        .event_tx
        .send(StoreEvent::ImportEvents {
            path: path.to_string(),
            response: tx,
        })
        .await?;

    rx.await.context("Failed to receive import_events")?
}

pub async fn process_import_events(app: &AppHandle, path: String) -> Result<()> {
    let state = app.state::<MnemnkDatabase>();

    log::info!("Importing events from {}", path);

    let file = fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    // Parse header from first line
    #[derive(Debug, Deserialize)]
    struct Header {
        version: String,
        timestamp: i64,
    }

    reader.read_line(&mut line)?;
    let header: Header = serde_json::from_str(line.trim())?;

    log::info!(
        "Importing data from export version: {} at {}",
        header.version,
        chrono::DateTime::from_timestamp(header.timestamp, 0)
            .unwrap_or_default()
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    );

    let mut success_count = 0;
    let mut error_count = 0;

    // Get day_start_hour setting, default to 0 if not set
    let day_start_hour = {
        let settings = app.state::<Mutex<CoreSettings>>();
        let settings = settings.lock().unwrap();
        settings.day_start_hour.unwrap_or(0)
    };

    line.clear();
    while reader.read_line(&mut line)? > 0 {
        if line.trim().is_empty() {
            line.clear();
            continue;
        }

        // Parse the event from the current line
        let event: ExportEventRecord = match serde_json::from_str(line.trim()) {
            Ok(event) => event,
            Err(e) => {
                log::error!("Error parsing event: {}", e);
                error_count += 1;
                line.clear();
                continue;
            }
        };

        let dt: DateTime<Utc> = event.time.clone().into();
        let dt_local = DateTime::<Local>::from(dt);
        let (local_y, local_ym, local_ymd) = adjust_local_ymd(dt_local, day_start_hour);

        // Calculate text_tokens if text exists
        let text_tokens = event.text.as_ref().map(|t| tokenize_text(t));

        let record = Event {
            kind: event.kind,
            time: event.time,
            agent: event.agent,
            local_offset: event.local_offset,
            local_y,
            local_ym,
            local_ymd,
            value: event.value,
            text: event.text,
            text_tokens,
        };
        for i in 0..3 {
            match state
                .db
                .create::<Option<Record>>("event")
                .content(record.clone())
                .await
            {
                Ok(_) => {
                    success_count += 1;
                    if success_count % 1000 == 0 {
                        log::info!("Imported {} events", success_count);
                    }
                    break;
                }
                Err(e) => {
                    log::error!("Error importing event: {}", e);
                    if i < 2 {
                        log::info!("Retrying...");
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    } else {
                        log::error!("Failed to import event after 3 attempts");
                        error_count += 1;
                    }
                }
            }
        }

        line.clear();
    }

    if error_count > 0 {
        log::warn!(
            "Import completed with {} successes and {} errors",
            success_count,
            error_count
        );
    } else {
        log::info!(
            "Successfully imported {} events with no errors",
            success_count
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn import_events_cmd(app: AppHandle, path: String) -> Result<(), String> {
    import_events(&app, &path).await.map_err(|e| e.to_string())
}

// Backup

async fn process_backup(app: &AppHandle) -> Result<()> {
    let enable_auto_backup = {
        let settings = app.state::<Mutex<CoreSettings>>();
        let settings = settings.lock().unwrap();
        settings.enable_auto_backup.unwrap_or(true)
    };

    if !enable_auto_backup {
        log::debug!("Auto backup is disabled");
        return Ok(());
    }

    let data_dir = data_dir(app).context("data_dir is not set")?;
    let backup_dir = PathBuf::from(&data_dir).join("backups");

    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir)?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("backup_{}.surql", timestamp));

    let state = app.state::<MnemnkDatabase>();
    let db = &state.db;
    db.use_db(MNEMNK_DB).await?;
    db.export(&backup_path).await?;

    log::info!("Database backup created: {}", backup_path.display());

    cleanup_old_backups(app, &backup_dir)?;

    Ok(())
}

fn cleanup_old_backups(app: &AppHandle, backup_dir: &Path) -> Result<()> {
    let max_backup_count = {
        let settings = app.state::<Mutex<CoreSettings>>();
        let settings = settings.lock().unwrap();
        settings.max_backup_count.unwrap_or(7) as usize
    };

    let mut backups: Vec<_> = std::fs::read_dir(backup_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("surql"))
        .collect();

    // Sort by creation date (oldest first)
    backups.sort_by_key(|entry| {
        entry
            .metadata()
            .and_then(|m| m.created())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    // Delete backups exceeding the maximum count
    if backups.len() > max_backup_count {
        let to_remove = backups.len() - max_backup_count;
        for entry in backups.into_iter().take(to_remove) {
            if let Err(e) = std::fs::remove_file(entry.path()) {
                log::error!("Failed to remove old backup: {}", e);
            } else {
                log::info!("Removed old backup: {}", entry.path().display());
            }
        }
    }

    Ok(())
}

pub async fn start_auto_backup(app: &AppHandle) -> Result<()> {
    let (enable_auto_backup, backup_interval_hours) = {
        let settings = app.state::<Mutex<CoreSettings>>();
        let settings = settings.lock().unwrap();
        (
            settings.enable_auto_backup.unwrap_or(false),
            settings.backup_interval_hours.unwrap_or(24),
        )
    };

    if !enable_auto_backup {
        log::info!("Auto backup is disabled in settings");
        return Ok(());
    }

    let state = app.state::<MnemnkDatabase>();
    let event_tx = state.event_tx.clone();

    let backup_interval = Duration::from_secs(backup_interval_hours as u64 * 3600);

    log::info!(
        "Starting auto backup with interval: {} hours",
        backup_interval_hours
    );

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(backup_interval);
        loop {
            interval.tick().await; // The first tick is completed immediately

            let (tx, rx) = oneshot::channel();
            if let Err(e) = event_tx.send(StoreEvent::Backup { response: tx }).await {
                log::error!("Failed to send backup event: {}", e);
                continue;
            }

            match rx.await {
                Ok(Ok(())) => log::info!("Auto backup completed"),
                Ok(Err(e)) => log::error!("Auto backup failed: {}", e),
                Err(e) => log::error!("Auto backup response error: {}", e),
            }
        }
    });

    Ok(())
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_mimg_uri() {
        // Good
        assert!(check_mimg_path("/screen/20210901-123456"));
        assert!(check_mimg_path("/screen/20210901-123456.t"));
        assert!(check_mimg_path("/screen/20210901-123456-abcdef"));
        assert!(check_mimg_path("/screen/20210901-123456-abcdef.t"));
        // Bad
        assert!(!check_mimg_path("/screen/20210901-123456-abcdef.t-t"));
        assert!(!check_mimg_path("/screen/20210901-123456-abcdef.png"));
        assert!(!check_mimg_path("/screen/20210901-123456-abcdef/"));
        assert!(!check_mimg_path("/screen/20210901-123456-abcdef/abc"));
        assert!(!check_mimg_path("/screen/20210901-123456-abcdef/.."));
        assert!(!check_mimg_path("//screen/20210901-123456-abcdef"));
        assert!(!check_mimg_path("../screen/20210901-123456-abcdef"));
        assert!(!check_mimg_path("/screen/../20210901-123456-abcdef"));
    }
}
