use std::process::Command;

// this module contain file management functions

/// pre-process the source file, return a path to the pre-processed file
pub fn pre_process_file(file_path: &str) -> String {
    let pre_processed_file = set_file_name(file_path, "pre_process.i");
    Command::new("gcc")
        .args(["-E", "-P", file_path, "-o", &pre_processed_file])
        .output()
        .expect("failed to pre-process the program");
    pre_processed_file
}

/// compile assembly file
pub fn compile_assembly_file(file_path: &str, output_file_path: &str) {
    Command::new("gcc")
        .args([file_path, "-o", output_file_path])
        .output()
        .expect("failed to compile assembly file");
}

/// remove the file extension from a path
pub fn remove_file_extension(file_path: &str) -> &str {
    let dot_index = file_path.rfind('.').expect("invalide source file name");
    &file_path[..dot_index]
}

pub fn delete_file(file_path: &str) {
    Command::new("rm")
        .arg(file_path)
        .output()
        .expect("failed to delete file");
}

/// return a new path with the new file name
pub fn set_file_name(file_path: &str, file_name: &str) -> String {
    let mut path: Vec<&str> = file_path.split('/').collect();
    path.pop();

    let mut new_path = String::new();
    for item in path {
        new_path.push_str(item);
        new_path.push('/');
    }

    new_path.push_str(file_name);
    new_path
}

/// return a reference to the file name
pub fn get_file_name(file_path: &str) -> &str {
    let path: Vec<&str> = file_path.split('/').collect();
    path.last().expect("failed to fetch file name")
}
