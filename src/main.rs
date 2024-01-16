use colored::*;
use serde::Deserialize;
use std::fs;
use std::io::{self, stdin, Write};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Deserialize)]
struct Config {
    python_files: Vec<PythonFileConfig>,
}

#[derive(Deserialize)]
struct PythonFileConfig {
    file_name: String,
    test_dir: String,
}

fn main() -> io::Result<()> {
    let config_file_path = "config.json";
    let config_contents = fs::read_to_string(config_file_path)?;
    let config: Config = serde_json::from_str(&config_contents)?;

    // Prompt the user to enter the name of the Python file to test
    print!("Enter the name of the Python file to test: ");
    io::stdout().flush()?; // Flush stdout to ensure the print! message appears before the input prompt
    let mut python_file_name = String::new();
    stdin().read_line(&mut python_file_name)?;
    python_file_name = python_file_name.trim().to_string(); // Trim newline characters from the input

    // Find the Python file in the configuration
    let python_file_config = config
        .python_files
        .into_iter()
        .find(|config| config.file_name == python_file_name);

    match python_file_config {
        Some(python_file_config) => {
            let test_dir_path = Path::new(&python_file_config.test_dir);
            let python_file_path = Path::new(&python_file_config.file_name);

            // Filter the stdin_files to only include those that contain the name of the Python file in their filename
            let python_file_stem = python_file_name.replace(".py", "");
            let stdin_files = fs::read_dir(test_dir_path)?
                .filter_map(Result::ok)
                .filter(|entry| {
                    let file_name = entry.file_name().into_string().unwrap_or_default();
                    file_name.ends_with(".stdin") && file_name.contains(&python_file_stem)
                });

            println!("{}", python_file_path.display().to_string().cyan().bold());

            let mut passed = 0;
            let mut failed = 0;

            for stdin_file in stdin_files {
                let stdin_path = stdin_file.path();
                let output_file_path = stdin_path.with_extension("out");

                let expected_output = fs::read_to_string(&output_file_path)?;

                let start = Instant::now();
                let output = Command::new("python")
                    .arg(python_file_path)
                    .stdin(fs::File::open(&stdin_path)?)
                    .output()?;
                let duration = start.elapsed();

                let output_str = String::from_utf8_lossy(&output.stdout);

                if output_str.trim() == expected_output.trim() {
                    println!(
                        "{}: {} (took {:.2?})",
                        stdin_path.display(),
                        "PASSED".green(),
                        duration
                    );
                    passed += 1;
                } else {
                    println!(
                        "{}: {} (took {:.2?})",
                        stdin_path.display(),
                        "FAILED".red(),
                        duration
                    );
                    failed += 1;
                }
            }

            println!(
                "\n[SUMMARY] Successfully ran {} tests on {}:",
                passed + failed,
                python_file_path.display().to_string().cyan().bold()
            );
            println!(
                "{} PASSED ({:.2}%)",
                passed.to_string().green(),
                if passed + failed > 0 {
                    (passed as f64 / (passed + failed) as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!(
                "{} FAILED ({:.2}%)\n",
                failed.to_string().red(),
                if passed + failed > 0 {
                    (failed as f64 / (passed + failed) as f64) * 100.0
                } else {
                    0.0
                }
            );
        }
        None => println!("Python file not found in the configuration."),
    }

    Ok(())
}
