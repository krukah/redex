use units::*;

use crate::*;

/// Options for one tween. Unset fields take the primitive's defaults, then `BEAT`, no delay,
/// `Ease::Move`. `keep` leaves the final frame applied; otherwise the animation is cancelled when
/// it ends, so the element returns to its own styles.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Tween {
    ms: Option<Ms>,
    delay: Option<Ms>,
    ease: Option<Ease>,
    keep: bool,
}

impl Tween {
    pub fn ms(self, ms: Ms) -> Self {
        Self { ms: Some(ms), ..self }
    }

    pub fn ease(self, ease: Ease) -> Self {
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
        let (
            Self { ms, delay, ease, keep },
            Self {
                ms: m,
                delay: d,
                ease: e,
                keep: k,
            },
        ) = (self, base);
        Self {
            ms: ms.or(m),
            delay: delay.or(d),
            ease: ease.or(e),
            keep: keep || k,
        }
    }

    pub fn duration(&self) -> Ms {
        let Self { ms, .. } = self;
        ms.unwrap_or(Motion::BEAT)
    }

    pub fn wait(&self) -> Ms {
        let Self { delay, .. } = self;
        delay.unwrap_or(Ms::ZERO)
    }

    pub fn easing(&self) -> Ease {
        let Self { ease, .. } = self;
        ease.unwrap_or(Ease::Move)
    }

    pub fn keeps(&self) -> bool {
        self.keep
    }
}
