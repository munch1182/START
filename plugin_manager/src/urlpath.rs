#[derive(Clone)]
pub struct UrlPath<'a> {
    host: &'a str,
    path: Vec<&'a str>,
}

impl std::fmt::Debug for UrlPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.all_path())
    }
}

impl<'a> UrlPath<'a> {
    pub fn new(host: &'a str) -> Self {
        Self { host, path: vec![] }
    }

    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let mut path = UrlPath::new_with_path("http://127.0.0.1:8000", &["api", "v1"]);
    ///
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// ```
    pub fn new_with_path(host: &'a str, path: &[&'a str]) -> Self {
        Self {
            host,
            path: path.to_vec(),
        }
    }

    ///
    /// 返回最后传入的路径
    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let mut path = UrlPath::new("http://127.0.0.1:8000");
    /// path.push("api").push("v1");
    ///
    /// assert_eq!(path.curr_part().unwrap_or_default(), "v1");
    /// ```
    ///
    pub fn curr_part(&self) -> Option<&'a str> {
        self.path.last().copied()
    }

    pub fn router_str(&self) -> &str {
        self.curr_part().unwrap_or_default()
    }
    ///
    /// 返回传入的host
    ///
    pub fn host(&self) -> &str {
        self.host
    }

    ///
    /// 返回上一级的路径
    ///
    pub fn parent(&self) -> Self {
        let mut new = self.new_path();
        new.path.pop();
        new
    }

    ///
    /// 添加一段路径
    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let mut path = UrlPath::new("http://127.0.0.1:8000");
    /// path.push("api").push("v1");
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// ```
    pub fn push(&mut self, part: &'a str) -> &mut Self {
        self.path.push(part);
        self
    }

    ///
    /// 复制当前的值到一个新的UrlPath
    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let path = UrlPath::new_with_path("http://127.0.0.1:8000", &["api", "v1"]);
    /// let mut path2 = path.new_path();
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// assert_eq!(path2.all_path(), "http://127.0.0.1:8000/api/v1");
    /// path2.push("test");
    /// assert_eq!(path2.all_path(), "http://127.0.0.1:8000/api/v1/test");
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// ```
    pub fn new_path(&self) -> Self {
        let host = self.host;
        let path = self.path.clone();
        Self { host, path }
    }

    ///
    /// 复制当前的值到一个新的UrlPath，并添加路径
    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let path = UrlPath::new_with_path("http://127.0.0.1:8000", &["api", "v1"]);
    /// let mut path2 = path.new_path_with("test");
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// assert_eq!(path2.all_path(), "http://127.0.0.1:8000/api/v1/test");
    /// ```
    pub fn new_path_with(&self, host: &'a str) -> Self {
        let mut path = self.new_path();
        path.path.push(host);
        path
    }

    ///
    /// 复制当前的值到一个新的UrlPath，并添加路径
    ///
    /// ```
    /// use plugin_manager::urlpath::UrlPath;
    /// let path = UrlPath::new_with_path("http://127.0.0.1:8000", &["api", "v1"]);
    /// let mut path2 = path.new_path_with_slice(&["test"]);
    /// assert_eq!(path.all_path(), "http://127.0.0.1:8000/api/v1");
    /// assert_eq!(path2.all_path(), "http://127.0.0.1:8000/api/v1/test");
    /// ```
    pub fn new_path_with_slice(&self, host: &[&'a str]) -> Self {
        let mut path = self.new_path();
        path.path.extend_from_slice(host);
        path
    }

    ///
    /// 返回完整的url
    ///
    pub fn all_path(&self) -> String {
        self.path
            .iter()
            .fold(self.host.to_string(), |mut acc, part| {
                if !acc.ends_with('/') && !part.starts_with('/') {
                    acc.push('/');
                }
                acc.push_str(part);
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{log::log_setup, prelude::info};

    #[test]
    fn test_path() {
        log_setup();
        let mut path = UrlPath::new("http://127.0.0.1:800");
        let path: &mut UrlPath<'_> = path.push("api").push("v1");
        info!("Path: {:?}", path);
        assert_eq!(path.all_path(), "http://127.0.0.1:800/api/v1");

        let mut rest = path.new_path();
        let rest = rest.push("test").push("test2");
        info!("Rest: {:?}", rest);
        assert_eq!(rest.all_path(), "http://127.0.0.1:800/api/v1/test/test2");

        path.push("test3");
        info!("Path: {:?}", path);
        assert_eq!(path.all_path(), "http://127.0.0.1:800/api/v1/test3");

        let last = path.curr_part();
        info!("Last: {:?}", last);
        assert_eq!(last, Some("test3"));

        let last = path.parent();
        info!("Path: {:?}", last);
        assert_eq!(last.all_path(), "http://127.0.0.1:800/api/v1");
    }
}
