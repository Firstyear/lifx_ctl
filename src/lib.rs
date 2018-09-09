extern crate actix;
use actix::prelude::*;

use std::time::Duration;

extern crate time;


extern crate lifx_core;
use lifx_core::HSBK;

// The actual lifx control bits

pub trait LightController {
}

struct LifxController {
}

impl LightController for LifxController {
}

impl Actor for LifxController {
    type Context = Context<Self>;
}

// LightPlans?

// trait LightPlan

// struct redshift_main
fn redshift_main (ts: time::Tm) -> HSBK {
    HSBK {
        hue: 0,
        saturation: 0,
        brightness: 0,
        kelvin: 0,
    }
}

// struct redshift_toilet
// struct party_main + expire time?
// struct party_toilet + expire time?
// struct manual (just store and return a value) + expire time?


// LightBulbs
//    A bulb has a default light plan
//    A bulb should have an active light plan, which optional expires
//    A plan returns a transition which defines the packets and a delay after
//      the packets (IE for flashing etc).
//    They should store the current bulb state from the plan
//    Send changes to the control

struct LightBulbActor {
    bright: usize,
    // active lightplan
    // default lightplan
    // controller <trait based>
}

impl Actor for LightBulbActor {
    type Context = Context<Self>;
}

// Logs

pub struct LogEvent {
    pub msg: String,
}

impl Message for LogEvent {
    type Result = Result<(), ()>;
}

pub struct LogActor {
}

impl Actor for LogActor {
    type Context = Context<Self>;
}

impl Handler<LogEvent> for LogActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, event: LogEvent, _: &mut Context<Self>) -> Self::Result {
        println!("EVENT: {}", event.msg );
        Ok(())
    }
}

// Every X seconds we wake and trigger the bulbs?

// Every Y seconds we wake and trigger a discovery?

// Every Z seconds we wake and check the lightplan expiry?

// Do we make multiple timers? Or one and modulo?

pub struct IntervalActor {
    count: usize,
    log_addr: actix::Addr<LogActor>,
}

impl IntervalActor {
    pub fn new(log_addr: actix::Addr<LogActor>) -> Self {
        IntervalActor {
            count: 0,
            log_addr: log_addr,
        }
    }

    fn log_event(&self, s: String) {
        self.log_addr.do_send(
            LogEvent{ msg: s }
        );
    }

    fn sched_event(&mut self) {
        self.log_event(String::from(format!("Sched {}", self.count)) );
        self.count += 1;
    }

    fn sched_different_event(&mut self) {
        self.log_event(String::from("Sched diff 5") );
    }
}

impl Actor for IntervalActor {
    type Context = actix::Context<Self>;

    // Called after the actor has started.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(1000), move |act, ctx| {
            act.sched_event();
        });
        ctx.run_interval(Duration::from_millis(5000), move |act, ctx| {
            act.sched_different_event();
        });
    }
}

