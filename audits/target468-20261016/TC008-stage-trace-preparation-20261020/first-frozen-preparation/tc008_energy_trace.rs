//! Preparation only. Observe real original TC008 calls; do not certify energy here.
//! Stage in kerotakis-cli/tests, with the exact original fixture beside it.
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use serde_json::{json, Value};
use std::cell::{Cell, RefCell};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

struct Trace {
    file: RefCell<File>,
    sequence: Cell<u64>,
    command: Cell<usize>,
    call: Cell<u64>,
}
impl Trace {
    fn record(&self, mut value: Value) {
        let sequence = self.sequence.get();
        self.sequence.set(sequence + 1);
        value["sequence"] = json!(sequence);
        value["command_index"] = json!(self.command.get());
        let mut file = self.file.borrow_mut();
        serde_json::to_writer(&mut *file, &value).expect("write real trace");
        file.write_all(b"\n").expect("trace newline");
        file.flush().expect("preserve completed trace records");
    }
}
fn snapshot(vessel: &Vessel) -> Value {
    json!({
        "vessel": vessel,
        // These fields are intentionally serde-skipped in persisted Vessel.
        // They are observed directly, never inferred from a saved solution.
        "aqueous_heat_ownership": format!("{:?}", vessel.aqueous_heat),
        "heat_input": vessel.heat_input.as_ref().map(|heat| json!({
            "contents": heat.contents,
            "temperature": heat.temperature,
            "delivered_j": heat.delivered_j,
        })),
    })
}
struct Observed<T> {
    inner: T,
    boundary: &'static str,
    trace: Rc<Trace>,
}
impl<T: Equilibrator> Observed<T> {
    fn new(inner: T, boundary: &'static str, trace: &Rc<Trace>) -> Self {
        Self {
            inner,
            boundary,
            trace: Rc::clone(trace),
        }
    }
}
impl<T: Equilibrator> Equilibrator for Observed<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    fn trusted_aqueous_readback(
        &mut self,
    ) -> Option<&mut dyn kerotakis_core::aqueous_continuation::TrustedAqueousReadback> {
        self.inner.trusted_aqueous_readback()
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        self.inner.element_conservation_tolerance()
    }
    fn route_kind(&self) -> kerotakis_core::solve::SolverRouteKind {
        self.inner.route_kind()
    }
    fn applies(&self, vessel: &Vessel) -> bool {
        self.inner.applies(vessel)
    }
    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.inner.chemistry_applies(vessel)
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let call = self.trace.call.get();
        self.trace.call.set(call + 1);
        self.trace.record(json!({"kind":"adapter_enter", "boundary":self.boundary, "call":call, "state":snapshot(vessel)}));
        // Exactly ONE real invocation, with the original mutable vessel.
        // No fixture result, cloned trial solve, model change or retry.
        let result = self.inner.equilibrate(vessel);
        match &result {
            Ok(events) => self.trace.record(json!({"kind":"adapter_exit", "boundary":self.boundary, "call":call, "state":snapshot(vessel), "events":events, "result":"ok"})),
            Err(error) => self.trace.record(json!({"kind":"adapter_exit", "boundary":self.boundary, "call":call, "state":snapshot(vessel), "error":error.to_string(), "result":"error"})),
        }
        result
    }
    fn record_route(&mut self, route: kerotakis_core::solve::SolverRoute) {
        self.inner.record_route(route);
    }
    fn time_boundaries(&self, vessel: &Vessel) -> Vec<Event> {
        self.inner.time_boundaries(vessel)
    }
    fn mix(
        &mut self,
        vessel: &mut Vessel,
        a: &Vessel,
        fraction_a: f64,
        b: &Vessel,
        fraction_b: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        self.trace.record(json!({"kind":"unexpected_non_equilibrate_call", "method":"mix", "boundary":self.boundary}));
        self.inner.mix(vessel, a, fraction_a, b, fraction_b)
    }
    fn equilibrate_delta(
        &mut self,
        vessel: &Vessel,
    ) -> Result<(kerotakis_core::delta::StateDelta, Vec<Event>), SolveError> {
        self.trace.record(json!({"kind":"unexpected_non_equilibrate_call", "method":"equilibrate_delta", "boundary":self.boundary}));
        self.inner.equilibrate_delta(vessel)
    }
    fn capability(&self, vessel: &Vessel) -> kerotakis_core::solve::CapabilityReport {
        self.inner.capability(vessel)
    }
}
#[test]
fn original_tc008_records_actual_stage_states_without_energy_verdict() {
    let folder = PathBuf::from(
        std::env::var_os("KERO_TC008_TRACE_DIR").expect("explicit fresh owned trace directory"),
    );
    assert!(!folder.exists(), "never overwrite an earlier trace");
    std::fs::create_dir_all(&folder).expect("fresh trace directory");
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(folder.join("tc008-trace.jsonl"))
        .expect("new trace file");
    let trace = Rc::new(Trace {
        file: RefCell::new(file),
        sequence: Cell::new(0),
        command: Cell::new(0),
        call: Cell::new(0),
    });
    let script = include_str!("fixtures/tc008-original.lab");
    trace.record(json!({"kind":"trace_design", "script":script, "scope":"actual original stage observation only; no energy success claim", "stack":"standard_solvers + Phase(Displacement(real Phreeqc))", "private_heat_branch":"not observable through Equilibrator; do not invent whole-step/fallback selection"}));
    let aqueous = Observed::new(
        PhreeqcEquilibrator::new().expect("required real native engine"),
        "aqueous",
        &trace,
    );
    let displacement = Observed::new(
        DisplacementEquilibrator::wrapping(Box::new(aqueous)),
        "displacement",
        &trace,
    );
    let phase = Observed::new(
        PhaseEquilibrator::wrapping(Box::new(displacement)),
        "phase_coupled",
        &trace,
    );
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(phase)]));
    let mut bench = Bench::new();
    let mut commands = 0;
    for (index, line) in script.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        trace.command.set(index + 1); // Original physical script line, never renumbered.
        trace.record(json!({"kind":"command_enter", "text":line, "bench":&bench}));
        match parse_op(line).expect("official original parser") {
            Some(operator) => {
                let events = bench
                    .step_with(
                        operator.clone(),
                        &mut stack,
                        &kerotakis_safety::ReactiveGroupScreen,
                    )
                    .expect("original bench step")
                    .to_vec();
                trace.record(json!({"kind":"command_exit", "text":line, "operator":operator, "events":events, "bench":&bench}));
            }
            None => {
                // Original register is presentation only; inspect reads the
                // existing bench and never advances chemistry or bench time.
                assert!(line == "register lv3" || line.starts_with("inspect"));
                trace.record(
                    json!({"kind":"command_exit", "text":line, "meta_only":true, "bench":&bench}),
                );
            }
        }
        commands += 1;
    }
    assert_eq!(
        commands, 9,
        "all original noncomment commands, including register/inspects"
    );
    trace.record(json!({"kind":"final_bench", "bench":&bench, "energy_acceptance":"UNADJUDICATED; separate independent saved-stage ledger required"}));
    trace
        .file
        .borrow()
        .sync_all()
        .expect("durable completed trace");
}
