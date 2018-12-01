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

extern crate rand;
use rand::{thread_rng, Rng};

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

macro_rules! send_bytes {
    ($log_addr:expr, $sock:expr, $bytes:expr, $addr:expr) => ({
        let res1 = $sock.send_to($bytes, $addr);
        match res1 {
            Ok(_) => {
                log_event!($log_addr, "event 1 success");
            }
            Err(e) => {
                log_event!($log_addr, "Failed to send {}", e);
            }
        };
        // Send twice to be sure ...
        let res2 = $sock.send_to($bytes, $addr);
        match res2 {
            Ok(_) => {
                log_event!($log_addr, "event 2 success");
            }
            Err(e) => {
                log_event!($log_addr, "Failed to send {}", e);
            }
        };
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
struct LifxControllerSetColour {
    pub addr: SocketAddr,
    pub duration: u32,
    pub flicker: bool,
    pub colour: HSBK,
}

impl Message for LifxControllerSetColour {
    type Result = ();
}

impl Handler<LifxControllerSetColour> for LifxController {
    type Result = ();

    fn handle(&mut self, event: LifxControllerSetColour, _: &mut Context<Self>) -> Self::Result {

        // Set the default lifx options. This could be good to cache in the struct?
        log_event!(self.log_addr, "Change colour to: {:?}", event);

        let opts = lifx_core::BuildOptions {
            // target: Some(event.addr),
            // source: 12345678,
            ..Default::default()
        };

        let rawmsg = lifx_core::RawMessage::build(&opts,
            lifx_core::Message::LightSetColor {
                reserved: 0,
                color: event.colour,
                duration: event.duration,
            }
        ).unwrap();
        let raw_bytes = rawmsg.pack().unwrap();

        // Always set once, even on flicker, to make sure it's the colour
        send_bytes!(self.log_addr, self.sock, &raw_bytes, &(event.addr));

        let mut rng = thread_rng();
        let r = rng.gen_range(0, 6);

        if event.flicker && r == 0 {

            let flick_rawmsg = lifx_core::RawMessage::build(&opts,
                lifx_core::Message::LightSetColor {
                    reserved: 0,
                    color: HSBK {
                        hue: 0,
                        saturation: 0,
                        brightness: 0,
                        kelvin: 0,
                    },
                    duration: event.duration,
                }
            ).unwrap();
            let flick_bytes = flick_rawmsg.pack().unwrap();

            for i in 0..rng.gen_range(1, 4) {
                use std::time::Duration;
                use std::thread;
                {
                    let d = rng.gen_range(0, 6);
                    thread::sleep(Duration::from_millis(d * 25));
                    send_bytes!(self.log_addr, self.sock, &flick_bytes, &(event.addr));
                }
                {
                    let d = rng.gen_range(0, 6);
                    thread::sleep(Duration::from_millis(d * 25));
                    send_bytes!(self.log_addr, self.sock, &raw_bytes, &(event.addr));
                }
            } // end for
        } // end flicker
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

struct LightBulbState {
    bulb: LightBulb,
    plan: plans::LightPlan,
    last_event: time::Tm,
}

pub struct LightManager {
    bulbs: Vec<LightBulbState>,
    log_addr: actix::Addr<LogActor>,
    lifx: actix::Addr<LifxController>,
}

impl LightManager {
    pub fn new(log_addr: actix::Addr<LogActor>,
        lifx: actix::Addr<LifxController>) -> Self {
        // Init all the light plans and attach them here?
        LightManager {
            log_addr: log_addr,
            bulbs: Vec::new(),
            lifx: lifx,
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

        // TODO: This should make a registration object that is mutable.
        let plan = if reg.0.name == "toilet" {
            plans::LightPlan::RedshiftToilet
        } else if reg.0.name == "kitchen" {
            plans::LightPlan::RedshiftKitchen
        } else if reg.0.name == "lamp" {
            plans::LightPlan::Pause
        } else {
            plans::LightPlan::RedshiftMain
        };

        self.bulbs.push(LightBulbState {
            bulb: reg.0,
            plan: plan,
            last_event: time::empty_tm(),
        });

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
            let s = b.bulb.status();
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
        for b in self.bulbs.iter_mut() {
            let t_now = time::now();

            let shift = if t_now > b.last_event {
                b.plan.shift(t_now)
            } else {
                None
            };

            match shift {
                Some(lshift) => {
                    log_event!(self.log_addr, "Shift requested to {:?}", lshift);
                    self.lifx.do_send(
                        LifxControllerSetColour {
                            addr: b.bulb.addr.clone(),
                            duration: lshift.duration,
                            flicker: lshift.flicker,
                            colour: lshift.colour.clone(),
                        }
                    );
                    // Update the shift event
                    b.last_event = t_now + time::Duration::milliseconds(lshift.duration as i64);
                }
                _ => {
                    log_event!(self.log_addr, "No shift for {}", b.bulb.name.as_str());
                }
            }
        }; // end for
    }
}

pub struct LightManagerPlanChange {
    plan: plans::LightPlan
}

impl Message for LightManagerPlanChange {
    type Result = ();
}

impl Handler<LightManagerPlanChange> for LightManager {
    type Result = ();

    fn handle(&mut self, req: LightManagerPlanChange, ctx: &mut Context<Self>) -> Self::Result {

        for b in self.bulbs.iter_mut() {
            let prop_plan = if b.bulb.name == "toilet" {
                if req.plan == plans::LightPlan::RedshiftMain {
                    plans::LightPlan::RedshiftToilet
                } else {
                    plans::LightPlan::PartyHardToilet
                }
            } else if b.bulb.name == "kitchen" {
                plans::LightPlan::RedshiftKitchen
            } else if b.bulb.name == "lamp" {
                plans::LightPlan::Pause
            } else {
                req.plan
            };
            if prop_plan != b.plan {
                log_event!(self.log_addr, "Changing to proposed plan {} -> {:?}", b.bulb.name.as_str(), prop_plan);
                // Set the time to 0 to cause the change to be asap
                b.last_event = time::empty_tm();
                b.plan = prop_plan;
            };
        }
    }
}

// Need a way to register bulbs
// Need to query all

// Every X seconds we wake and trigger the bulbs?

// Every Y seconds we wake and trigger a discovery?

// Every Z seconds we wake and check the lightplan expiry?

// Do we make multiple timers? Or one and modulo?

pub struct IntervalActor {
    log_addr: actix::Addr<LogActor>,
    lm: actix::Addr<LightManager>,
}

impl IntervalActor {
    pub fn new(log_addr: actix::Addr<LogActor>,
        lm: actix::Addr<LightManager>
        ) -> Self {
        IntervalActor {
            log_addr: log_addr,
            lm: lm,
        }
    }

    fn bulb_shift(&mut self) {
        log_event!(self.log_addr, "shift ...");
        self.lm.do_send(LightManagerShift);
    }

    fn check_party(&mut self) {
        let plan = if std::path::Path::new("/tmp/partyhard").exists() {
            plans::LightPlan::PartyHardMain
        } else {
            plans::LightPlan::RedshiftMain
        };
        log_event!(self.log_addr, "Checking for party hard ... {:?}", plan);

        self.lm.do_send(LightManagerPlanChange{
            plan: plan
        });
    }
}

impl Actor for IntervalActor {
    type Context = actix::Context<Self>;

    // Called after the actor has started.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(1000), move |act, _ctx| {
            act.bulb_shift();
        });
        ctx.run_interval(Duration::from_millis(10000), move |act, _ctx| {
            act.check_party();
        });
    }
}

