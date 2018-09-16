extern crate actix;
use actix::prelude::*;
extern crate futures;
use futures::future::Future;
use futures::future::lazy;
use futures::future;

extern crate tokio;
use tokio::executor::current_thread::CurrentThread;

extern crate lifx_ctl;
use lifx_ctl::*;

extern crate time;

extern crate lifx_core;
use lifx_core::HSBK;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use lifx_ctl::plans::{LightPlan, LightShift};

fn assert_shift(plan: &LightPlan, time_str: &str, expect: Option<LightShift>)
{
    let t = time::strptime(time_str, "%T").unwrap();
    let shift = plan.shift(t);
    println!("T: {} -> {:?}", time_str, shift);
    assert_eq!(shift, expect);
}


#[test]
fn plan_redshift_main() {
    let redshift_main = LightPlan::RedshiftMain;

    // At midday it should be max white
    assert_shift(&redshift_main, "12:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 65535,
            kelvin: 4000,
        }})
    );
    // at 16:30 mid afternoon we start evening.
    assert_shift(&redshift_main, "16:30:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 55267,
            kelvin: 4000,
        }})
    );
    // at 17:00 we start evening.
    assert_shift(&redshift_main, "17:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 45000,
            kelvin: 4000,
        }})
    );
    assert_shift(&redshift_main, "18:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 39000,
            kelvin: 3375,
        }})
    );
    // at 19:00 we are night
    assert_shift(&redshift_main, "19:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 33000,
            kelvin: 2750,
        }})
    );
    // at midnight
    assert_shift(&redshift_main, "00:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 33000,
            kelvin: 2750,
        }})
    );
    // at 7 am
    assert_shift(&redshift_main, "07:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 33000,
            kelvin: 2750,
        }})
    );

}

#[test]
fn plan_redshift_toilet() {
    let redshift_toilet = LightPlan::RedshiftToilet;

    // At midday it should be max white
    assert_shift(&redshift_toilet, "12:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 65535,
            kelvin: 3000,
        }})
    );
    // mid evening
    assert_shift(&redshift_toilet, "20:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 42321,
            kelvin: 1860,
        }})
    );
    // night
    assert_shift(&redshift_toilet, "00:00:00", Some(LightShift {
        duration: 4000,
        colour: HSBK {
            hue: 0,
            saturation: 0,
            brightness: 7500,
            kelvin: 150,
        }})
    );
}

#[test]
fn simple_setup() {
    // Build a test light bulb
    // Attach it to the controller
    System::run(|| {

        let logactor_addr = LogActor{ }.start();

        let lm = LightManager::new(logactor_addr.clone());
        let lmaddr = lm.start();

        // add some bulbs

        let tbulb_1 = LightBulb::new(
            "tbulb1".to_string(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(172, 24, 18, 13)), 56700)
        );

        println!("b1: {:?}", tbulb_1);

        lmaddr.try_send(LightManagerRegister(tbulb_1)).unwrap();

        let fut = lmaddr.send(LightManagerStatus)
                    .map_err(|_| ())
                    .and_then(|r| {
                        println!("status outer: {:?}", r);
                        // Okay now stop-pu!
                        actix::System::current().stop();
                        future::result(Ok(()))
                    });

        tokio::spawn(fut);


    });
}

