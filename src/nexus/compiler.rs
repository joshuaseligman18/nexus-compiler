use crate::util::nexus_log;

pub fn compile(source_code: String) {
    nexus_log::clear_logs();
    nexus_log::log(String::from("COMPILER"), String::from("Compile called"));
    nexus_log::log(String::from("COMPILER"), source_code);
}