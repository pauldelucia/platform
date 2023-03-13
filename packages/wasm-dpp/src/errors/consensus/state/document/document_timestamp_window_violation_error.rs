use crate::buffer::Buffer;
use dpp::{identifier::Identifier, prelude::TimestampMillis};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=DocumentTimestampWindowViolationError)]
pub struct DocumentTimestampWindowViolationErrorWasm {
    timestamp_name: String,
    document_id: Identifier,
    timestamp: TimestampMillis,
    time_window_start: TimestampMillis,
    time_window_end: TimestampMillis,
    code: u32,
}

#[wasm_bindgen(js_class=DocumentTimestampWindowViolationError)]
impl DocumentTimestampWindowViolationErrorWasm {
    #[wasm_bindgen(js_name=getDocumentId)]
    pub fn document_id(&self) -> Buffer {
        Buffer::from_bytes(self.document_id.as_bytes())
    }

    #[wasm_bindgen(js_name=getTimestampName)]
    pub fn timestamp_name(&self) -> String {
        self.timestamp_name.clone()
    }

    #[wasm_bindgen(js_name=getTimestamp)]
    pub fn timestamp(&self) -> js_sys::Date {
        let date = js_sys::Date::new_0();
        date.set_time(self.timestamp as f64);
        date
    }

    #[wasm_bindgen(js_name=getTimeWindowStart)]
    pub fn time_window_start(&self) -> js_sys::Date {
        let date = js_sys::Date::new_0();
        date.set_time(self.time_window_start as f64);
        date
    }

    #[wasm_bindgen(js_name=getTimeWindowEnd)]
    pub fn time_window_end(&self) -> js_sys::Date {
        let date = js_sys::Date::new_0();
        date.set_time(self.time_window_end as f64);
        date
    }

    #[wasm_bindgen(js_name=getCode)]
    pub fn get_code(&self) -> u32 {
        self.code
    }
}

impl DocumentTimestampWindowViolationErrorWasm {
    pub fn new(
        timestamp_name: String,
        document_id: Identifier,
        timestamp: TimestampMillis,
        time_window_start: TimestampMillis,
        time_window_end: TimestampMillis,
        code: u32,
    ) -> Self {
        Self {
            timestamp_name,
            document_id,
            timestamp,
            time_window_start,
            time_window_end,
            code,
        }
    }
}
