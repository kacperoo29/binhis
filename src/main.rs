mod image;
mod image_component;

use gloo_events::EventListener;
use image_component::ImageComponent;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use yew::prelude::*;

enum Msg {
    FileUpload(Event),
    FileLoaded(Vec<u8>),
}

struct App {
    image_data: Option<Vec<u8>>,
    is_loading: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            image_data: None,
            is_loading: false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div>
                    <input type="file" onchange={ctx.link().callback(|event: Event| Msg::FileUpload(event))} />
                    if self.is_loading {
                        <span>{"Loading image..."}</span>
                    }
                </div>
                if self.image_data.is_some() {
                    <ImageComponent
                        image_data={self.image_data.as_ref().unwrap().clone()}
                    />
                }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileUpload(event) => {
                self.is_loading = true;
                let file_cb = ctx.link().callback(|value: Vec<u8>| Msg::FileLoaded(value));
                let target = event.target().unwrap();
                let target: web_sys::HtmlInputElement = target.dyn_into().unwrap();
                let file = target.files().unwrap().get(0).unwrap();
                let file_reader = web_sys::FileReader::new().unwrap();
                file_reader.read_as_array_buffer(&file).unwrap();
                let listener = EventListener::new(&file_reader, "load", move |event| {
                    let target = event.target().unwrap();
                    let target: web_sys::FileReader = target.dyn_into().unwrap();
                    let result = target.result().unwrap();
                    let array = Uint8Array::new(&result);

                    file_cb.emit(array.to_vec());
                });
                listener.forget();

                true
            }
            Msg::FileLoaded(data) => {
                log::info!("Image loaded");
                self.is_loading = false;
                self.image_data = Some(data);

                true
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
