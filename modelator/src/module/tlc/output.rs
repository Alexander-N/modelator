use crate::artifact::{TlaState, TlaTrace};
use crate::{Error, ModelCheckerOptions};

pub(crate) fn parse(output: String, options: &ModelCheckerOptions) -> Result<Vec<TlaTrace>, Error> {
    let mut traces = Vec::new();

    // skip all lines until after the message "Computing initial states..."
    let mut lines = output
        .lines()
        .skip_while(|line| !line.starts_with("@!@!@ENDMSG 2189 @!@!@"));

    // after this, we have two possibilities, depending on whether the invariant
    // was violated in the initial state, or in the following states
    todo!();

    while let Some(line) = lines.next() {
        // check if found the beginning of the next trace
        if line.starts_with("@!@!@ENDMSG 2121 @!@!@") {
            if let Some(trace) = parse_trace(&mut lines, options)? {
                traces.push(trace);
            }
        }
    }
    Ok(traces)
}

fn parse_trace<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    options: &ModelCheckerOptions,
) -> Result<Option<TlaTrace>, Error> {
    let mut state_index = 0;
    let mut state = None;
    let mut trace = TlaTrace::new();
    loop {
        let line = lines
            .next()
            .ok_or_else(|| Error::InvalidTLCOutput(options.log.clone()))?;

        // check if found the start of the next state
        if line.starts_with("@!@!@STARTMSG 2217:4 @!@!@") {
            state_index += 1;
            continue;
        }

        // check if found the end of the current state
        if line.starts_with("@!@!@ENDMSG 2217 @!@!@") {
            let state = state
                .take()
                .expect("[modelator] a trace state should have been started");
            trace.add(state);
            continue;
        }

        // any other tag means that this trace has ended
        if line.starts_with("@!@!@STARTMSG") {
            break;
        }

        // check if the next state is starting
        if line.starts_with(&format!("{}: ", state_index)) {
            // start next state
            assert!(
                state.is_none(),
                "[modelator] previous trace state has not ended yet"
            );
            state = Some(TlaState::new());
            continue;
        }

        // save any remaining line
        let state = state
            .as_mut()
            .expect("[modelator] trace state should have been started");
        state.push_str(line);
        state.push('\n');
    }

    let trace = if trace.is_empty() { None } else { Some(trace) };
    Ok(trace)
}
