extern crate actix;
use actix::prelude::*;
extern crate futures;
use futures::future::Future;

use std::time::Duration;
use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

extern crate time;


extern crate lifx_core;
use lifx_core::HSBK;

pub mod plans;

// Helper for internal logging.
macro_rules! log_event {
    ($log_addr:expr, $($arg:tt)*) => ({
        $log_addr.do_send(
            LogEvent {
                msg: std::fmt::format(
                    format_args!($($arg)*)
                )
            }
        )
    })
}

// The actual lifx control bits

pub struct LifxController {
    sock: UdpSocket,
    log_addr: actix::Addr<LogActor>,
}

impl LifxController {
    pub fn new(log_addr: actix::Addr<LogActor>) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:56701").unwrap();

        LifxController {
            sock: sock,
            log_addr: log_addr,
        }
    }
}

impl Actor for LifxController {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub struct LifxControllerSetColour(pub HSBK);

impl Message for LifxControllerSetColour {
    type Result = ();
}

impl Handler<LifxControllerSetColour> for LifxController {
    type Result = ();

    fn handle(&mut self, event: LifxControllerSetColour, _: &mut Context<Self>) -> Self::Result {
        log_event!(self.log_addr, "Change colour to: {:?}", event);
    }
}

// LightBulbs
//    A bulb has a default light plan
//    A bulb should have an active light plan, which optional expires
//    A plan returns a transition which defines the packets and a delay after
//      the packets (IE for flashing etc).
//    They should store the current bulb state from the plan
//    Send changes to the control

#[derive(Debug)]
pub struct LightBulbStatus {
    name: String,
    current: HSBK,
}

#[derive(Debug)]
pub struct LightBulb {
    name: String,
    addr: SocketAddr,
    current: HSBK,
    // active_plan: P
    // plan: plans::LightPlan,
}

impl LightBulb {
    pub fn new(name: String, addr: SocketAddr) -> Self {
        LightBulb {
            name: name,
            current: HSBK {
                hue: 0,
                saturation: 0,
                brightness: 0,
                kelvin: 0,
            },
            addr: addr,
        }
    }

    pub fn status(&self) -> LightBulbStatus {
        LightBulbStatus {
            name: self.name.clone(),
            current: self.current.clone(),
        }
    }
}

// Logs

pub struct LogEvent {
    pub msg: String,
}

impl Message for LogEvent {
    type Result = ();
}

pub struct LogActor {}

impl Actor for LogActor {
    type Context = Context<Self>;
}

impl Handler<LogEvent> for LogActor {
    type Result = ();

    fn handle(&mut self, event: LogEvent, _: &mut Context<Self>) -> Self::Result {
        println!("EVENT: {}", event.msg );
    }
}


pub struct LightManager {
    bulbs: Vec<LightBulb>,
    log_addr: actix::Addr<LogActor>,
}

impl LightManager {
    pub fn new(log_addr: actix::Addr<LogActor>) -> Self {
        // Init all the light plans and attach them here?


        LightManager {
            log_addr: log_addr,
            bulbs: Vec::new(),
        }
    }

}

impl Actor for LightManager {
    type Context = Context<Self>;
}

pub struct LightManagerRegister(pub LightBulb);

impl Message for LightManagerRegister {
    type Result = Result<(), ()>;
}

impl Handler<LightManagerRegister> for LightManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, reg: LightManagerRegister, _: &mut Context<Self>) -> Self::Result {
        log_event!(self.log_addr, "Registered {}", reg.0.name);
        self.bulbs.push(reg.0);

        Ok(())
    }
}

pub struct LightManagerStatus;

impl Message for LightManagerStatus {
    type Result = Result<Vec<LightBulbStatus>, ()>;
}

impl Handler<LightManagerStatus> for LightManager {
    type Result = Result<Vec<LightBulbStatus>, ()>;

    fn handle(&mut self, _req: LightManagerStatus, ctx: &mut Context<Self>) -> Self::Result {
        log_event!(self.log_addr, "Status req");
        let status: Vec<LightBulbStatus> = self.bulbs.iter().map( |b| {
            let s = b.status();
            log_event!(self.log_addr, "status inner: {:?}", s);
            s
        }).collect();

        Ok(status)
    }
}

pub struct LightManagerShift;

impl Message for LightManagerShift {
    type Result = ();
}

impl Handler<LightManagerShift> for LightManager {
    type Result = ();

    fn handle(&mut self, _req: LightManagerShift, ctx: &mut Context<Self>) -> Self::Result {
        // Temporary
        let plan = plans::LightPlan::RedshiftToilet;
        let shift = plan.shift(time::now());
        log_event!(self.log_addr, "Shift requested to {:?}", shift);



    }
}

// Need a way to register bulbs
// Need to query all

// Every X seconds we wake and trigger the bulbs?

// Every Y seconds we wake and trigger a discovery?

// Every Z seconds we wake and check the lightplan expiry?

// Do we make multiple timers? Or one and modulo?

pub struct IntervalActor {
    count: usize,
    log_addr: actix::Addr<LogActor>,
    lm: actix::Addr<LightManager>,
}

impl IntervalActor {
    pub fn new(log_addr: actix::Addr<LogActor>,
        lm: actix::Addr<LightManager>
        ) -> Self {
        IntervalActor {
            count: 0,
            log_addr: log_addr,
            lm: lm,
        }
    }

    fn bulb_shift(&mut self) {
        log_event!(self.log_addr, "Sched {}", self.count);
        self.count += 1;
        self.lm.do_send(LightManagerShift);
    }
}

impl Actor for IntervalActor {
    type Context = actix::Context<Self>;

    // Called after the actor has started.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(2000), move |act, _ctx| {
            act.bulb_shift();
        });
    }
}

