use flo_script::*;
use flo_script::gluon_host::*;

use futures::stream;
use futures::executor;

#[test]
pub fn read_input_stream_as_output() {
    let host                = GluonScriptHost::new();
    let input_x             = FloScriptSymbol::with_name("x");

    host.editor().set_input_type::<i32>(input_x);

    // Start reading the stream before attaching some output
    let mut output_x_stream = executor::spawn(host.notebook().receive_output::<i32>(input_x).expect("output stream"));

    // Send some data to the input
    let input_data          = stream::iter_ok::<_, ()>(vec![1, 2, 3]);
    host.notebook().attach_input(input_x, input_data).expect("attaching input");

    // Should be able to read the items from the input stream
    assert!(output_x_stream.wait_stream() == Some(Ok(1)));
    assert!(output_x_stream.wait_stream() == Some(Ok(2)));
    assert!(output_x_stream.wait_stream() == Some(Ok(3)));
    assert!(output_x_stream.wait_stream() == None);
}
