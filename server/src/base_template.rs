use serde::Serialize;

use crate::asset_cache::SharedAssetCache;

pub type SharedBaseTemplateData = &'static BaseTemplateData;

#[derive(Clone, Serialize)]
pub struct BaseTemplateData {
    styles: String,
    scripts: String,
}

impl BaseTemplateData {
    pub fn new(assets: SharedAssetCache) -> Self {
        let styles = assets
            .get("index.css")
            .expect("failed to build base template data: index.css")
            .path
            .clone();

        let scripts = assets
            .get("index.js")
            .expect("failed to build base template data: index.js")
            .path
            .clone();

        Self { styles, scripts }
    }

    /// Loads the assets and leaks the allocation, returning a &'static AssetCache.
    pub fn load_static(assets: SharedAssetCache) -> &'static BaseTemplateData {
        let template_data = BaseTemplateData::new(assets);
        Box::leak(Box::new(template_data))
    }
}
