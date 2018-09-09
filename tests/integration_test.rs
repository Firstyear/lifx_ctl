extern crate actix;
use actix::prelude::*;

extern crate lifx_ctl;
use lifx_ctl::LightController;

struct VirtualController {
}

impl LightController for VirtualController {
}

impl Actor for VirtualController {
    type Context = Context<Self>;
}

#[test]
fn it_works() {
    println!("woo");
}

#[test]
fn plan_redshift_main() {
}

#[test]
fn simple_setup() {

}

