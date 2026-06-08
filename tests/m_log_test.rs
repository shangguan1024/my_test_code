use hello_world::{ModuleLog, m_log};
use m_log::{define_module, m_info, m_warn, m_error, m_debug};

define_module!(TestMod, info=true, warn=true, error=true, debug=false);

define_module!(OffMod, info=false, warn=false, error=false, debug=false);

#[test]
fn test_define_module_creates_struct() {
    let _ = TestMod;
}

#[test]
fn test_module_log_trait_impl() {
    assert!(TestMod::INFO);
    assert!(TestMod::WARN);
    assert!(TestMod::ERROR);
    assert!(!TestMod::DEBUG);
}

#[test]
fn test_convenience_macros_compile() {
    m_log::init();
    testmod_info!("test info message");
    testmod_warn!("test warn message");
    testmod_error!("test error message");
    testmod_debug!("test debug message");
}

#[test]
fn test_m_info_macro_enabled() {
    m_log::init();
    m_info!(TestMod, "test log output");
}

#[test]
fn test_req005_off_switch_no_output() {
    m_log::init();
    offmod_info!("should not appear");
    offmod_warn!("should not appear");
    offmod_error!("should not appear");
    offmod_debug!("should not appear");
    assert!(!OffMod::INFO);
    assert!(!OffMod::WARN);
    assert!(!OffMod::ERROR);
    assert!(!OffMod::DEBUG);
}

#[test]
fn test_req006_on_switch_outputs_log() {
    m_log::init();
    assert!(TestMod::INFO);
    assert!(TestMod::ERROR);
    m_info!(TestMod, "REQ-006: ON switch should output");
    m_error!(TestMod, "REQ-006: ON error should output");
}

#[test]
fn test_req008_init_sets_logger() {
    m_log::init();
    m_info!(TestMod, "REQ-008: logger initialized");
}

#[test]
fn test_req009_no_business_hardcoding() {
    let m_log_source = include_str!("../m_log/src/lib.rs");
    assert!(!m_log_source.contains("Pipeline"), "REQ-009: m_log.rs must not hardcode Pipeline");
    assert!(!m_log_source.contains("DataModule"), "REQ-009: m_log.rs must not hardcode Data");
    assert!(!m_log_source.contains("MainModule"), "REQ-009: m_log.rs must not hardcode Main");
}

#[test]
fn test_req007_no_println_in_business_code() {
    let main_source = include_str!("../src/main.rs");
    let count = main_source.matches("println!").count();
    assert_eq!(count, 0, "REQ-007: main.rs should have no println! calls");
}