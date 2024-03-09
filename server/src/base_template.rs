use serde::Serialize;

use crate::asset_cache::SharedAssetCache;

pub type SharedBaseTemplateData = &'static BaseTemplateData;

#[derive(Clone, Serialize)]
pub struct BaseTemplateData {
    styles: String,
    scripts: String,
}

impl BaseTemplateData {
    pub fn new(assets: SharedAssetCache, css: &str, js: &str) -> Self {
        let styles = assets
            .get(css)
            .expect(format!("failed to build base template data: {}", css).as_str())
            .path
            .clone();

        let scripts = assets
            .get(js)
            .expect(format!("failed to build base template data: {}", js).as_str())
            .path
            .clone();

        Self { styles, scripts }
    }

    /// Loads the assets and leaks the allocation, returning a &'static AssetCache.
    pub fn load_static(assets: SharedAssetCache, css: &str, js: &str) -> &'static BaseTemplateData {
        let template_data = BaseTemplateData::new(assets, css, js);
        Box::leak(Box::new(template_data))
    }
}
