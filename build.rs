use subprocess::Exec;

fn main() {

    Exec::shell("mkdir render\\tmp_build").join().unwrap();
    Exec::shell("cmake -S render -B render\\tmp_build").join().unwrap();
    Exec::shell("cmake --build render\\tmp_build").join().unwrap();
    Exec::shell("mkdir lib_folder").join().unwrap();
    Exec::shell("copy render\\tmp_build\\Debug\\render.lib lib_folder\\render.lib").join().unwrap();

    println!("cargo:rustc-link-search=lib_folder");
    println!("cargo:rustc-link-lib=render");
}