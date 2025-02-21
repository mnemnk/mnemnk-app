use std::sync::{LazyLock, Mutex};

use anyhow::{Context as _, Result};
use base64::{self, Engine as _};
use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use image::RgbaImage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io::Cursor, path::PathBuf};
use surrealdb::{
    engine::local::{Db, RocksDb},
    RecordId, Surreal,
};
use tauri::{
    http::{self, request::Request, response::Response, StatusCode},
    AppHandle, Manager,
};

use crate::mnemnk;

#[derive(Debug, Serialize)]
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
    // metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: RecordId,
}

static DB: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

pub async fn init(app: &AppHandle) -> Result<()> {
    let data_dir = mnemnk::settings::data_dir(app).context("data_dir is not set")?;
    let db_path = PathBuf::from(data_dir).join("store.db");

    DB.connect::<RocksDb>(db_path).await?;
    DB.use_ns("mnemnk").use_db("mnemnk").await?;

    let _result = DB
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
        .query("DEFINE ANALYZER IF NOT EXISTS eventTextAnalyzer TOKENIZERS class,camel FILTERS lowercase,ngram(2,2)")
        .query("DEFINE INDEX IF NOT EXISTS eventTextIndex ON TABLE event COLUMNS text SEARCH ANALYZER eventTextAnalyzer")
        .await?;
    log::info!("store::init: {:?}", _result);

    Ok(())
}

pub fn quit(_app: &AppHandle) {
    // nothing
}

pub async fn store(
    app: &AppHandle,
    agent: String,
    kind: String,
    mut json_value: Value,
) -> Result<()> {
    // extract timestamp from the value if it exists
    let timestamp = if let Some(t) = json_value.get("t").cloned() {
        // remove timestamp from the value
        json_value.as_object_mut().unwrap().remove("t");
        t.as_i64().unwrap()
    } else {
        Utc::now().timestamp_millis()
    };

    let utc_dt = Utc
        .timestamp_millis_opt(timestamp)
        .single()
        .context("Failed to parse timestamp")?;
    let local_dt: DateTime<Local> = DateTime::from(utc_dt);
    let local_offset = local_dt.offset().local_minus_utc() as i64;
    let local_y = local_dt.year() as i64;
    let local_ym = local_y * 100 + (local_dt.month() as i64);
    let local_ymd = local_ym * 100 + (local_dt.day() as i64);

    // extract text from the value if it exists
    let text = if let Some(t) = json_value.get("text").cloned() {
        // remove timestamp from the value
        json_value.as_object_mut().unwrap().remove("text");
        t.as_str().map(|s| s.to_string())
    } else {
        None
    };

    // extract image from the value if it exists
    if let Some(image) = json_value.get("image").cloned() {
        // remove image from the value. it's too big to store into the database.
        json_value.as_object_mut().unwrap().remove("image");
        let image = image.as_str().unwrap().to_string();

        if let Some(image_id) = json_value.get("image_id").cloned() {
            let image_id = image_id.as_str().unwrap().to_string();

            let app = app.clone();
            let kind = kind.to_string();
            tauri::async_runtime::spawn(async move {
                save_image(&app, kind, image_id, image)
                    .await
                    .expect("Failed to save image");
            });
        }
    };

    let record: Option<Record> = DB
        .create("event")
        .content(Event {
            kind,
            time: surrealdb::Datetime::from(utc_dt),
            agent,
            local_offset,
            local_y,
            local_ym,
            local_ymd,
            value: json_value,
            text,
        })
        .await?;

    if record.is_none() {
        return Err(anyhow::anyhow!("Failed to store event"));
    }

    Ok(())
}

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

    let settings = app.state::<Mutex<mnemnk::settings::MnemnkSettings>>();
    let thumbnail_width;
    let thumbnail_height;
    {
        let settings = settings.lock().unwrap();
        thumbnail_width = settings.core.thumbnail_width.clone();
        thumbnail_height = settings.core.thumbnail_height.clone();
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
    if let Some(data_dir) = mnemnk::settings::data_dir(app) {
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

// Index Year

#[derive(Debug, Deserialize)]
struct YmdCount {
    local_ymd: i64,
    count: i64,
}

#[derive(Debug, Serialize)]
pub struct IndexYearResult {
    pub dates: Vec<i32>,
    pub num_events: Vec<i32>,
}

impl From<Vec<YmdCount>> for IndexYearResult {
    fn from(ymd_count: Vec<YmdCount>) -> Self {
        let dates = ymd_count.iter().map(|e| e.local_ymd as i32).collect();
        let num_events = ymd_count.iter().map(|e| e.count as i32).collect();
        IndexYearResult { dates, num_events }
    }
}

async fn do_index_year(year: i32) -> Result<IndexYearResult> {
    let sql = r#"
        SELECT
            local_ymd,
            COUNT() AS count
        FROM
            event
        WHERE
            local_y = $local_y
        GROUP BY
            local_ymd
        ORDER BY
            local_ymd
        ;
    "#;

    let mut result = DB.query(sql).bind(("local_y", year)).await?;
    let ymd_count: Vec<YmdCount> = result.take(0)?;

    Ok(ymd_count.into())
}

#[tauri::command]
pub async fn index_year(year: i32) -> Result<IndexYearResult, String> {
    let result = do_index_year(year).await.map_err(|e| e.to_string())?;
    Ok(result)
}

// find events by ymd

#[derive(Debug, Deserialize)]
struct EventRecordInternal {
    id: RecordId,
    kind: String,
    time: surrealdb::Datetime,
    local_offset: i64,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventRecord {
    id: String,
    kind: String,
    time: surrealdb::Datetime,
    local_offset: i64,
    value: serde_json::Value,
}

async fn find_events_by_ymd(year: i32, month: i32, day: i32) -> Result<Vec<EventRecordInternal>> {
    let sql = r#"
        SELECT 
            id,
            kind,
            time,
            local_offset,
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
    let mut result = DB.query(sql).bind(("local_ymd", local_ymd)).await?;
    let events: Vec<EventRecordInternal> = result.take(0)?;
    Ok(events)
}

#[tauri::command]
pub async fn find_events_by_ymd_cmd(
    year: i32,
    month: i32,
    day: i32,
) -> Result<Vec<EventRecord>, String> {
    let result = find_events_by_ymd(year, month, day)
        .await
        .map_err(|e| e.to_string())?;
    let events = result
        .iter()
        .map(|e| EventRecord {
            id: e.id.to_string().replace(':', "-"),
            kind: e.kind.clone(),
            time: e.time.clone(),
            local_offset: e.local_offset,
            value: e.value.clone(),
        })
        .collect();
    Ok(events)
}

// Search

async fn search_events(query: String) -> Result<Vec<EventRecordInternal>> {
    let sql = r#"
        SELECT 
            id,
            kind,
            time,
            local_offset,
            value
        FROM
            event
        ORDER BY
            time ASC
        WHERE
            text @@ array::join(search::analyze("eventTextAnalyzer", $query), " ")
        ;
        "#;
    let mut result = DB.query(sql).bind(("query", query)).await?;
    let events: Vec<EventRecordInternal> = result.take(0)?;
    Ok(events)
}

#[tauri::command]
pub async fn search_events_cmd(query: String) -> Result<Vec<EventRecord>, String> {
    let result = search_events(query).await.map_err(|e| e.to_string())?;
    let events = result
        .iter()
        .map(|e| EventRecord {
            id: e.id.to_string().replace(':', "-"),
            kind: e.kind.clone(),
            time: e.time.clone(),
            local_offset: e.local_offset,
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
        assert!(check_mimg_path("screen/20210901-123456-abcdef"));
        assert!(check_mimg_path("screen/20210901-123456-abcdef.t"));
        assert!(!check_mimg_path("screen/20210901-123456-abcdef.t-t"));
        assert!(!check_mimg_path("screen/20210901-123456-abcdef.png"));
        assert!(!check_mimg_path("screen/20210901-123456-abcdef/"));
        assert!(!check_mimg_path("screen/20210901-123456-abcdef/abc"));
        assert!(!check_mimg_path("screen/20210901-123456-abcdef/.."));
        assert!(!check_mimg_path("screen//20210901-123456-abcdef"));
        assert!(!check_mimg_path("screen/../20210901-123456-abcdef"));
    }
}
