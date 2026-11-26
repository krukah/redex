/// Options for one tween. Unset fields take the primitive's defaults, then `BEAT`, no delay,
/// `Ease::MOVE`. `keep` leaves the final frame applied; otherwise the animation is cancelled when
/// it ends, so the element returns to its own styles.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tween {
    ms: Option<f64>,
    delay: Option<f64>,
    ease: Option<&'static str>,
    keep: bool,
}

impl Tween {
    pub fn ms(self, ms: f64) -> Self {
        Self { ms: Some(ms), ..self }
    }

    pub fn delay(self, delay: f64) -> Self {
        Self {
            delay: Some(delay),
            ..self
        }
    }

    pub fn ease(self, ease: &'static str) -> Self {
        Self {
            ease: Some(ease),
            ..self
        }
    }

    pub fn keep(self) -> Self {
        Self { keep: true, ..self }
    }

    /// These options over a primitive's defaults: what the caller set wins.
    pub fn over(self, base: Tween) -> Self {
        Self {
            ms: self.ms.or(base.ms),
            delay: self.delay.or(base.delay),
            ease: self.ease.or(base.ease),
            keep: self.keep || base.keep,
        }
    }

    pub fn duration(&self) -> f64 {
        self.ms.unwrap_or(crate::Motion::BEAT)
    }

    pub fn wait(&self) -> f64 {
        self.delay.unwrap_or(0.0)
    }

    pub fn easing(&self) -> &'static str {
        self.ease.unwrap_or(crate::Ease::MOVE)
    }

    pub fn keeps(&self) -> bool {
        self.keep
    }
}
