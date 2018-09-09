extern crate actix;
extern crate actix_web;
use actix::prelude::*;
use actix_web::{
    AsyncResponder, FutureResponse, HttpResponse, HttpRequest, Path
};

extern crate lifx_ctl;
use lifx_ctl::*;


// web services act a bit differently from the actor parts above.

struct AppState {
    log_addr: actix::Addr<LogActor>,
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

    let _int_addr = IntervalActor::new(logactor_addr.clone())
        .start();

    actix_web::server::new(move || {
        actix_web::App::with_state(AppState{
            log_addr: logactor_addr.clone(),
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
