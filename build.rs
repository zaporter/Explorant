fn main() {
    println!("cargo:rerun-if-changed=synoptic/Dockerfile");
    let _res = std::process::Command::new("./synoptic/build.sh")
        .output()
        .expect("failed to build synoptic");
}
