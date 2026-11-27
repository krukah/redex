use std::future::Future;
use std::pin::Pin;

use units::*;

use crate::*;

/// A step's timeline at a clock: the three phases, and the window of each part of the move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Schedule(Clock);

impl Schedule {
    /// One step at a time, at reading pace.
    pub const STEP: Schedule = Schedule(Clock::new(1.0));
    /// Normalize and To WHNF: every step at the same, quicker cadence.
    pub const RUN: Schedule = Schedule(Clock::new(2.0));

    pub fn clock(self) -> Clock {
        self.0
    }

    /// A phase's length at this clock.
    pub fn span(self, phase: Phase) -> Ms {
        self.clock().scale(phase.ms())
    }

    /// A whole step: the phases end to end.
    pub fn step(self) -> Ms {
        Phase::ALL.iter().fold(Ms::ZERO, |sum, p| sum + self.span(*p))
    }

    /// When a phase ends, counted from the start of its step.
    pub fn end(self, phase: Phase) -> Ms {
        Phase::ALL
            .iter()
            .take_while(|p| **p != phase)
            .fold(self.span(phase), |sum, p| sum + self.span(*p))
    }

    /// The time now on the page's clock (`performance.now()`).
    pub fn now() -> Ms {
        Ms::new(
            web_sys::window()
                .as_ref()
                .and_then(web_sys::Window::performance)
                .map_or(0.0, |p| p.now()),
        )
    }

    /// Waits until a phase of the step that began at `start` ends, so work done inside a phase
    /// (drawing the new term, pinning ghosts) never lengthens the step.
    pub fn until(self, start: Ms, phase: Phase) -> Pin<Box<dyn Future<Output = ()>>> {
        Motion::wait(start + self.end(phase) - Self::now())
    }

    /// A part's tween options, written at speed 1 as `Motion` expects; the clock scales them when
    /// the move plays.
    pub fn part(role: Role) -> Tween {
        let (start, length) = role.window();
        Tween::default()
            .ms(Phase::Move.ms() * length)
            .delay(Phase::Move.ms() * start)
    }
}
