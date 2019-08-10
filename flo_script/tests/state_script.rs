use flo_script::*;
use flo_script::gluon_host::*;

use futures::stream;
use futures::executor;

#[test]
fn read_input_stream_as_state() {
    let host                = GluonScriptHost::new();
    let input_x             = FloScriptSymbol::with_name("x");

    host.editor().set_input_type::<i32>(input_x);

    // Start reading the stream before attaching some output
    let mut output_x_stream = executor::spawn(host.notebook().receive_output_state::<i32>(input_x).expect("output state"));

    // Send some data to the input
    let input_data          = stream::iter_ok::<_, ()>(vec![1, 2, 3]);
    host.notebook().attach_input(input_x, input_data).expect("attaching input");

    // Only the most recent state is considered 'interesting' so we should just read '3' here
    assert!(output_x_stream.wait_stream() == Some(Ok(3)));
    assert!(output_x_stream.wait_stream() == None);
}

#[test]
fn update_from_state_stream() {
    // Declare some symbols
    let input_x             = FloScriptSymbol::with_name("x");
    let output_y            = FloScriptSymbol::with_name("y");

    // Create the script host
    let host                = GluonScriptHost::new();
    let editor              = host.editor();

    // 'x' is an input state stream, 'y' is a state that adds one to the current state of 'x'
    editor.clear();
    editor.set_input_type::<i32>(input_x);
    editor.set_computing_script(output_y, r#"
            let state = import! flo.script.state
            do x = state.x()
            x + 1
        "#);

    // Get the stream from our state
    let mut output_stream   = executor::spawn(host.notebook().receive_output::<i32>(output_y).expect("output stream"));

    // Send a state update
    host.notebook().attach_input(input_x, stream::iter_ok(vec![3])).expect("attached input");

    // Should receive a state update of '4' for our output state
    let next_state_y        = output_stream.wait_stream().expect("at least one update").expect("no errors");
    assert!(next_state_y == 4);
}
