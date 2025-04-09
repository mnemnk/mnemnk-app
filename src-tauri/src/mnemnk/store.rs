use std::sync::{LazyLock, Mutex};

use anyhow::{Context as _, Result};
use base64::{self, Engine as _};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc};
use image::RgbaImage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, io::Cursor, path::PathBuf};
use surrealdb::{
    engine::local::{Db, RocksDb},
    RecordId, Surreal,
};
use tauri::{
    http::{self, request::Request, response::Response, StatusCode},
    AppHandle, Manager, State,
};
use tokio_util::task::TaskTracker;

use super::{agent::AgentData, tokenize::tokenize_text};
use crate::mnemnk::settings::{data_dir, CoreSettings};

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    kind: String,
    time: surrealdb::Datetime,
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

pub struct MnemnkDatabase {
    db: Surreal<Db>,
    tracker: TaskTracker,
}

pub async fn init(app: &AppHandle) -> Result<()> {
    let store = MnemnkDatabase {
        db: Surreal::init(),
        tracker: TaskTracker::new(),
    };

    let data_dir = data_dir(app).context("data_dir is not set")?;
    let db_path = PathBuf::from(data_dir).join("store.db");

    let db = &store.db;

    db.connect::<RocksDb>(db_path).await?;
    db.use_ns("mnemnk").use_db("mnemnk").await?;

    log::info!("store::init: initializing tables");
    let _result = db
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
        // .query("DEFINE FIELD IF NOT EXISTS metadata ON TABLE event FLEXIBLE TYPE option<object>")
        // index
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYIndex ON TABLE event COLUMNS local_y")
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYmIndex ON TABLE event COLUMNS local_ym")
        .query("DEFINE INDEX IF NOT EXISTS eventLocalYmdIndex ON TABLE event COLUMNS local_ymd")
        .query("DEFINE INDEX IF NOT EXISTS eventKindTimeIndex ON TABLE event COLUMNS kind,time")
        // text
        .query("DEFINE FIELD IF NOT EXISTS text ON TABLE event TYPE option<string>")
        .query("REMOVE ANALYZER IF EXISTS eventTextAnalyzer")
        .query("REMOVE INDEX IF EXISTS eventTextIndex ON TABLE event")
        .query("DEFINE FIELD IF NOT EXISTS text_tokens ON TABLE event TYPE option<string>")
        .query("DEFINE ANALYZER IF NOT EXISTS eventTextTokensAnalyzer TOKENIZERS blank")
        .query("DEFINE INDEX IF NOT EXISTS eventTextTokensIndex ON TABLE event COLUMNS text_tokens SEARCH ANALYZER eventTextTokensAnalyzer")
        .await?;
    log::info!("store::init: {:?}", _result);

    app.manage(store);

    Ok(())
}

pub async fn quit(app: &AppHandle) {
    let state = app.state::<MnemnkDatabase>();
    state.tracker.close();
    state.tracker.wait().await;
}

pub fn store(app: &AppHandle, data: AgentData) {
    let state = app.state::<MnemnkDatabase>();

    if state.tracker.is_closed() {
        log::warn!("store: database is closed");
        return;
    }

    let db = state.db.clone();
    let app = app.clone();
    state.tracker.spawn(async move {
        let kind = data.kind;
        let mut json_value = data.value;

        // extract timestamp from the value if it exists
        let timestamp = if let Some(t) = json_value.get("t").cloned() {
            // remove timestamp from the value
            json_value.as_object_mut().unwrap().remove("t");
            t.as_i64().unwrap()
        } else {
            Utc::now().timestamp_millis()
        };

        let Some(utc_dt) = Utc.timestamp_millis_opt(timestamp).single() else {
            log::error!("store: Failed to parse timestamp: {}", timestamp);
            return;
        };
        let local_dt: DateTime<Local> = DateTime::from(utc_dt);
        let local_offset = local_dt.offset().local_minus_utc() as i64;

        let day_start_hour: u32 = {
            let settings = app.state::<Mutex<CoreSettings>>();
            let settings = settings.lock().unwrap();
            settings.day_start_hour.unwrap_or(0)
        };
        let (local_y, local_ym, local_ymd) = adjust_local_ymd(local_dt, day_start_hour);
        // let adjusted_dt = if local_dt.hour() < day_start_hour {
        //     local_dt - chrono::Duration::days(1)
        // } else {
        //     local_dt
        // };

        // let local_y = adjusted_dt.year() as i64;
        // let local_ym = local_y * 100 + (adjusted_dt.month() as i64);
        // let local_ymd = local_ym * 100 + (adjusted_dt.day() as i64);

        // extract text from the value if it exists
        let text = if let Some(t) = json_value.get("text").cloned() {
            // remove the text from the value
            json_value.as_object_mut().unwrap().remove("text");
            t.as_str().map(|s| s.to_string())
        } else {
            None
        };
        let text_tokens = text.as_ref().map(|t| tokenize_text(t));

        // extract image from the value if it exists
        if let Some(image) = json_value.get("image").cloned() {
            // remove image from the value. it's too big to store into the database.
            json_value.as_object_mut().unwrap().remove("image");
            let image = image.as_str().unwrap().to_string();

            if let Some(image_id) = json_value.get("image_id").cloned() {
                let image_id = image_id.as_str().unwrap().to_string();

                let app = app.clone();
                let kind = kind.clone();
                tauri::async_runtime::spawn(async move {
                    save_image(&app, kind, image_id, image)
                        .await
                        .unwrap_or_else(|e| {
                            log::error!("Failed to save image: {}", e);
                        });
                });
            }
        };

        let record: Option<Record> = db
            .create("event")
            .content(Event {
                kind,
                time: surrealdb::Datetime::from(utc_dt),
                agent: "".to_string(),
                local_offset,
                local_y,
                local_ym,
                local_ymd,
                value: json_value,
                text,
                text_tokens,
            })
            .await
            .unwrap_or_default();

        if record.is_none() {
            log::error!("Failed to store event");
        }
    });
}

// Image
async fn save_image(
    app: &AppHandle,
    kind: String,
    image_id: String,
    image_str: String,
) -> Result<()> {
    let image_dir = image_dir(app, &kind)?;

    let base64_str = image_str.trim_start_matches("data:image/png;base64,");
    let rgba_image = base64_to_rgba_image(base64_str)?;

    // TODO: check if the image_id is valid
    let ymd = &image_id[0..8];
    let ymd_dir = image_dir.join(ymd);
    if !ymd_dir.exists() {
        std::fs::create_dir(&ymd_dir).context("Failed to create ymd directory")?;
    }

    let filename = &image_id[9..];
    rgba_image
        .save(ymd_dir.join(filename).with_extension("png"))
        .context("Failed to save image")?;

    let settings = app.state::<Mutex<CoreSettings>>();
    let thumbnail_width;
    let thumbnail_height;
    {
        let settings = settings.lock().unwrap();
        thumbnail_width = settings.thumbnail_width.clone();
        thumbnail_height = settings.thumbnail_height.clone();
    }
    let thumbnail = make_thumbnail(&rgba_image, thumbnail_width, thumbnail_height);
    thumbnail
        .save(ymd_dir.join(filename).with_extension("t.png"))
        .context("Failed to save thumbnail")?;

    Ok(())
}

fn base64_to_rgba_image(base64_str: &str) -> Result<RgbaImage> {
    let png_data = base64::engine::general_purpose::STANDARD.decode(base64_str)?;
    let cursor = Cursor::new(png_data);
    let dynamic_image = image::load_from_memory(&cursor.into_inner())?;
    Ok(dynamic_image.to_rgba8())
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

// fn load_image(app: &AppHandle, event: String, image_id: String) -> Result<RgbaImage> {
//     let image_dir = image_dir(app, &event)?;
//     // TODO: check if the image_id is valid
//     let ymd = &image_id[0..8];
//     let image_id = &image_id[9..];
//     let image_path = image_dir.join(ymd).join(image_id).with_extension("png");

//     let rgba_image = image::open(image_path)?.to_rgba8();

//     Ok(rgba_image)
// }

fn make_thumbnail(image: &RgbaImage, width: Option<u32>, height: Option<u32>) -> RgbaImage {
    let (width, height) = thumbnail_size(image, width, height);
    image::imageops::thumbnail(image, width, height)
}

fn thumbnail_size(image: &RgbaImage, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
    static DEFAULT_THUMBNAIL_HEIGHT: u32 = 36;

    let mut height = height;
    if width.is_some() && height.is_some() {
        return (width.unwrap(), height.unwrap());
    }
    if width.is_none() && height.is_none() {
        height = Some(DEFAULT_THUMBNAIL_HEIGHT);
    }
    if let Some(height) = height {
        let ratio = height as f32 / image.height() as f32;
        let width = (image.width() as f32 * ratio) as u32;
        return (width, height);
    }
    if let Some(width) = width {
        if let Some(height) = height {
            return (width, height);
        }
        let ratio = width as f32 / image.width() as f32;
        let height = (image.height() as f32 * ratio) as u32;
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

    let mut result = state.db.query(sql).await?;
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
    time: surrealdb::Datetime,
    local_offset: i64,
    local_ymd: i64,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventRecord {
    id: String,
    kind: String,
    time: surrealdb::Datetime,
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
    let mut result = state.db.query(sql).bind(("local_ymd", local_ymd)).await?;
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
    log::info!("store: reindexing local_ymd...");
    let state = app.state::<MnemnkDatabase>();
    let db = state.db.clone();
    state.tracker.spawn(async move {
        let Ok(mut result) = db
            .query("SELECT id, time::unix(time) + local_offset AS timestamp FROM event;")
            .await
        else {
            log::error!("store::reindex_ymd: failed to query events");
            return;
        };

        let Ok(events) = result.take::<Vec<TimestampRecord>>(0) else {
            log::error!("store::reindex_ymd: no results");
            return;
        };
        let num_events = events.len();
        let mut i = 0;
        for rec in events {
            i += 1;
            if i % 100 == 0 {
                log::info!("store: indexed {} / {}", i, num_events);
            }
            let dt = DateTime::from_timestamp(rec.timestamp, 0).unwrap_or_default();
            let (local_y, local_ym, local_ymd) = adjust_local_ymd(dt, day_start_hour);
            if let Err(e) = db
                .update::<Option<Event>>(rec.id)
                .merge(LocalYmd {
                    local_y,
                    local_ym,
                    local_ymd,
                })
                .await
            {
                log::error!("store::reindex_ymd: failed to update local_ymd: {}", e);
                return;
            }
        }
        log::info!("store: reindexed local_ymd");
    });
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

async fn reindex_text(app: &AppHandle) {
    let state = app.state::<MnemnkDatabase>();
    let db = state.db.clone();
    state.tracker.spawn(async move {
        log::info!("store::init: reindexing text");
        let Ok(mut result) = db.query("SELECT id, text FROM event").await else {
            log::error!("store::reindex_text: failed to query events");
            return;
        };
        let Ok(texts) = result.take::<Vec<TextRecord>>(0) else {
            log::error!("store::reindex_text: no results");
            return;
        };
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
            if let Err(e) = db
                .update::<Option<Event>>(rec.id)
                .merge(TextTokens {
                    text_tokens: Some(tokenized_text),
                })
                .await
            {
                log::error!("store::reindex_text: failed to update text_tokens: {}", e);
                return;
            }
        }
    });
}

#[tauri::command]
pub async fn reindex_text_cmd(app: AppHandle) -> Result<(), String> {
    reindex_text(&app).await;
    Ok(())
}

async fn search_events(app: &AppHandle, query: String) -> Result<Vec<EventRecordInternal>> {
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
    let tokenized_query = tokenize_text(&query);
    if tokenized_query.is_empty() {
        return Ok(Vec::new());
    }
    let state = app.state::<MnemnkDatabase>();
    let mut result = state.db.query(sql).bind(("query", tokenized_query)).await?;
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
