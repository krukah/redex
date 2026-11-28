use std::future::Future;
use std::pin::Pin;

use units::*;
use wasm_bindgen::prelude::*;
use web_sys::DomRect;
use web_sys::Element;

use crate::*;

/// Something that plays at a `Clock` and resolves when it is done. It always resolves: each
/// primitive also races a timer, so a throttled or paused timeline never leaves the page half
/// animated. Nothing is ever scaled; a change of shape is two copies that crossfade.
pub struct Motion(Box<dyn FnOnce(Clock) -> Pin<Box<dyn Future<Output = ()>>>>);

impl Motion {
    /// How long past its end a tween waits for a stalled animation before it resolves anyway. The
    /// step never waits on it: its phases end at deadlines (`Schedule::until`).
    const SLACK: Ms = Ms::new(60.0);

    pub fn new<F>(play: F) -> Self
    where
        F: FnOnce(Clock) -> Pin<Box<dyn Future<Output = ()>>> + 'static,
    {
        Self(Box::new(play))
    }

    /// A motion already playing, to compose with others that start later.
    pub fn begun(play: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        Self::new(move |_| play)
    }

    pub fn none() -> Self {
        Self::new(|_| Box::pin(async {}))
    }

    /// Starts the motion now; the future resolves when it is done.
    pub fn play(self, clock: Clock) -> Pin<Box<dyn Future<Output = ()>>> {
        (self.0)(clock)
    }

    /// False when the reader asked for reduced motion. Every primitive checks it when it plays.
    pub fn enabled() -> bool {
        !web_sys::window()
            .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
            .as_ref()
            .is_some_and(web_sys::MediaQueryList::matches)
    }

    /// Resolves after `ms`, as `setTimeout` counts them.
    pub fn wait(ms: Ms) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(gloo_timers::future::TimeoutFuture::new(ms.whole()))
    }

    /// The one place that calls `Element::animate`.
    fn tween(el: &Element, frames: &[&[(Prop, JsValue)]], o: Tween) -> Self {
        let el = el.clone();
        let frames = frames
            .iter()
            .map(|frame| {
                let obj = js_sys::Object::new();
                frame.iter().for_each(|(k, v)| {
                    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(k.key()), v);
                });
                obj
            })
            .collect::<js_sys::Array>();
        Self::new(move |clock| {
            if !Self::enabled() {
                return Box::pin(async {});
            }
            let (duration, delay) = (clock.scale(o.duration()), clock.scale(o.wait()));
            let opts = js_sys::Object::new();
            [
                ("duration", JsValue::from_f64(duration.get())),
                ("delay", JsValue::from_f64(delay.get())),
                ("easing", o.easing().to_string().into()),
                ("fill", "both".into()),
            ]
            .iter()
            .for_each(|(k, v)| {
                let _ = js_sys::Reflect::set(&opts, &JsValue::from_str(k), v);
            });
            let a = Self::call(&el, "animate", &[frames.clone().into(), opts.into()]);
            let finished = js_sys::Reflect::get(&a, &"finished".into())
                .ok()
                .and_then(|p| p.dyn_into::<js_sys::Promise>().ok())
                .map_or_else(|| Box::pin(async {}) as Pin<Box<dyn Future<Output = ()>>>, |p| Box::pin(Settle::new(&p)));
            Box::pin(async move {
                First::new(finished, Self::wait(delay + duration + Self::SLACK)).await;
                if !o.keeps() {
                    Self::call(&a, "cancel", &[]);
                }
            })
        })
    }

    /// Calls a method by name: `Element::animate` sits behind web-sys's unstable APIs.
    fn call(this: &JsValue, name: &str, args: &[JsValue]) -> JsValue {
        js_sys::Reflect::get(this, &JsValue::from_str(name))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .and_then(|f| f.apply(this, &args.iter().collect::<js_sys::Array>()).ok())
            .unwrap_or(JsValue::UNDEFINED)
    }

    fn translate(dx: Px, dy: Px) -> JsValue {
        JsValue::from_str(&format!("translate({dx}px, {dy}px)"))
    }

    /// Fade in where it stands.
    pub fn enter(el: &Element, o: Tween) -> Self {
        Self::tween(
            el,
            &[&[(Prop::Opacity, 0.into())], &[(Prop::Opacity, 1.into())]],
            o.over(Tween::default().ease(Ease::Enter)),
        )
    }

    /// Fade out where it stands, then remove it.
    pub fn exit(el: &Element, o: Tween) -> Self {
        let gone = el.clone();
        Self::seq(vec![
            Self::tween(
                el,
                &[
                    &[(
                        Prop::Opacity,
                        JsValue::from_str(
                            &web_sys::window()
                                .and_then(|w| w.get_computed_style(el).ok().flatten())
                                .and_then(|cs| cs.get_property_value(Prop::Opacity.key()).ok())
                                .unwrap_or_default(),
                        ),
                    )],
                    &[(Prop::Opacity, 0.into())],
                ],
                o.over(Schedule::part(Role::Fade).ease(Ease::Exit)).keep(),
            ),
            Self::act(move || gone.remove()),
        ])
    }

    /// Play back from an offset to where layout put it (FLIP).
    pub fn slide(el: &Element, dx: Px, dy: Px, o: Tween) -> Self {
        if dx.still() && dy.still() {
            return Self::none();
        }
        Self::tween(
            el,
            &[
                &[(Prop::Transform, Self::translate(dx, dy))],
                &[(Prop::Transform, "none".into())],
            ],
            o,
        )
    }

    /// Grow or shrink its box between two rects.
    pub fn resize(el: &Element, from: &DomRect, to: &DomRect, o: Tween) -> Self {
        if (from.width() - to.width()).abs() < 0.5 && (from.height() - to.height()).abs() < 0.5 {
            return Self::none();
        }
        let px = |v: f64| JsValue::from_str(&format!("{v}px"));
        Self::tween(
            el,
            &[
                &[(Prop::Width, px(from.width())), (Prop::Height, px(from.height()))],
                &[(Prop::Width, px(to.width())), (Prop::Height, px(to.height()))],
            ],
            o,
        )
    }

    /// Stroke an SVG path from start to end (it needs `pathLength="1"` and a dash array of 1).
    pub fn draw(path: &Element, o: Tween) -> Self {
        Self::tween(
            path,
            &[
                &[(Prop::StrokeDashoffset, 1.into())],
                &[(Prop::StrokeDashoffset, 0.into())],
            ],
            o.over(Tween::default().ease(Ease::Enter)),
        )
    }

    /// Travel in a straight line by (dx, dy) and stay there.
    pub fn fly(el: &Element, dx: Px, dy: Px, o: Tween) -> Self {
        Self::tween(
            el,
            &[
                &[(Prop::Transform, "none".into())],
                &[(Prop::Transform, Self::translate(dx, dy))],
            ],
            o.keep(),
        )
    }

    /// `a` fades out while `b` fades in; both stay.
    pub fn crossfade(a: &Element, b: &Element, o: Tween) -> Self {
        let o = o.over(Schedule::part(Role::Swap).ease(Ease::Exit)).keep();
        Self::all(vec![
            Self::tween(a, &[&[(Prop::Opacity, 1.into())], &[(Prop::Opacity, 0.into())]], o),
            Self::tween(b, &[&[(Prop::Opacity, 0.into())], &[(Prop::Opacity, 1.into())]], o),
        ])
    }

    /// Play together.
    pub fn all(ms: Vec<Motion>) -> Self {
        Self::new(move |clock| Box::pin(Join::new(ms.into_iter().map(|m| m.play(clock)).collect())))
    }

    /// Play one after another.
    pub fn seq(ms: Vec<Motion>) -> Self {
        Self::new(move |clock| {
            let mut rest = ms.into_iter();
            let first = rest.next().map(|m| m.play(clock));
            Box::pin(async move {
                if let Some(f) = first {
                    f.await;
                }
                for m in rest {
                    m.play(clock).await;
                }
            })
        })
    }

    /// Start `m` after a delay.
    pub fn after(ms: Ms, m: Motion) -> Self {
        if ms <= Ms::ZERO {
            return m;
        }
        Self::new(move |clock| {
            Box::pin(async move {
                Self::wait(clock.scale(ms)).await;
                m.play(clock).await;
            })
        })
    }

    /// Play together, each one `gap` later than the one before; a gap in the list still counts.
    pub fn stagger(gap: Ms, ms: Vec<Option<Motion>>) -> Self {
        Self::all(
            ms.into_iter()
                .enumerate()
                .filter_map(|(i, m)| m.map(|m| Self::after(gap * i as f64, m)))
                .collect(),
        )
    }

    /// A side effect at a point in a sequence.
    pub fn act<F>(f: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        Self::new(move |_| {
            f();
            Box::pin(async {})
        })
    }
}
