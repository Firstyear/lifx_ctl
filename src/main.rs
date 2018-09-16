extern crate actix;
extern crate actix_web;
use actix::prelude::*;
use actix_web::{
    AsyncResponder, FutureResponse, HttpResponse, HttpRequest, Path
};

extern crate lifx_ctl;
use lifx_ctl::*;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// web services act a bit differently from the actor parts above.

struct AppState {
    log_addr: actix::Addr<LogActor>,
    lightmanager: actix::Addr<LightManager>,
}

impl AppState {
    fn log_event(&self, s: String) {
        self.log_addr.do_send(
            LogEvent{ msg: s }
        );
    }
}

//fn index((name, state): (Path<String>, State<AppState>)) -> FutureResponse<HttpResponse> {
fn index(req: &HttpRequest<AppState>) -> &'static str {
    req.state().log_event(String::from("index request"));
    "Hello World!\n"
}

fn main() {
    let sys = actix::System::new("lifx_ctl");

    let logactor_addr = LogActor{ }.start();

    let lifx_addr = LifxController::new(logactor_addr.clone()).start();

    let lm = LightManager::new(logactor_addr.clone()).start();

    // For now, we cheat and pre-reg the bulbs.

    let bulb_lounge = LightBulb::new(
        "lounge".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 10)), 56700)
    );
    let bulb_pole = LightBulb::new(
        "pole".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 12)), 56700)
    );
    let bulb_toilet = LightBulb::new(
        "toilet".to_string(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 13)), 56700)
    );

    lm.try_send(LightManagerRegister(bulb_lounge)).unwrap();
    lm.try_send(LightManagerRegister(bulb_pole)).unwrap();
    lm.try_send(LightManagerRegister(bulb_toilet)).unwrap();

    let _int_addr = IntervalActor::new(logactor_addr.clone(), lm.clone())
        .start();

    actix_web::server::new(move || {
        actix_web::App::with_state(AppState{
            log_addr: logactor_addr.clone(),
            lightmanager: lm.clone(),
        })
        .middleware(actix_web::middleware::Logger::default())
        // .resource("/{name}", |r| r.method(actix_web::http::Method::GET).with(index))
        .resource("", |r| r.f(index))
        .resource("/", |r| r.f(index))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Starting event server ...");

    let _ = sys.run();
}
