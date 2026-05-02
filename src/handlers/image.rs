use lazy_static::lazy_static;
use mime_guess::from_path;
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    path::Path,
    sync::Mutex,
    time::UNIX_EPOCH,
};
use tiny_http::{Header, Request, Response, StatusCode};
use url::form_urlencoded;

const IMAGE_CACHE_LIMIT: usize = 128;

#[derive(Clone, Eq, Hash, PartialEq)]
struct ImageCacheKey {
    path: String,
    modified_at_millis: u128,
}

#[derive(Clone)]
struct ImageCacheValue {
    bytes: Vec<u8>,
    mime_type: String,
}

lazy_static! {
    static ref IMAGE_CACHE: Mutex<HashMap<ImageCacheKey, ImageCacheValue>> =
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

fn image_cache_key(path: &Path) -> Option<ImageCacheKey> {
    Some(ImageCacheKey {
        path: path.to_string_lossy().into_owned(),
        modified_at_millis: modified_at_millis(path)?,
    })
}

fn image_response(bytes: Vec<u8>, mime_type: &str) -> Response<Cursor<Vec<u8>>> {
    let content_length = bytes.len();
    Response::new(
        StatusCode(200),
        vec![Header::from_bytes(&b"Content-Type"[..], mime_type).unwrap()],
        Cursor::new(bytes),
        Some(content_length),
        None,
    )
}

/// Return an image at the given `path` query parameter
pub fn handle_image_request(request: &Request) -> Option<Response<Cursor<Vec<u8>>>> {
    let query_part = request.url().split('?').nth(1)?;

    let params = form_urlencoded::parse(query_part.as_bytes())
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<(String, String)>>();

    let path_param = params.iter().find(|&&(ref key, _)| key == "path")?;

    let file_path = std::path::Path::new(&path_param.1);

    if let Some(cache_key) = image_cache_key(file_path) {
        if let Some(cached) = IMAGE_CACHE.lock().unwrap().get(&cache_key).cloned() {
            return Some(image_response(cached.bytes, &cached.mime_type));
        }
    }

    // Guess MIME type
    let mime_type = from_path(&file_path).first_or_octet_stream();

    let mut file = File::open(file_path).ok()?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;

    if let Some(cache_key) = image_cache_key(file_path) {
        let mut cache = IMAGE_CACHE.lock().unwrap();
        if cache.len() >= IMAGE_CACHE_LIMIT {
            cache.clear();
        }

        cache.insert(
            cache_key,
            ImageCacheValue {
                bytes: buf.clone(),
                mime_type: mime_type.as_ref().to_string(),
            },
        );
    }

    Some(image_response(buf, mime_type.as_ref()))
}
