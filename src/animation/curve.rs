use std::ops::{Add, Div, Mul, Sub};

use bevy::math::{Vec2, Vec3, Vec4};

pub fn linear(t: f32) -> f32 {
    t
}

pub mod standard {
    pub fn clamp(t: f32) -> f32 {
        t.clamp(0.0, 1.0)
    }

    pub fn repeat(t: f32) -> f32 {
        (t - (0.5 + t).floor()) + 0.5
    }

    pub fn pingpong(t: f32) -> f32 {
        2.0 * (t - (t + 0.5).floor()).abs()
    }

    pub fn reverse(t: f32) -> f32 {
        1.0 - t
    }
}

/// Easing functions from https://easings.net/
pub mod ease {
    use super::Curve;
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;
    const C3: f32 = C1 + 1.0;

    pub fn in_sine(t: f32) -> f32 {
        1.0 - (t * std::f32::consts::FRAC_PI_2).cos()
    }
    pub fn out_sine(t: f32) -> f32 {
        (t * std::f32::consts::FRAC_PI_2).sin()
    }
    pub fn inout_sine(t: f32) -> f32 {
        -((t * std::f32::consts::PI) - 1.0) / 2.0
    }
    pub fn in_quad(t: f32) -> f32 {
        t * t
    }
    pub fn out_quad(t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }
    pub fn inout_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }
    pub fn in_cubic(t: f32) -> f32 {
        t.powi(3)
    }
    pub fn out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }
    pub fn inout_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t.powi(3)
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
    pub fn in_quart(t: f32) -> f32 {
        t.powi(4)
    }
    pub fn out_quart(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(4)
    }
    pub fn inout_quart(t: f32) -> f32 {
        if t < 0.5 {
            8.0 * t.powi(4)
        } else {
            1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
        }
    }
    pub fn in_expo(t: f32) -> f32 {
        if t < f32::MIN {
            0.0
        } else {
            2f32.powf(10.0 * t - 10.0)
        }
    }
    pub fn out_expo(t: f32) -> f32 {
        if 1.0 - t < f32::MIN {
            1.0
        } else {
            1.0 - 2f32.powf(-10.0 * t)
        }
    }
    pub fn inout_expo(t: f32) -> f32 {
        if t < f32::MIN {
            0.0
        } else if 1.0 - t < f32::MIN {
            1.0
        } else if t < 0.5 {
            2f32.powf(20.0 * t - 10.0) / 2.0
        } else {
            (2.0 - 2f32.powf(-20.0 * t + 10.0)) / 2.0
        }
    }
    pub fn in_circ(t: f32) -> f32 {
        1.0 - (1.0 - t * t).sqrt()
    }
    pub fn out_circ(t: f32) -> f32 {
        (1.0 - (t - 1.0).powi(2)).sqrt()
    }
    pub fn inout_circ(t: f32) -> f32 {
        if t < 0.5 {
            (1.0 - (2.0 * t).powi(2)) / 2.0
        } else {
            ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        }
    }

    pub fn in_back(t: f32) -> f32 {
        C3 * t.powi(3) + C1 * t.powi(2)
    }
    pub fn out_back(t: f32) -> f32 {
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }
    pub fn inout_back(t: f32) -> f32 {
        if t < 0.5 {
            (2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2) / 2.0
        } else {
            ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
        }
    }
    pub fn in_elastic(t: f32) -> f32 {
        if t < f32::MIN {
            0.0
        } else if (1.0 - t) < f32::MIN {
            1.0
        } else {
            2f32.powf(10.0 * t - 10.0)
                * ((t * 10.0 - 10.75) * std::f32::consts::FRAC_PI_3 * 2.0).sin()
        }
    }
    pub fn out_elastic(t: f32) -> f32 {
        if t < f32::MIN {
            0.0
        } else if (1.0 - t) < f32::MIN {
            1.0
        } else {
            2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * std::f32::consts::FRAC_PI_3 * 2.0).sin()
                + 1.0
        }
    }
    pub fn inout_elastic(t: f32) -> f32 {
        if t < f32::MIN {
            0.0
        } else if (1.0 - t) < f32::MIN {
            1.0
        } else if t < 0.5 {
            -(2f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * std::f32::consts::PI * 2.0 / 4.5))
                / 2.0
        } else {
            (2f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * std::f32::consts::PI * 2.0 / 4.5))
                / 2.0
                + 1.0
        }
    }
    pub fn in_bounce(t: f32) -> f32 {
        1.0 - out_bounce(1.0 - t)
    }
    pub fn out_bounce(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;

        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t = t - 1.5 / D1;
            N1 * t * t + 0.75
        } else if t < 2.5 / D1 {
            let t = t - 2.25 / D1;
            N1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / D1;
            N1 * t * t + 0.984375
        }
    }
    pub fn inout_bounce(t: f32) -> f32 {
        if t < 0.5 {
            (1.0 - out_bounce(1.0 - 2.0 * t)) / 2.0
        } else {
            (1.0 + out_bounce(2.0 * t - 1.0)) / 2.0
        }
    }

    pub struct InPoly(pub i32);
    impl Curve for InPoly {
        fn eval(&self, t: f32) -> f32 {
            t.powi(self.0)
        }
    }
    pub struct OutPoly(pub i32);
    impl Curve for OutPoly {
        fn eval(&self, t: f32) -> f32 {
            1.0 - (1.0 - t).powi(self.0)
        }
    }
    pub struct InOutPoly(pub i32);
    impl Curve for InOutPoly {
        fn eval(&self, t: f32) -> f32 {
            if t < 0.5 {
                (std::f32::consts::SQRT_2 * t).powi(self.0)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(self.0) / 2.0
            }
        }
    }
}

pub struct ChainCurve<O: Sized + Curve, I: Sized + Curve> {
    outer: O,
    inner: I,
}

impl<O: Curve, I: Curve> Curve for ChainCurve<O, I> {
    fn eval(&self, t: f32) -> f32 {
        self.outer.eval(self.inner.eval(t))
    }
}
pub struct TransitionCurve<T: Curve, A: Curve, B: Curve> {
    a: A,
    b: B,
    t: T,
}

impl<T: Curve, A: Curve, B: Curve> Curve for TransitionCurve<T, A, B> {
    fn eval(&self, t: f32) -> f32 {
        self.t.eval(t).lerp(self.a.eval(t), self.b.eval(t))
    }
}

pub struct AddCurve<A: Curve, B: Curve> {
    a: A,
    b: B,
}

impl<A: Curve, B: Curve> Curve for AddCurve<A, B> {
    fn eval(&self, t: f32) -> f32 {
        self.a.eval(t) + self.b.eval(t)
    }
}

pub struct SubCurve<A: Curve, B: Curve> {
    a: A,
    b: B,
}

impl<A: Curve, B: Curve> Curve for SubCurve<A, B> {
    fn eval(&self, t: f32) -> f32 {
        self.a.eval(t) - self.b.eval(t)
    }
}

pub struct MulCurve<A: Curve, B: Curve> {
    a: A,
    b: B,
}

impl<A: Curve, B: Curve> Curve for MulCurve<A, B> {
    fn eval(&self, t: f32) -> f32 {
        self.a.eval(t) * self.b.eval(t)
    }
}

pub struct PowCurve<A: Curve, B: Curve> {
    a: A,
    b: B,
}

impl<A: Curve, B: Curve> Curve for PowCurve<A, B> {
    fn eval(&self, t: f32) -> f32 {
        self.a.eval(t).powf(self.b.eval(t))
    }
}

pub trait Curve {
    fn eval(&self, t: f32) -> f32;
}

pub trait CurveLerp: Curve {
    fn lerp<T: Mul<f32, Output = T> + Add<T, Output = T> + Sized>(&self, a: T, b: T, t: f32) -> T {
        let t = self.eval(t);
        a * (1.0 - t) + b * t
    }
}

pub struct ConstCurve(f32);

trait IntoConstCurve {
    fn curve(self) -> ConstCurve;
}

impl<T: Into<f32>> IntoConstCurve for T {
    fn curve(self) -> ConstCurve {
        ConstCurve(self.into())
    }
}

impl Curve for ConstCurve {
    fn eval(&self, t: f32) -> f32 {
        self.0
    }
}

pub trait IntoCurve: Sized + Curve {
    fn chain<I: Curve>(self, inner: I) -> ChainCurve<Self, I> {
        ChainCurve { outer: self, inner }
    }
    fn repeat(self) -> ChainCurve<Self, fn(f32) -> f32> {
        self.chain(standard::repeat)
    }
    fn clamp(self) -> ChainCurve<Self, fn(f32) -> f32> {
        self.chain(standard::clamp)
    }
    fn pingpong(self) -> ChainCurve<Self, fn(f32) -> f32> {
        self.chain(standard::pingpong)
    }
    fn reverse(self) -> ChainCurve<Self, fn(f32) -> f32> {
        self.chain(standard::reverse)
    }
    fn transition<A: Curve, B: Curve>(self, a: A, b: B) -> TransitionCurve<Self, A, B> {
        TransitionCurve { a, b, t: self }
    }
    fn add<T: Curve>(self, b: T) -> AddCurve<Self, T> {
        AddCurve { a: self, b }
    }
    fn sub<T: Curve>(self, b: T) -> SubCurve<Self, T> {
        SubCurve { a: self, b }
    }
    fn mul<T: Curve>(self, b: T) -> MulCurve<Self, T> {
        MulCurve { a: self, b }
    }
    fn pow<T: Curve>(self, b: T) -> PowCurve<Self, T> {
        PowCurve { a: self, b }
    }
}

impl<T: Curve + Sized> IntoCurve for T {}

impl<F: Fn(f32) -> f32> Curve for F {
    fn eval(&self, t: f32) -> f32 {
        (self)(t)
    }
}
