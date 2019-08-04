#[macro_use] extern crate gluon_vm;

use gluon::*;
use gluon::vm::ExternModule;
use gluon::import;

#[test]
fn threads_have_same_modules() {
    // In Gluon: importing a module replaces an existing module and also applies across all threads created for the same VM
    let root_thread = new_vm();

    // Create two threads
    let thread1     = root_thread.new_thread().unwrap();
    let thread2     = root_thread.new_thread().unwrap();

    // Import two modules into them (same name in both threads, but different modules)
    fn module1(thread: &Thread) -> vm::Result<ExternModule> {
        ExternModule::new(thread, record! {
            module => primitive!(1, |_: ()| "Module1".to_string())
        })
    }

    fn module2(thread: &Thread) -> vm::Result<ExternModule> {
        ExternModule::new(thread, record! {
            module => primitive!(1, |_: ()| "Module2".to_string())
        })
    }

    import::add_extern_module(&thread1, "test_module", module1);
    import::add_extern_module(&thread2, "test_module", module2);

    // Check that test_module returns the same things on different threads
    let mut compiler = Compiler::new();

    let module1 = compiler.run_expr::<String>(&thread1, "test", r#"
        let test_module = import! "test_module"
        let s = test_module.module()
        s
    "#).map_err(|err| err.emit_string(&compiler.code_map())).unwrap();
    assert!(&module1.0 == "Module2");
 
    let module2 = compiler.run_expr::<String>(&thread2, "test", r#"
        let test_module = import! "test_module"
        let s = test_module.module()
        s
    "#).map_err(|err| err.emit_string(&compiler.code_map())).unwrap();
    assert!(&module2.0 == "Module2");
}
