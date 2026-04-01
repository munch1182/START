#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("PluginInfo is invalid")]
    InvalidPluginInfo,
    #[error("Plugin Res Type is not supported")]
    UnSupportResType,
    #[error("UnExist Resource: {0}")]
    UnExistResource(String),
    #[error("Load Plugin Error: {0}")]
    LoadErr(#[from] libloading::Error),
    #[error("Plugin is not found")]
    PluginNotFound,
}
