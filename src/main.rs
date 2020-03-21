extern crate actix;
extern crate actix_web;
use actix::prelude::*;
use actix_web::web::{self, HttpResponse};
use actix_web::{middleware, App, HttpServer};

extern crate lifx_ctl;
use lifx_ctl::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// web services act a bit differently from the actor parts above.

struct AppState {
    _log_addr: actix::Addr<LogActor>,
    _lightmanager: actix::Addr<LightManager>,
}

async fn index_view() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
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
    );
    let bulb_pole = LightBulb::new(
        "pole".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 12)), 56700),
    );
    let bulb_toilet = LightBulb::new(
        "toilet".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 13)), 56700),
    );

    let bulb_office = LightBulb::new(
        "office".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 21)), 56700),
    );
    let bulb_kitchen = LightBulb::new(
        "kitchen".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 23)), 56700),
    );
    let bulb_lamp = LightBulb::new(
        "lamp".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 22)), 56700),
    );
    let bulb_deck = LightBulb::new(
        "deck".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 24)), 56700),
    );

    lm.try_send(LightManagerRegister(bulb_lounge)).unwrap();
    lm.try_send(LightManagerRegister(bulb_pole)).unwrap();
    lm.try_send(LightManagerRegister(bulb_toilet)).unwrap();
    lm.try_send(LightManagerRegister(bulb_office)).unwrap();
    lm.try_send(LightManagerRegister(bulb_kitchen)).unwrap();
    lm.try_send(LightManagerRegister(bulb_lamp)).unwrap();
    lm.try_send(LightManagerRegister(bulb_deck)).unwrap();

    let _int_addr = IntervalActor::new(logactor_addr.clone(), lm.clone()).start();

    let server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                _log_addr: logactor_addr.clone(),
                _lightmanager: lm.clone(),
            })
            .wrap(middleware::Logger::default())
            .route("", web::get().to(index_view))
            .route("/", web::get().to(index_view))
    });
    server.bind("127.0.0.1:8081").unwrap().run();

    println!("Starting event server ...");

    let _ = sys.run();
}
