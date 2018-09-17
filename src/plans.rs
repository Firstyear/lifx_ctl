extern crate lifx_core;
use lifx_core::HSBK;

extern crate time;

extern crate rand;
use rand::{thread_rng, Rng};

// LightPlans?

static party_colours: [HSBK; 5] = [
    // Blue
    HSBK {
        hue: 43634,
        saturation: 65535,
        brightness: 47142,
        kelvin: 3500,
    },
    // RED
    HSBK {
        hue: 65535,
        saturation: 65535,
        brightness: 65535,
        kelvin: 3500,
    },
    // GREEN
    HSBK {
        hue: 16173,
        saturation: 65535,
        brightness: 65535,
        kelvin: 3500,
    },
    // ORANGE
    HSBK {
        hue: 6500,
        saturation: 65535,
        brightness: 65535,
        kelvin: 3500,
    },
    // PINK
    HSBK {
        hue: 58275,
        saturation: 65535,
        brightness: 47142,
        kelvin: 3500,
    },
];


// struct redshift_main
fn rshift_calc(vmax: u16, vmin: u16, hour: i32, hlow: i32, hhigh: i32, min: i32) -> u16 {
    let h_rem = hhigh - hour;
    // Number of minutes in the window
    let min_tot = ((hhigh - hlow) * 60) as f32;
    // Number of minutes left in the window
    let min_rem = ((h_rem * 60) - min) as f32;

    let vdiff = (vmax - vmin) as f32;

    let v = (vmin as f32) + (vdiff * (min_rem / min_tot));

    v as u16
}

#[derive(Debug, PartialEq)]
pub struct LightShift {
    pub colour: HSBK,
    pub duration: u32,
    pub flicker: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightPlan {
    RedshiftMain,
    RedshiftToilet,
    PartyHardMain,
    PartyHardToilet,
    Pause,
}

impl LightPlan {
    pub fn shift(&self, ts: time::Tm) -> Option<LightShift> {
        let hour = ts.tm_hour;
        let minute = ts.tm_min;

        match self {
            LightPlan::RedshiftMain => {
                // Helper values
                let DAY_START = 8;
                let DAY_END = 16;
                let EVENING_END = 17;
                let NIGHT_END = 19;

                Some(LightShift {
                    duration: 4000,
                    flicker: false,
                    colour: if hour >= DAY_START && hour < DAY_END {
                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: 65535,
                            kelvin: 4000,
                        }
                    } else if hour >= DAY_END && hour < EVENING_END {
                        let bright = rshift_calc(65535, 45000, hour, DAY_END, EVENING_END, minute);

                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: bright as u16,
                            kelvin: 4000,
                        }
                    } else if hour >= EVENING_END && hour < NIGHT_END {
                        let bright = rshift_calc(45000, 33000, hour, EVENING_END, NIGHT_END, minute);
                        let k = rshift_calc(4000, 2750, hour, EVENING_END, NIGHT_END, minute);

                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: bright as u16,
                            kelvin: k as u16,
                        }
                    } else {
                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: 33000,
                            // brightness: 65535,
                            kelvin: 2750,
                         }
                    }
                }) // End some
            }
            LightPlan::RedshiftToilet => {
                let DAY_START = 8;
                let DAY_END = 18;
                let NIGHT_END = 23;

                // This may need an extra stepping perhaps

                Some(LightShift {
                    duration: 4000,
                    flicker: false,
                    colour: if hour >= DAY_START && hour < DAY_END {
                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: 65535,
                            kelvin: 3000,
                        }
                    } else if hour >= DAY_END && hour < NIGHT_END {
                        let bright = rshift_calc(65535, 7500, hour, DAY_END, NIGHT_END, minute);
                        let k = rshift_calc(3000, 150, hour, DAY_END, NIGHT_END, minute);

                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: bright as u16,
                            kelvin: k as u16,
                        }
                    } else {
                        HSBK {
                            hue: 0,
                            saturation: 0,
                            brightness: 7500,
                            kelvin: 150,
                         }
                    }
                }) // End some
            }
            LightPlan::PartyHardMain => {
                let mut rng = thread_rng();

                Some(LightShift {
                    duration: 2000,
                    flicker: false,
                    colour: rng.choose(&party_colours)
                        //std::option::Option<&lifx_core::HSBK>
                        .unwrap()
                        .clone()
                })
            }
            LightPlan::PartyHardToilet => {
                Some(LightShift {
                    duration: 65,
                    flicker: true,
                    colour: HSBK {
                        hue: 45074,
                        saturation: 65535,
                        brightness: 39799,
                        kelvin: 3500
                    }
                })
            }
            _ => {
                println!("Do nothing");
                None
            }
        }
    }
}
// struct party_main + expire time?

// struct party_toilet + expire time?

// struct manual (just store and return a value) + expire time?

