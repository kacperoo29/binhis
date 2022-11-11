use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlInputElement, ImageData};
use yew::{html, Component, NodeRef, Properties};

use crate::image::Image;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub image_data: Vec<u8>,
}

pub enum Msg {
    StretchHistogram,
    EqualizeHistogram,
    ApplyThreshold,
    ApplyMeanIterativeSelectionThreshold,
    ApplyPercentBlackSelectionThreshold,
    ApplyEntropySelectionThreshold,
    ApplyMinimumErrorThreshold,
    ApplyFuzzyMinimumErrorThreshold,
    TresholdLowChanged(Event),
    TresholdHighChanged(Event),
    PercentBlackChanged(Event)
}

pub struct ImageComponent {
    image: Image,
    image_to_display: Image,
    canvas_ref: NodeRef,
    canvas_ctx: Option<CanvasRenderingContext2d>,
    treshold_low: u8,
    treshold_high: u8,
    black_percent: f32,
}

impl Component for ImageComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let image = Image::new_with_data(ctx.props().image_data.clone());
        Self {
            image: image.clone(),
            image_to_display: image,
            canvas_ref: NodeRef::default(),
            canvas_ctx: None,
            treshold_low: 0,
            treshold_high: 255,
            black_percent: 0.0,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let link = ctx.link();

        html! {
            <>
                <div>
                    <button onclick={link.callback(|_| Msg::StretchHistogram )}>{"Normalize (stretch histogram)"}</button>
                    <button onclick={link.callback(|_| Msg::EqualizeHistogram )}>{"Normalize (equalize histogram)"}</button>
                </div>
                <div>
                    <input type="number" min="0" max={self.treshold_high.to_string()} step="1"
                        value={self.treshold_low.to_string()}
                        onchange={link.callback(|event: Event| Msg::TresholdLowChanged(event))} />
                    <input type="number" min={self.treshold_low.to_string()} max="255" step="1"
                        value={self.treshold_high.to_string()}
                        onchange={link.callback(|event: Event| Msg::TresholdHighChanged(event))} />
                    <button onclick={link.callback(|_| Msg::ApplyThreshold )}>{"Apply treshold"}</button>
                </div>
                <div>
                    <input type="range" min="0" max="1" step="0.01"
                        value={self.black_percent.to_string()}
                        onchange={link.callback(|event: Event| Msg::PercentBlackChanged(event))} />
                    <span>{format!("{:.2}%", self.black_percent * 100.0)}</span>
                    <button onclick={link.callback(|_| Msg::ApplyPercentBlackSelectionThreshold )}>{"Apply treshold (Percent Black Selection)"}</button>
                </div>
                <div>
                    <button onclick={link.callback(|_| Msg::ApplyMeanIterativeSelectionThreshold )}>{"Apply treshold (Mean Iterative Selection)"}</button>
                    <button onclick={link.callback(|_| Msg::ApplyEntropySelectionThreshold )}>{"Apply treshold (Entropy Selection)"}</button>
                    <button onclick={link.callback(|_| Msg::ApplyMinimumErrorThreshold )}>{"Apply treshold (Minimum Error)"}</button>
                    <button onclick={link.callback(|_| Msg::ApplyFuzzyMinimumErrorThreshold )}>{"Apply treshold (Fuzzy Minimum Error)"}</button>
                </div>
                <div>
                    <canvas ref={self.canvas_ref.clone()}
                        width={self.image.get_width().to_string()}
                        height={self.image.get_height().to_string()}
                    />
                </div>
            </>
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StretchHistogram => {
                self.image_to_display = self.image.get_stretched_image();

                true
            }
            Msg::EqualizeHistogram => {
                self.image_to_display = self.image.get_equalized_image();

                true
            }
            Msg::ApplyThreshold => {
                self.image_to_display = self
                    .image
                    .threshold((self.treshold_low, self.treshold_high));

                true
            }
            Msg::TresholdLowChanged(event) => {
                let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                self.treshold_low = input.value_as_number() as u8;
                if self.treshold_low > self.treshold_high {
                    self.treshold_high = self.treshold_low;
                }

                true
            }
            Msg::TresholdHighChanged(event) => {
                let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                self.treshold_high = input.value_as_number() as u8;
                if self.treshold_high < self.treshold_low {
                    self.treshold_high = self.treshold_low;
                }

                true
            }
            Msg::ApplyMeanIterativeSelectionThreshold => {
                self.image_to_display = self.image.mean_iterative_selection();

                true
            }
            Msg::ApplyPercentBlackSelectionThreshold => {
                self.image_to_display = self.image.percent_black_selection(self.black_percent);

                true
            },
            Msg::PercentBlackChanged(event) => {
                let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                self.black_percent = input.value_as_number() as f32;

                true
            },
            Msg::ApplyEntropySelectionThreshold => {
                self.image_to_display = self.image.entropy_selection();

                true
            },
            Msg::ApplyMinimumErrorThreshold => {
                self.image_to_display = self.image.minimum_error_selection();

                true
            },
            Msg::ApplyFuzzyMinimumErrorThreshold => {
                self.image_to_display = self.image.fuzzy_minimum_error_selection();

                true
            },
        }
    }

    fn changed(&mut self, ctx: &yew::Context<Self>) -> bool {
        self.image = Image::new_with_data(ctx.props().image_data.clone());
        self.image_to_display = self.image.clone();

        true
    }

    fn rendered(&mut self, _ctx: &yew::Context<Self>, first_render: bool) {
        if first_render {
            self.canvas_ctx = Some(
                self.canvas_ref
                    .cast::<HtmlCanvasElement>()
                    .unwrap()
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap(),
            );
        }

        let width = self.image_to_display.get_width();
        let height = self.image_to_display.get_height();
        let canvas_ctx = self.canvas_ctx.as_ref().unwrap();
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(self.image_to_display.get_data_ref()),
            width,
            height,
        )
        .unwrap();

        canvas_ctx.clear_rect(0.0, 0.0, width.into(), height.into());
        canvas_ctx.set_image_smoothing_enabled(false);
        canvas_ctx
            .put_image_data(&image_data, 0.0, 0.0)
            .expect("Couldn't draw image");
    }
}
