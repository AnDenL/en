use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct AssetLoader {
    base_path: String,
}

impl AssetLoader {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    pub async fn load_string(&self, path: &str) -> Result<String, String> {
        let full_path = format!("{}{}", self.base_path, path);

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read_to_string(&full_path).map_err(|e| e.to_string())
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;

            let window = web_sys::window().ok_or("No access to window")?;
            let resp_value = JsFuture::from(window.fetch_with_str(&full_path))
                .await
                .map_err(|_| format!("Fetch error for {}", full_path))?;
            
            let resp: web_sys::Response = resp_value.dyn_into().unwrap();
            let text = JsFuture::from(resp.text().unwrap())
                .await
                .map_err(|_| "Unable to read the text".to_string())?;
                
            Ok(text.as_string().unwrap())
        }
    }

    pub async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let full_path = format!("{}{}", self.base_path, path);

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read(&full_path).map_err(|e| e.to_string())
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;

            let window = web_sys::window().ok_or("No access to window")?;
            let resp_value = JsFuture::from(window.fetch_with_str(&full_path))
                .await
                .map_err(|_| format!("Fetch error for {}", full_path))?;
            
            let resp: web_sys::Response = resp_value.dyn_into().unwrap();
            let buffer = JsFuture::from(resp.array_buffer().unwrap())
                .await
                .map_err(|_| "Failed to obtain buffer".to_string())?;
                
            let array = js_sys::Uint8Array::new(&buffer);
            Ok(array.to_vec())
        }
    }

    pub async fn load_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let json_str = self.load_string(path).await?;
        
        serde_json::from_str::<T>(&json_str)
            .map_err(|e| format!("Failed to parse JSON in {}: {}", path, e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_prefs_path() -> std::path::PathBuf {
    std::path::PathBuf::from("prefs.json")
}

pub struct EnPrefs;

impl EnPrefs {
    pub fn set_string(key: &str, value: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = get_prefs_path();
            let mut map: std::collections::HashMap<String, String> = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            map.insert(key.to_string(), value.to_string());
            if let Ok(json) = serde_json::to_string_pretty(&map) {
                let _ = std::fs::write(path, json);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(key, value);
                }
            }
        }
    }

    pub fn get_string(key: &str) -> Option<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = get_prefs_path();
            let map: std::collections::HashMap<String, String> = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            map.get(key).cloned()
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    return storage.get_item(key).unwrap_or(None);
                }
            }
            None
        }
    }
}