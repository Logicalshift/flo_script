#[macro_use] extern crate gluon_vm;

use gluon::*;
use gluon::compiler_pipeline::*;
use gluon::vm::ExternModule;
use gluon::import;
use gluon::vm::api::*;

use futures::*;
use futures::executor;

#[test]
fn threads_have_same_modules() {
    // In Gluon: importing a module replaces an existing module and also applies across all threads created for the same VM
    // Once bound by the compiler, modules stay bound to the same place
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

    // This redefines module1 to module2
    import::add_extern_module(&thread1, "test_module", module1);
    import::add_extern_module(&thread2, "test_module", module2);

    // Check that test_module returns the same things on different threads
    let mut compiler = Compiler::new();

    let module1 = compiler.run_expr::<String>(&thread1, "test", r#"
        let test_module = import! test_module
        let s = test_module.module()
        s
    "#).map_err(|err| err.emit_string(&compiler.code_map())).unwrap();
    assert!(&module1.0 == "Module2");
 
    let module2 = compiler.run_expr::<String>(&thread2, "test", r#"
        let test_module = import! test_module
        let s = test_module.module()
        s
    "#).map_err(|err| err.emit_string(&compiler.code_map())).unwrap();
    assert!(&module2.0 == "Module2");
}

#[test]
fn modules_stay_bound() {
    // Once a symbol is bound, it stays bound (we can't replace it as we did in the previous example)
    let root_thread = new_vm();

    // Create two threads
    let thread1     = root_thread.new_thread().unwrap();
    let thread2     = root_thread.new_thread().unwrap();

    // Import two modules into them (same name in both threads, but different modules)
    fn module1(thread: &Thread) -> vm::Result<ExternModule> {
        ExternModule::new(thread, record! {
            module => "Module1".to_string()
        })
    }

    fn module2(thread: &Thread) -> vm::Result<ExternModule> {
        ExternModule::new(thread, record! {
            module => "Module2".to_string()
        })
    }

    // Compile two functions with different modules imported
    let mut compiler1 = Compiler::new();

    import::add_extern_module(&thread1, "test_module", module1);
    let module1     = r#"
            let test_module = import! test_module
            test_module.module
        "#;
    let module1     = module1.compile(&mut compiler1, &thread1, "testfile", module1, Some(&String::make_type(&thread1)))
        .map_err(|err| err.emit_string(&compiler1.code_map())).unwrap();

    let mut compiler2 = Compiler::new();
    import::add_extern_module(&thread2, "test_module", module2);
    let module2     = r#"
            let test_module = import! test_module
            test_module.module
        "#;
    let module2     = module2.compile(&mut compiler2, &thread2, "testfile", module2, Some(&String::make_type(&thread2)))
        .map_err(|err| err.emit_string(&compiler2.code_map())).unwrap();

    // Evaluate the compiled module1 expression
    let mod1        = module1.run_expr(&mut compiler1, thread1.clone(), "module1", "", ())
        .and_then(move |execute_value| {
                Ok((
                    String::from_value(&thread1, execute_value.value.get_variant()),
                    execute_value.typ,
                ))
            });
    let mut mod1    = executor::spawn(mod1);
    let mod1        = mod1.wait_future().unwrap();
    assert!(mod1.0 == "Module1".to_string());

    // Evaluate the compiled module2 expression
    let mod2        = module2.run_expr(&mut compiler2, thread2.clone(), "module2", "", ())
        .and_then(move |execute_value| {
                Ok((
                    String::from_value(&thread2, execute_value.value.get_variant()),
                    execute_value.typ,
                ))
            });
    let mut mod2    = executor::spawn(mod2);
    let mod2        = mod2.wait_future().unwrap();
    println!("{:?}", mod2);
    assert!(mod2.0 == "Module1".to_string());
}
