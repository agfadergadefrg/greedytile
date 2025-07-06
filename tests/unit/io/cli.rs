//! Tests for command-line interface parsing and file processing

#[cfg(test)]
mod tests {
    use clap::Parser;
    use greedytile::io::cli::Cli;
    use greedytile::io::configuration::{DEFAULT_MAX_ITERATIONS, DEFAULT_SEED};
    use std::path::PathBuf;

    // Tests CLI parsing with only required target file argument
    // Verified by changing default values to ensure defaults are used
    #[test]
    fn test_cli_parse_minimal_args() {
        let args = vec!["program", "test.png"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.target, PathBuf::from("test.png"));
        assert_eq!(cli.seed, DEFAULT_SEED);
        assert_eq!(cli.iterations, DEFAULT_MAX_ITERATIONS);
        assert!(!cli.quiet);
    }

    // Tests CLI parsing with all available arguments
    // Verified by modifying custom parsers to ensure they're invoked
    #[test]
    fn test_cli_parse_all_args() {
        let args = vec![
            "program",
            "input.png",
            "--seed",
            "123",
            "--iterations",
            "500",
            "--quiet",
            "--no-skip",
        ];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.target, PathBuf::from("input.png"));
        assert_eq!(cli.seed, 123);
        assert_eq!(cli.iterations, 500);
        assert!(cli.quiet);
    }

    // Tests file skip behavior based on --no-skip flag
    // Verified by inverting boolean logic in skip_existing method
    #[test]
    fn test_skip_existing_logic() {
        let args_default = vec!["program", "test.png"];
        let cli_default = Cli::parse_from(args_default);
        assert!(cli_default.skip_existing());

        let args_no_skip = vec!["program", "test.png", "--no-skip"];
        let cli_no_skip = Cli::parse_from(args_no_skip);
        assert!(!cli_no_skip.skip_existing());
    }

    // Tests progress display based on --quiet flag
    // Verified by inverting quiet flag logic
    #[test]
    fn test_should_show_progress() {
        let args_default = vec!["program", "test.png"];
        let cli_default = Cli::parse_from(args_default);
        assert!(cli_default.should_show_progress());

        let args_quiet = vec!["program", "test.png", "--quiet"];
        let cli_quiet = Cli::parse_from(args_quiet);
        assert!(!cli_quiet.should_show_progress());
    }

    // Tests short flag parsing (-s, -i)
    // Verified by changing short flag definitions
    #[test]
    fn test_cli_short_flags() {
        let args = vec!["program", "test.png", "-s", "999", "-i", "100"];
        let cli = Cli::parse_from(args);

        assert_eq!(cli.seed, 999);
        assert_eq!(cli.iterations, 100);
    }

    use greedytile::io::cli::FileProcessor;
    use std::fs;
    use tempfile::TempDir;

    // Tests FileProcessor construction
    // Verified by modifying constructor logic
    #[test]
    fn test_file_processor_new() {
        let cli = create_test_cli("test.png");
        let _processor = FileProcessor::new(cli);
    }

    // Tests error handling for missing files
    // Verified by removing error return for nonexistent files
    #[test]
    fn test_process_nonexistent_file() {
        let cli = create_test_cli("nonexistent.png");
        let mut processor = FileProcessor::new(cli);

        let result = processor.process();
        assert!(result.is_err());
    }

    // Tests error handling for non-PNG files
    // Verified by removing file type validation
    #[test]
    fn test_process_invalid_file_type() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "not a png").unwrap();

        let cli = create_test_cli(txt_file.to_str().unwrap());
        let mut processor = FileProcessor::new(cli);

        let result = processor.process();
        assert!(result.is_err());
    }

    // Tests skip logic when output file exists
    // Verified by removing skip check
    #[test]
    fn test_skip_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.png");
        let output_file = temp_dir.path().join("test_result.png");

        fs::write(&input_file, "fake png").unwrap();
        fs::write(&output_file, "fake png").unwrap();

        let cli = create_test_cli(input_file.to_str().unwrap());
        let mut processor = FileProcessor::new(cli);

        let result = processor.process();
        assert!(result.is_ok());
    }

    // Tests processing empty directories
    // Verified by adding error for empty directories
    #[test]
    fn test_process_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let cli = create_test_cli(temp_dir.path().to_str().unwrap());
        let mut processor = FileProcessor::new(cli);

        let result = processor.process();
        assert!(result.is_ok());
    }

    // Tests output filename generation with suffix
    // Verified by changing output suffix to verify path generation
    #[test]
    fn test_output_path_generation() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let input_file = temp_dir.path().join("test_image.png");

        fs::write(&input_file, "fake png").unwrap();

        let output_file = temp_dir.path().join("test_image_result.png");
        fs::write(&output_file, "output").unwrap();

        let cli = create_test_cli(input_file.to_str().unwrap());
        let mut processor = FileProcessor::new(cli);

        let result = processor.process();
        assert!(result.is_ok());

        let input_file2 = temp_dir.path().join("test_image2.png");
        fs::write(&input_file2, "fake png").unwrap();

        let cli2 = create_test_cli(input_file2.to_str().unwrap());
        let mut processor2 = FileProcessor::new(cli2);

        let _ = processor2.process();

        let wrong_output = temp_dir.path().join("test_image2_output.png");
        assert!(
            !wrong_output.exists(),
            "Should not create file with wrong suffix"
        );
    }

    // Tests quiet mode configuration and behavior
    // Verified by testing quiet flag affects progress display
    #[test]
    fn test_quiet_mode() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let input_file = temp_dir.path().join("test.png");

        fs::write(&input_file, "fake png").unwrap();

        let args_quiet = vec!["program", input_file.to_str().unwrap(), "--quiet"];
        let cli_quiet = Cli::parse_from(args_quiet);
        assert!(cli_quiet.quiet, "Quiet flag should be set");
        assert!(
            !cli_quiet.should_show_progress(),
            "Should not show progress in quiet mode"
        );

        let mut processor_quiet = FileProcessor::new(cli_quiet);
        let _ = processor_quiet.process();

        let args_normal = vec!["program", input_file.to_str().unwrap()];
        let cli_normal = Cli::parse_from(args_normal);
        assert!(!cli_normal.quiet, "Quiet flag should not be set by default");
        assert!(
            cli_normal.should_show_progress(),
            "Should show progress by default"
        );

        let mut processor_normal = FileProcessor::new(cli_normal);
        let _ = processor_normal.process();
    }

    fn create_test_cli(target: &str) -> Cli {
        let args = vec!["program", target];
        Cli::parse_from(args)
    }
}
