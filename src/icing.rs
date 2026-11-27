//! ice: on the leading edges and the windshield in cloud or rain below
//! freezing, over the pitot tube without its heat, and in a piston engine's
//! carburettor in damp air at part throttle. the rates are the textbook's
//! rough ones (light about 0.5 mm a minute, severe over 2), the effects the
//! usual rules of thumb: a thin rough layer costs a quarter of the wing's
//! best lift and doubles its drag. estimates for a game, not icing data.

/// the air around the aircraft, as icing goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Env {
    /// outside air temperature (deg c).
    pub oat_c: f64,
    /// visible moisture: cloud or rain, 0..1.
    pub wet: f64,
    /// relative humidity, 0..1.
    pub humidity: f64,
}

/// what has built up.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ice {
    /// on the leading edges (mm).
    pub wing_mm: f64,
    /// the pitot tube iced over, 0..1.
    pub pitot: f64,
    /// the carburettor iced up, 0..1.
    pub carb: f64,
    /// the windshield frosted, 0..1.
    pub screen: f64,
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// the switches against it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Heat {
    pub pitot: bool,
    /// wing, engine and windshield anti ice (a jet's bleed air).
    pub anti_ice: bool,
    pub carb: bool,
}

impl Ice {
    /// how fast ice builds on a leading edge (mm a minute): most from -15 to 0 c, faster with speed.
    pub fn rate(env: &Env, v: f64) -> f64 {
        let cold = smooth(1.0, -2.0, env.oat_c) * smooth(-25.0, -12.0, env.oat_c);
        2.0 * env.wet * cold * (v / 60.0).clamp(0.25, 1.6)
    }

    /// one step. `v`: true airspeed (m/s); `piston`: it has a carburettor;
    /// `throttle`: 0..1; `airborne`.
    pub fn step(&mut self, dt: f64, env: &Env, heat: Heat, v: f64, piston: bool, throttle: f64, airborne: bool) {
        let min = dt / 60.0;
        let rate = if airborne { Self::rate(env, v) } else { 0.0 };
        if heat.anti_ice {
            self.wing_mm = (self.wing_mm - 2.0 * min).max(0.0);
            self.screen = (self.screen - dt / 15.0).max(0.0);
        } else {
            self.wing_mm += rate * min;
            self.screen = (self.screen + rate * min * 0.25).min(1.0);
        }
        // above freezing it melts; in clear dry air below it slowly goes.
        if env.oat_c > 0.0 {
            let melt = (0.4 + 0.5 * env.oat_c) * min;
            self.wing_mm = (self.wing_mm - melt).max(0.0);
            self.screen = (self.screen - melt * 0.3).max(0.0);
        } else if env.wet < 0.05 {
            self.wing_mm = (self.wing_mm - 0.04 * min).max(0.0);
            self.screen = (self.screen - 0.01 * min).max(0.0);
        }
        self.wing_mm = self.wing_mm.min(40.0);
        // the pitot ices over in a minute or so without its heat.
        if heat.pitot || env.oat_c > 2.0 {
            self.pitot = (self.pitot - dt / 12.0).max(0.0);
        } else if airborne && env.wet > 0.2 && env.oat_c < 0.5 {
            self.pitot = (self.pitot + dt / 50.0 * env.wet).min(1.0);
        }
        // carburettor ice: the venturi cools damp air by up to 30 c, worst at part throttle.
        if piston {
            if heat.carb {
                self.carb = (self.carb - dt / 25.0).max(0.0);
            } else {
                let band = smooth(-12.0, -5.0, env.oat_c) * smooth(30.0, 18.0, env.oat_c);
                let damp = smooth(0.5, 0.95, env.humidity.max(env.wet));
                let part = (1.15 - throttle).clamp(0.15, 1.0);
                self.carb = (self.carb + dt / 240.0 * band * damp * part).min(1.0);
            }
        } else {
            self.carb = 0.0;
        }
    }

    /// share of the leading edges' worst ice, 0..1.
    pub fn frac(&self) -> f64 {
        (self.wing_mm / 20.0).clamp(0.0, 1.0)
    }

    /// what the ice takes off the stall angle of attack (rad).
    pub fn stall_loss(&self) -> f64 {
        0.09 * self.frac().sqrt()
    }

    /// what it multiplies the lift by, and the drag it adds (coefficient).
    pub fn lift_k(&self) -> f64 {
        1.0 - 0.12 * self.frac()
    }

    pub fn drag_add(&self) -> f64 {
        0.035 * self.frac().sqrt()
    }

    /// its weight (kg) on a wing of this area (m²): a band along the leading edge.
    pub fn mass(&self, wing_area: f64) -> f64 {
        wing_area * 0.2 * self.wing_mm / 1000.0 * 900.0
    }

    /// what a piston engine keeps of its power, and with the carb heat pulled.
    pub fn engine_k(&self, carb_heat: bool) -> f64 {
        (1.0 - self.carb.powf(1.5)) * if carb_heat { 0.92 } else { 1.0 }
    }

    pub fn pitot_blocked(&self) -> bool {
        self.pitot > 0.8
    }

    /// worth a caution on the panel.
    pub fn any(&self) -> bool {
        self.wing_mm > 1.0 || self.pitot > 0.3 || self.carb > 0.25
    }
}

impl crate::Game {
    /// a word on the ice as it builds, now and then.
    pub(crate) fn ice_notes(&mut self) {
        let ice = self.plane.ice;
        if !ice.any() || self.real < self.ice_note_t {
            return;
        }
        self.ice_note_t = self.real + 20.0;
        let mut parts: Vec<String> = Vec::new();
        if ice.wing_mm > 1.0 {
            parts.push(format!("{:.0} mm {}", ice.wing_mm, crate::tr_en("on the wings")));
        }
        if ice.pitot > 0.3 {
            parts.push(crate::tr_en("pitot icing").into());
        }
        if ice.carb > 0.25 {
            parts.push(crate::tr_en("carburettor ice").into());
        }
        self.status = format!("{}: {}", crate::tr_en("ICE"), parts.join(", "));
        self.status_until = self.real + 6.0;
    }
}

/// the air's temperature at a height, from the freezing level (m) and the standard lapse.
pub fn oat_at(h: f64, freezing: f64) -> f64 {
    (freezing - h) * 0.0065
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ice_builds_in_cold_cloud_and_anti_ice_clears_it() {
        let env = Env { oat_c: -8.0, wet: 1.0, humidity: 1.0 };
        let mut ice = Ice::default();
        for _ in 0..600 {
            ice.step(0.1, &env, Heat::default(), 60.0, true, 0.6, true);
        }
        assert!(ice.wing_mm > 1.5 && ice.wing_mm < 3.0, "{}", ice.wing_mm);
        assert!(ice.pitot_blocked());
        let heat = Heat { pitot: true, anti_ice: true, carb: true };
        for _ in 0..1200 {
            ice.step(0.1, &env, heat, 60.0, true, 0.6, true);
        }
        assert_eq!((ice.wing_mm, ice.pitot), (0.0, 0.0));
        // nothing builds in clear air above freezing.
        let warm = Env { oat_c: 12.0, wet: 0.0, humidity: 0.3 };
        let mut ice = Ice::default();
        for _ in 0..600 {
            ice.step(0.1, &warm, Heat::default(), 60.0, false, 0.6, true);
        }
        assert_eq!(ice.wing_mm, 0.0);
    }
}
