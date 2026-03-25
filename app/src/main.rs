use libcommon::{New, newerr, prelude::*};
use plugin::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let a = A::new((0, 2, 4));
    debug!("{:?}", a);

    let result: u8 = a.mock_inject("call_a", a._c).await?;
    let result2: bool = a.mock_inject("call_is_a", &a._a).await?;

    debug!("{result:?}, {result2:?}");
    Ok(())
}

#[derive(New, Debug)]
struct A {
    #[new(default = "A".to_string())]
    _a: String,
    _c: (u8, u8, u8),
}

#[bridge]
impl A {
    async fn call_a(&self, i: (u8, u8, u8)) -> u8 {
        i.1 * 2 + 1
    }

    async fn call_is_a(&self, str: String) -> bool {
        self._a == str
    }

    async fn mock_inject<T: DeserializeOwned, D: Serialize>(
        &self,
        name: impl ToString,
        value: D,
    ) -> Result<T> {
        self.call(json!(Call::new(name, json!(value))))
            .await
            .map_err(|e| newerr!(e))
            .and_then(|s| serde_json::from_value::<T>(s).map_err(|e| newerr!(e)))
    }
}

#[derive(New, Deserialize, Serialize)]
struct Call {
    #[new(from = impl ToString, to = method.to_string())]
    method: String,
    params: Value,
}
