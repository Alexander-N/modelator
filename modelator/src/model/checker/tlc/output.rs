use crate::artifact::tla_trace::{TlaState, TlaTrace};
use crate::model::checker::ModelCheckerRuntime;
use crate::Error;

use std::collections::HashMap;

/// Parses all tla traces from a .tla trace generated by tlc.
pub(crate) fn parse_traces(
    output: &str,
    runtime: &ModelCheckerRuntime,
) -> Result<Vec<TlaTrace>, Error> {
    let mut parsed_output: HashMap<u8, HashMap<usize, Vec<String>>> = HashMap::new();

    let mut curr_message_id = None;
    let mut curr_message = String::new();

    output.lines().for_each(|line| {
        if line.starts_with("@!@!@STARTMSG ") {
            // without annotattion
            let (code, class) = curr_message_id.take().unwrap_or((0, 0));
            parsed_output
                .entry(class)
                .or_default()
                .entry(code)
                .or_default()
                .push(curr_message.drain(..).collect());

            let (code, class) = line.splitn(3, ' ').nth(1).unwrap().split_once(':').unwrap();
            curr_message_id.insert((code.parse().unwrap(), class.parse().unwrap()));
        } else if line.starts_with("@!@!@ENDMSG ") {
            let (code, class) = curr_message_id.take().unwrap_or((0, 0));
            parsed_output
                .entry(class)
                .or_default()
                .entry(code)
                .or_default()
                .push(curr_message.drain(..).collect());

            let c_code = line.splitn(3, ' ').nth(1).unwrap();
            assert_eq!(code, c_code.parse::<usize>().unwrap());
        } else {
            curr_message.push_str(line);
            curr_message.push('\n');
        }
    });

    if let Some(lines) = parsed_output.get(&4).and_then(|x| x.get(&2217)) {
        let mut traces = Vec::new();
        let mut trace = None;
        for line in lines {
            if line.starts_with("1: <Initial predicate>") {
                // start of new trace
                if let Some(t) = trace.take() {
                    traces.push(t);
                }
                trace = Some(TlaTrace::new());
            }
            if let Some(t) = trace.as_mut() {
                t.add(line.split_once('\n').unwrap().1.into());
            }
        }
        // last trace
        if let Some(t) = trace.take() {
            traces.push(t);
        }
        Ok(traces)
    } else if let Some(errors) = parsed_output.get(&1) {
        // Message Codes ref
        // https://github.com/tlaplus/tlaplus/blob/master/tlatools/org.lamport.tlatools/src/tlc2/output/EC.java
        // Message Classes ref
        // https://github.com/tlaplus/tlaplus/blob/master/tlatools/org.lamport.tlatools/src/tlc2/output/MP.java
        // NONE = 0; ERROR = 1; TLCBUG = 2; WARNING = 3; STATE = 4;

        let message = errors
            .iter()
            .map(|(code, message)| {
                format!(
                    "[{}:{}]: {}",
                    runtime.log.to_string_lossy(),
                    code,
                    &message
                        .iter()
                        .map(|x| x.trim().replace("\n", " "))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Err(Error::TLCFailure(message))
    } else {
        Ok(vec![])
    }
}