use ddsfile::Dds;
use image::ImageOutputFormat;
use image_dds::image_from_dds;
use lazy_static::lazy_static;
use mime_guess::from_ext;
use std::{collections::HashMap, fs::File, io::Cursor, path::Path, sync::Mutex, time::UNIX_EPOCH};
use tiny_http::{Header, Request, Response, StatusCode};
use url::form_urlencoded;

const DDS_IMAGE_CACHE_LIMIT: usize = 64;

#[derive(Clone, Eq, Hash, PartialEq)]
struct DdsImageCacheKey {
    path: String,
    format: String,
    modified_at_millis: u128,
}

#[derive(Clone)]
struct DdsImageCacheValue {
    bytes: Vec<u8>,
    mime_type: String,
}

lazy_static! {
    static ref DDS_IMAGE_CACHE: Mutex<HashMap<DdsImageCacheKey, DdsImageCacheValue>> =
        Mutex::new(HashMap::new());
}

fn modified_at_millis(path: &Path) -> Option<u128> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn dds_image_cache_key(path: &Path, format: &str) -> Option<DdsImageCacheKey> {
    Some(DdsImageCacheKey {
        path: path.to_string_lossy().into_owned(),
        format: format.to_string(),
        modified_at_millis: modified_at_millis(path)?,
    })
}

fn dds_image_response(bytes: Vec<u8>, mime_type: &str) -> Response<Cursor<Vec<u8>>> {
    let content_length = bytes.len();
    Response::new(
        StatusCode(200),
        vec![Header::from_bytes(&b"Content-Type"[..], mime_type).unwrap()],
        Cursor::new(bytes),
        Some(content_length),
        None,
    )
}

pub fn handle_dds_image_request(request: &Request) -> Option<Response<Cursor<Vec<u8>>>> {
    let query_part = request.url().split('?').nth(1)?;

    let params = form_urlencoded::parse(query_part.as_bytes())
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<(String, String)>>();

    let path_param = params.iter().find(|&&(ref key, _)| key == "path")?;
    let format_param = params
        .iter()
        .find(|&&(ref key, _)| key == "format")
        .map(|(_, v)| v.as_str())
        .unwrap_or("jpg");

    let file_path = std::path::Path::new(&path_param.1);
    let normalized_format = format_param.to_lowercase();

    if let Some(cache_key) = dds_image_cache_key(file_path, &normalized_format) {
        if let Some(cached) = DDS_IMAGE_CACHE.lock().unwrap().get(&cache_key).cloned() {
            return Some(dds_image_response(cached.bytes, &cached.mime_type));
        }
    }

    let mut file = File::open(file_path).ok()?;

    let dds = Dds::read(&mut file).ok()?;
    let image = image_from_dds(&dds, 0).ok()?;

    let format = match normalized_format.as_str() {
        "png" => ImageOutputFormat::Png,
        "jpg" | "jpeg" => ImageOutputFormat::Jpeg(80),
        _ => ImageOutputFormat::Jpeg(80),
    };

    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, format).ok()?;

    let bytes = buffer.into_inner();
    let mime_type = from_ext(&normalized_format).first_or_octet_stream();

    if let Some(cache_key) = dds_image_cache_key(file_path, &normalized_format) {
        let mut cache = DDS_IMAGE_CACHE.lock().unwrap();
        if cache.len() >= DDS_IMAGE_CACHE_LIMIT {
            cache.clear();
        }

        cache.insert(
            cache_key,
            DdsImageCacheValue {
                bytes: bytes.clone(),
                mime_type: mime_type.as_ref().to_string(),
            },
        );
    }

    Some(dds_image_response(bytes, mime_type.as_ref()))
}
