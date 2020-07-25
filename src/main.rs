#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate lifx_core;
extern crate rand;
extern crate time;

use actix::prelude::*;
use actix_files as fs;
use actix_web::web::{self, Data, Form, HttpResponse, Json, Path};
use actix_web::{guard, middleware, App, HttpServer};
use askama::Template;
use lifx_core::HSBK;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod plans;
mod srv;
use srv::*;

/*
use sr::{
    LightBulbStatus, LightManagerBulbReset, LightManagerBulbStatus, LightManagerPlanEndParty,
    LightManagerPlanStartParty,
};
*/

pub static APPLICATION_JSON: &'static str = "application/json";
pub static APPLICATION_FORM: &'static str = "application/x-www-form-urlencoded";
pub static CONTENT_TYPE: &'static str = "content-type";

#[derive(Template)]
#[template(path = "wasm.html")]
struct WasmTemplate;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManualStatus {
    name: String,
    plan: String,
    hue: u16,
    sat: u16,
    bri: u16,
    k: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManualReq {
    hue: u16,
    sat: u16,
    bri: u16,
    k: u16,
}

impl ManualReq {
    fn into_hsbk(self) -> HSBK {
        HSBK {
            hue: self.hue,
            saturation: self.sat,
            brightness: self.bri,
            kelvin: self.k,
        }
    }
}

struct AppState {
    _log_addr: actix::Addr<LogActor>,
    lightmanager: actix::Addr<LightManager>,
}

async fn wasm_view() -> HttpResponse {
    let t = WasmTemplate;
    match t.render() {
        Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(format!("{:?}", e)),
    }
}

async fn status_view() -> HttpResponse {
    HttpResponse::Ok().body("Ok")
}

async fn party_start_view(state: Data<AppState>) -> HttpResponse {
    let _ = state.lightmanager.send(LightManagerPlanStartParty).await;
    HttpResponse::Ok().body("Party Started!!")
}

async fn party_end_view(state: Data<AppState>) -> HttpResponse {
    let _ = state.lightmanager.send(LightManagerPlanEndParty).await;
    HttpResponse::Ok().body("Party Over :(")
}

async fn manual_view((state, name): (Data<AppState>, Path<String>)) -> HttpResponse {
    let r = state
        .lightmanager
        .send(LightManagerBulbStatus {
            name: name.into_inner(),
        })
        .await;
    match r {
        Ok(Some(s)) => {
            let r = ManualStatus {
                name: s.name,
                plan: s.plan,
                hue: s.current.hue,
                sat: s.current.saturation,
                bri: s.current.brightness,
                k: s.current.kelvin,
            };
            HttpResponse::Ok().json(r)
        }
        _ => {
            HttpResponse::InternalServerError()
                .content_type("text/html")
                .body("manager status")
        }
    }
}

async fn manual_post_reset((state, name): (Data<AppState>, Path<String>)) -> HttpResponse {
    let _ = state
        .lightmanager
        .send(LightManagerBulbReset {
            name: name.into_inner(),
        })
        .await;
    HttpResponse::Ok().body("Bulb Reset")
}

async fn manual_post_generic(state: Data<AppState>, name: String, req: ManualReq) -> HttpResponse {
    let msg = LightManagerBulbManual {
        name,
        hsbk: req.into_hsbk(),
    };
    let r = state.lightmanager.send(msg).await;
    HttpResponse::Ok().body(format!("Status -> {:?}", r))
}

async fn manual_post_form(
    (state, name, req): (Data<AppState>, Path<String>, Form<ManualReq>),
) -> HttpResponse {
    manual_post_generic(state, name.into_inner(), req.into_inner()).await
}

async fn manual_post_json(
    (state, name, req): (Data<AppState>, Path<String>, Json<ManualReq>),
) -> HttpResponse {
    manual_post_generic(state, name.into_inner(), req.into_inner()).await
}

fn main() {
    let sys = actix::System::new("lifx_ctl");

    let logactor_addr = LogActor {}.start();

    let lifx_addr = LifxController::new(logactor_addr.clone()).start();

    let lm = LightManager::new(logactor_addr.clone(), lifx_addr.clone()).start();

    // For now, we cheat and pre-reg the bulbs.

    let bulb_lounge = LightBulb::new(
        "lounge".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 10)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain,
    );
    let bulb_pole = LightBulb::new(
        "pole".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 12)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain,
    );
    let bulb_toilet = LightBulb::new(
        "toilet".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 13)), 56700),
        plans::LightPlan::RedshiftToilet,
        plans::LightPlan::PartyHardToilet,
    );
    let bulb_office = LightBulb::new(
        "office".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 21)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain,
    );
    let bulb_kitchen = LightBulb::new(
        "kitchen".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 23)), 56700),
        plans::LightPlan::RedshiftKitchen,
        plans::LightPlan::RedshiftKitchen,
    );
    let bulb_lamp = LightBulb::new(
        "lamp".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 22)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain,
    );
    let bulb_deck = LightBulb::new(
        "deck".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 24)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::RedshiftMain,
    );

    lm.try_send(LightManagerRegister(bulb_lounge)).unwrap();
    lm.try_send(LightManagerRegister(bulb_pole)).unwrap();
    lm.try_send(LightManagerRegister(bulb_toilet)).unwrap();
    lm.try_send(LightManagerRegister(bulb_office)).unwrap();
    lm.try_send(LightManagerRegister(bulb_kitchen)).unwrap();
    lm.try_send(LightManagerRegister(bulb_lamp)).unwrap();
    lm.try_send(LightManagerRegister(bulb_deck)).unwrap();

    let _int_addr = IntervalActor::new(logactor_addr.clone(), lm.clone()).start();

    env_logger::init();

    let server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                _log_addr: logactor_addr.clone(),
                lightmanager: lm.clone(),
            })
            .wrap(middleware::Logger::default())
            .service(fs::Files::new("/static", "./static"))
            .service(fs::Files::new("/pkg", "./pkg"))
            .route("", web::get().to(wasm_view))
            .route("/", web::get().to(wasm_view))
            .route("/status", web::get().to(status_view))
            .route("/party/start", web::post().to(party_start_view))
            .route("/party/end", web::post().to(party_end_view))
            .route("/manual/{name}", web::get().to(manual_view))
            .route(
                "/manual/{name}",
                web::post()
                    .to(manual_post_json)
                    .guard(guard::Header(CONTENT_TYPE, APPLICATION_JSON)),
            )
            .route("/manual/{name}/reset", web::post().to(manual_post_reset))
    });
    server.bind("[::]:8081").unwrap().run();

    info!("Starting event server ...");

    let _ = sys.run();
}
