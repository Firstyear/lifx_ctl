#![recursion_limit="512"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::format::{Json, Nothing};
use serde_json::json;
use serde_derive::Deserialize;
use anyhow::Error;


// === party buttons ===

pub struct Button {
    link: ComponentLink<Self>,
    title: &'static str,
    dest: &'static str,
    ft: Option<FetchTask>,
}

pub enum ButtonMsg {
    Clicked,
    Ignore
}

#[derive(Clone, PartialEq, Properties)]
pub struct ButtonProps {
    title: &'static str,
    dest: &'static str,
}

impl Button {
    fn call(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Nothing>| {
                let (meta, _) = response.into_parts();
                ConsoleService::log(format!("result -> {:?}", meta).as_str());
                ButtonMsg::Ignore
            }
        );
        let request = Request::post(self.dest).body(Nothing).unwrap();
        FetchService::fetch(request, callback).unwrap()
    }
}

impl Component for Button {
    type Message = ButtonMsg;
    type Properties = ButtonProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Button {
            link,
            title: props.title,
            dest: props.dest,
            ft: None,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ButtonMsg::Clicked => {
                ConsoleService::log("onlick");
                self.ft = Some(self.call());
            }
            ButtonMsg::Ignore => {}
        };
        true
    }

    fn view(&self) -> Html {
        html! {
            <button class="btn btn-success" onclick=self.link.callback(|_| ButtonMsg::Clicked)>{ format!("{}", self.title) }</button>
        }
    }
}

//=== bulbs ===
pub struct Bulb {
    link: ComponentLink<Self>,
    name: &'static str,
    ft: Option<FetchTask>,
    hue: u16,
    sat: u16,
    bri: u16,
    kel: u16,
}

pub enum BulbMsg {
    ResetPlan,
    HueInput(String),
    SatInput(String),
    BriInput(String),
    KelInput(String),
    Refresh,
    Status(u16, u16, u16, u16),
    Ignore
}

#[derive(Clone, PartialEq, Properties)]
pub struct BulbProps {
    name: &'static str,
    hue: u16,
    sat: u16,
    bri: u16,
    kel: u16,
}

#[derive(Debug, Deserialize)]
struct ManualStatus {
    name: String,
    plan: String,
    hue: u16,
    sat: u16,
    bri: u16,
    k: u16,
}

impl Bulb {
    fn call_reset_plan(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Nothing>| {
                let (meta, _) = response.into_parts();
                ConsoleService::log(format!("reset plan result -> {:?}", meta).as_str());
                BulbMsg::Refresh
            }
        );
        let request = Request::post(format!("/manual/{}/reset", self.name).as_str()).body(Nothing).unwrap();
        FetchService::fetch(request, callback).unwrap()
    }

    fn call_adjust(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Nothing>| {
                let (meta, _) = response.into_parts();
                ConsoleService::log(format!("bulb adjust result -> {:?}", meta).as_str());
                BulbMsg::Ignore
            }
        );
        let body = json!({
            "hue": self.hue,
            "sat": self.sat,
            "bri": self.bri,
            "k": self.kel,
        });
        let request = Request::post(format!("/manual/{}", self.name).as_str())
            .header("content-type", "application/json")
            .body(Json(&body))
            .unwrap();
        FetchService::fetch(request, callback).unwrap()
    }

    fn call_refresh(&mut self) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<ManualStatus, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                ConsoleService::log(format!("refresh status result -> {:?} {:?}", meta, data).as_str());
                match data {
                    Ok(x) => 
                        BulbMsg::Status(x.hue,x.sat,x.bri,x.k),
                    Err(e) => {
                        ConsoleService::log(format!("{:?}", e).as_str());
                        BulbMsg::Ignore
                    }
                }
            }
        );
        let request = Request::get(format!("/manual/{}", self.name).as_str()).body(Nothing).unwrap();
        FetchService::fetch(request, callback).unwrap()
    }
}

fn conv_u16(i: String) -> u16 {
    match u16::from_str_radix(i.as_str(), 10) {
        Ok(v) => v,
        Err(_) => 0,
    }
}

impl Component for Bulb {
    type Message = BulbMsg;
    type Properties = BulbProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Bulb {
            link,
            name: props.name,
            ft: None,
            hue: props.hue,
            sat: props.sat,
            bri: props.bri,
            kel: props.kel,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            BulbMsg::ResetPlan => {
                ConsoleService::log("onlick resetplan");
                self.ft = Some(self.call_reset_plan());
            }
            BulbMsg::HueInput(hval) => {
                let hval = conv_u16(hval);
                ConsoleService::log(format!("hue oninput {}", hval).as_str());
                self.hue = hval;
                self.ft = Some(self.call_adjust());
            }
            BulbMsg::SatInput(hval) => {
                let hval = conv_u16(hval);
                ConsoleService::log(format!("sat oninput {}", hval).as_str());
                self.sat = hval;
                self.ft = Some(self.call_adjust());
            }
            BulbMsg::BriInput(hval) => {
                let hval = conv_u16(hval);
                ConsoleService::log(format!("bri oninput {}", hval).as_str());
                self.bri = hval;
                self.ft = Some(self.call_adjust());
            }
            BulbMsg::KelInput(hval) => {
                let hval = conv_u16(hval);
                ConsoleService::log(format!("kel oninput {}", hval).as_str());
                self.kel = hval;
                self.ft = Some(self.call_adjust());
            }
            BulbMsg::Refresh => {
                self.ft = Some(self.call_refresh());
            }
            BulbMsg::Status(hue, sat, bri, kel) => {
                self.hue = hue;
                self.sat = sat;
                self.bri = bri;
                self.kel = kel;
            }
            BulbMsg::Ignore => {}
        };
        true
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.ft = Some(self.call_refresh());
        }
    }

    fn view(&self) -> Html {
        html! {
            <li>
                <p>{ format!("{}", self.name) }</p>
                  <ul>
                    <li>
                      <div class="form-group">
                          <label for="hue">{ "Hue" }</label>
                          <input type="range" class="custom-range" min="0" max="65535" id="hue" name="hue" step="1" value=self.hue
                                oninput=self.link.callback(|e: InputData| BulbMsg::HueInput(e.value)) />
                          <span id="hue-span" class="font-weight-bold text-primary ml-2 mt-1 valueSpan"/>
                      </div>
                    </li>
                    <li>
                      <div class="form-group">
                          <label for="sat">{ "Sat" }</label>
                          <input type="range" class="custom-range" min="0" max="65535" id="sat" name="sat" step="1" value=self.sat
                                oninput=self.link.callback(|e: InputData| BulbMsg::SatInput(e.value)) />
                          <span id="sat-span" class="font-weight-bold text-primary ml-2 mt-1 valueSpan"/>
                      </div>
                    </li>
                    <li>
                      <div class="form-group">
                          <label for="bri">{ "Bri" }</label>
                          <input type="range" class="custom-range" min="0" max="65535" id="bri" name="bri" step="1" value=self.bri
                                oninput=self.link.callback(|e: InputData| BulbMsg::BriInput(e.value)) />
                          <span id="bri-span" class="font-weight-bold text-primary ml-2 mt-1 valueSpan"/>
                      </div>
                    </li>
                    <li>
                      <div class="form-group">
                          <label for="k">{ "K" }</label>
                          <input type="range" class="custom-range" min="2000" max="9000" id="k" name="k" step="1" value=self.kel
                                oninput=self.link.callback(|e: InputData| BulbMsg::KelInput(e.value)) />
                          <span id="k-span" class="font-weight-bold text-primary ml-2 mt-1 valueSpan"/>
                      </div>
                    </li>
                  </ul>
                  <button class="btn btn-danger" onclick=self.link.callback(|_| BulbMsg::ResetPlan)>{ "Reset Bulb" }</button>
            </li>
        }
    }
}

//=== main app ===

pub struct App {
    link: ComponentLink<Self>,
}

pub enum AppMsg {}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div id="content" class="container">
                <ul>
                    <Bulb name="lounge"  hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="pole"    hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="toilet"  hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="office"  hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="kitchen" hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="lamp"    hue=0 sat=0 bri=0 kel=0 />
                    <Bulb name="deck"    hue=0 sat=0 bri=0 kel=0 />
                </ul>
                <Button title="Start Party! 🎉", dest="/party/start"/>
                <Button title="End Party ... 😔", dest="/party/end"/>
            </div>
        }
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<App>();

    Ok(())
}
