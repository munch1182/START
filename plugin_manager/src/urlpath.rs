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

    pub fn curr_part(&self) -> &'a str {
        self.path.last().unwrap_or(&"")
    }

    pub fn host(&self) -> &str {
        self.host
    }

    pub fn push(&mut self, part: &'a str) -> &mut Self {
        self.path.push(part);
        self
    }

    pub fn new_path(&self) -> Self {
        let host = self.host;
        let path = self.path.clone();
        Self { host, path }
    }

    pub fn new_path_with(&self, host: &'a str) -> Self {
        let mut path = self.new_path();
        path.push(host);
        path
    }

    pub fn all_path(&self) -> String {
        self.path
            .iter()
            .fold(self.host.to_string(), |mut acc, part| {
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
        assert_eq!(path.all_path(), "http://127.0.0.1:800/api/v1/");

        let mut rest = path.new_path();
        let rest = rest.push("test").push("test2");
        info!("Rest: {:?}", rest);
        assert_eq!(rest.all_path(), "http://127.0.0.1:800/api/v1/test/test2/");

        path.push("test3");
        info!("Path: {:?}", path);
        assert_eq!(path.all_path(), "http://127.0.0.1:800/api/v1/test3/");

        let last = path.curr_part();
        info!("Last: {:?}", last);
        assert_eq!(last, "test3");
    }
}
