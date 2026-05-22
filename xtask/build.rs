fn main() {
    let path = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc for xtask");
    println!("cargo:rustc-env=PROTOC_BIN={}", path.display());
}
