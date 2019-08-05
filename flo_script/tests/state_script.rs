use flo_script::*;
use flo_script::gluon_host::*;

use std::any::*;
use futures::stream;
use futures::executor;

#[test]
fn update_from_state_stream() {
    // Declare some symbols
    let input_x             = FloScriptSymbol::with_name("x");
    let output_y            = FloScriptSymbol::with_name("y");

    // Create the script host
    let host                = GluonScriptHost::new();

    // 'x' is an input state stream, 'y' is a state that adds one to the current state of 'x'
    let edits               = vec![
        //ScriptEdit::Clear,
        ScriptEdit::SetInputType(input_x, TypeId::of::<i32>()),
        ScriptEdit::SetComputingScript(output_y, String::from(r#"
            let state = import! flo.script.state
            do x = state.x()
            x + 1
        "#))
    ];

    // Send the edits to the script host
    let mut run_edits       = executor::spawn(host.editor().send_edits(stream::iter_ok(edits)));
    run_edits.wait_future().unwrap();

    // Get the stream from our state
    let mut output_stream   = executor::spawn(host.notebook().receive_output::<i32>(output_y));

    // Send a state update
    host.notebook().attach_input(input_x, stream::iter_ok(vec![3]));

    // Should receive a state update of '4' for our output state
    let next_state_y        = output_stream.wait_stream().expect("at least one update").expect("no errors");
    assert!(next_state_y == 4);
}
