use embuild::{build::CfgArgs, espidf};

fn main() {
    CfgArgs::output_propagated("ESP_IDF").unwrap();
    espidf::sysenv::output();

    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    println!("cargo:rerun-if-changed=partitions_singleapp.csv");
}

