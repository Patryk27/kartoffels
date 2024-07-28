use anyhow::{anyhow, Result};
use futures::StreamExt;
use glam::ivec2;
use kartoffels::prelude::*;
use serde::ser::Serialize;
use serde_wasm_bindgen::Serializer;
use std::borrow::Cow;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_streams::readable::sys;
use wasm_streams::ReadableStream;

#[wasm_bindgen]
pub struct Sandbox {
    handle: Handle,
}

#[wasm_bindgen]
impl Sandbox {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Sandbox, JsError> {
        let config = serde_wasm_bindgen::from_value(config)
            .map_err(|err| anyhow!("couldn't parse configuration: {:?}", err))
            .convert_err()?;

        let handle =
            kartoffels::create(WorldId::SANDBOX, config, None).convert_err()?;

        Ok(Self { handle })
    }

    pub async fn join(
        &self,
        id: Option<String>,
    ) -> Result<sys::ReadableStream, JsError> {
        let id = id.map(|id| id.parse()).transpose().convert_err()?;

        let stream = self
            .handle
            .join(id)
            .await
            .convert_err()?
            .map(|val| Ok(val.into_js_value()));

        Ok(ReadableStream::from_stream(stream).into_raw())
    }

    pub async fn pause(&self, paused: bool) -> Result<(), JsError> {
        self.handle.pause(paused).await.convert_err()?;

        Ok(())
    }

    pub async fn upload_bot(&self, src: Vec<u8>) -> Result<JsValue, JsError> {
        let id = self
            .handle
            .upload_bot(Cow::Owned(src))
            .await
            .convert_err()?
            .into_js_value();

        Ok(id)
    }

    pub async fn spawn_prefab_bot(
        &self,
        ty: String,
    ) -> Result<JsValue, JsError> {
        let src = match ty.as_str() {
            "roberto" => {
                include_bytes!(env!("KARTOFFELS_ROBERTO"))
            }

            _ => {
                return Err(JsError::new("unknown prefab"));
            }
        };

        let id = self
            .handle
            .upload_bot(Cow::Borrowed(src))
            .await
            .convert_err()?
            .into_js_value();

        Ok(id)
    }

    pub async fn restart_bot(&self, id: String) -> Result<(), JsValue> {
        let id = id.parse().convert_err()?;

        self.handle.restart_bot(id).await.convert_err()?;

        Ok(())
    }

    pub async fn destroy_bot(&self, id: String) -> Result<(), JsValue> {
        let id = id.parse().convert_err()?;

        self.handle.destroy_bot(id).await.convert_err()?;

        Ok(())
    }

    pub async fn set_spawn_point(
        &self,
        x: Option<i32>,
        y: Option<i32>,
    ) -> Result<(), JsValue> {
        let at = if x.is_none() && y.is_none() {
            None
        } else {
            Some(ivec2(x.unwrap_or_default(), y.unwrap_or_default()))
        };

        self.handle.set_spawn_point(at).await.convert_err()?;

        Ok(())
    }
}

trait ConvertErr<T> {
    fn convert_err(self) -> Result<T, JsError>;
}

impl<T> ConvertErr<T> for Result<T> {
    fn convert_err(self) -> Result<T, JsError> {
        self.map_err(|err| JsError::new(&format!("{:?}", err)))
    }
}

trait IntoJsValue {
    fn into_js_value(self) -> JsValue;
}

impl<T> IntoJsValue for T
where
    T: Serialize,
{
    fn into_js_value(self) -> JsValue {
        self.serialize(&Serializer::new().serialize_maps_as_objects(true))
            .unwrap()
    }
}

#[wasm_bindgen(start)]
fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();
}
