extern crate actix;
extern crate actix_web;
use actix::prelude::*;
use actix_web::web::{self, Data, HttpResponse, Json, Path};
use actix_web::{middleware, App, HttpServer};

extern crate lifx_ctl;
use lifx_ctl::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{LightManagerPlanStartParty, LightManagerPlanEndParty};
extern crate lifx_core;
use lifx_core::HSBK;

#[macro_use]
extern crate serde_derive;

// web services act a bit differently from the actor parts above.

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManualReq {
    hue: u16,
    sat: u16,
    bri: u16,
    k: u16
}

impl ManualReq {
    fn into_hsbk(self) -> HSBK {
        HSBK {
            hue: self.hue,
            saturation: self.sat,
            brightness: self.bri,
            kelvin: self.k
        }
    }
}

struct AppState {
    _log_addr: actix::Addr<LogActor>,
    lightmanager: actix::Addr<LightManager>,
}

async fn index_view() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

async fn party_start_view(state: Data<AppState>) -> HttpResponse {
    let _ = state.lightmanager.send(LightManagerPlanStartParty).await;
    HttpResponse::Ok().body("Party Started!!")
}

async fn party_end_view(state: Data<AppState>) -> HttpResponse {
    let _ = state.lightmanager.send(LightManagerPlanEndParty).await;
    HttpResponse::Ok().body("Party Over :(")
}

async fn manual_view((state, name, req): (Data<AppState>, Path<String>, Json<ManualReq>)) -> HttpResponse {
    let msg = LightManagerBulbManual {
        name: name.into_inner(),
        hsbk: req.into_inner().into_hsbk()
    };
    println!("SENDING -> {:?}", msg);
    let r = state.lightmanager.send(msg).await;
    HttpResponse::Ok().body(format!("Status -> {:?}", r))
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
        plans::LightPlan::PartyHardMain
    );
    let bulb_pole = LightBulb::new(
        "pole".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 12)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain
    );
    let bulb_toilet = LightBulb::new(
        "toilet".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 13)), 56700),
        plans::LightPlan::RedshiftToilet,
        plans::LightPlan::PartyHardToilet
    );
    let bulb_office = LightBulb::new(
        "office".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 21)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain
    );
    let bulb_kitchen = LightBulb::new(
        "kitchen".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 23)), 56700),
        plans::LightPlan::RedshiftKitchen,
        plans::LightPlan::RedshiftKitchen
    );
    let bulb_lamp = LightBulb::new(
        "lamp".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 22)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::PartyHardMain
    );
    let bulb_deck = LightBulb::new(
        "deck".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 24)), 56700),
        plans::LightPlan::RedshiftMain,
        plans::LightPlan::RedshiftMain
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
            .route("", web::get().to(index_view))
            .route("/", web::get().to(index_view))
            .route("/party/start", web::get().to(party_start_view))
            .route("/party/end", web::get().to(party_end_view))
            .route("/manual/{name}", web::post().to(manual_view))
    });
    server.bind("0.0.0.0:8081").unwrap().run();

    println!("Starting event server ...");

    let _ = sys.run();
}
