use kerotakis_core::inference::*;
struct Model;
impl Hypothesis for Model {
    fn name(&self) -> &str {
        "test"
    }
    fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        ((0.0, 2.0), (0.0, 2.0))
    }
    fn predict(&self, a: f64, b: f64) -> Vec<(&'static str, f64)> {
        vec![("x", a), ("y", b)]
    }
}
#[test]
fn repeated_observations_all_contribute() {
    let obs = [
        Observation::new("x", 0.0, 1.0).unwrap(),
        Observation::new("x", 2.0, 1.0).unwrap(),
    ];
    let fit = fit_two_parameters(&Model, &obs, 3).unwrap();
    assert_eq!(fit.best_a, 1.0);
    assert_eq!(fit.chi_squared, 2.0);
}
#[test]
fn invalid_public_observations_are_rejected() {
    for sigma in [0.0, -1.0, f64::NAN] {
        assert!(fit_two_parameters(
            &Model,
            &[Observation {
                observable: "x".into(),
                value: 1.0,
                sigma
            }],
            3
        )
        .is_err());
    }
}
struct Broken(bool);
impl Hypothesis for Broken {
    fn name(&self) -> &str {
        "broken"
    }
    fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        Model.bounds()
    }
    fn predict(&self, a: f64, _b: f64) -> Vec<(&'static str, f64)> {
        if a == 1.0 {
            if self.0 {
                vec![]
            } else {
                vec![("x", f64::NAN)]
            }
        } else {
            vec![("x", a)]
        }
    }
}
#[test]
fn missing_or_nonfinite_predictions_cannot_win_a_fit() {
    for missing in [true, false] {
        assert!(fit_two_parameters(
            &Broken(missing),
            &[Observation::new("x", 1.0, 1.0).unwrap()],
            3
        )
        .is_err());
    }
}
