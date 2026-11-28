use motion::*;
use units::*;

/// A schedule takes no rule, no term and no piece count, so a step's length cannot depend on them;
/// this pins what that length is.
#[test]
fn every_step_lasts_the_same() {
    [Schedule::STEP, Schedule::RUN].into_iter().for_each(|s| {
        assert_eq!(s.step(), Phase::ALL.iter().fold(Ms::ZERO, |sum, p| sum + s.span(*p)));
    });
    assert_eq!(Schedule::STEP.step(), Ms::new(1420.0));
    assert_eq!(Schedule::RUN.step(), Ms::new(710.0));
}

#[test]
fn every_part_runs_inside_its_phase() {
    Role::ALL.into_iter().for_each(|r| {
        let (part, from) = (Schedule::part(r), Schedule::since_move(r.phase()));
        assert!(part.wait() >= from, "{r:?}");
        assert!(part.wait() + part.duration() <= from + r.phase().ms(), "{r:?}");
    });
}

#[test]
fn the_move_has_no_stagger() {
    Role::ALL
        .into_iter()
        .filter(|r| r.phase() == Phase::Move)
        .for_each(|r| assert_eq!(Schedule::part(r).wait(), Ms::ZERO, "{r:?}"));
}

#[test]
fn nothing_plays_during_the_hold() {
    Role::ALL
        .into_iter()
        .for_each(|r| assert_ne!(r.phase(), Phase::Hold, "{r:?}"));
}

#[test]
fn phases_end_one_after_another() {
    [Schedule::STEP, Schedule::RUN].into_iter().for_each(|s| {
        assert_eq!(s.end(Phase::Hold), s.span(Phase::Hold));
        assert_eq!(s.end(Phase::Move), s.span(Phase::Hold) + s.span(Phase::Move));
        assert_eq!(s.end(Phase::Settle), s.step());
    });
}
