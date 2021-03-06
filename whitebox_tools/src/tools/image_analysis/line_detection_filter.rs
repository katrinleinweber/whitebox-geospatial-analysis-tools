/* 
This tool is part of the WhiteboxTools geospatial analysis library.
Authors: Dr. John Lindsay
Created: July 11, 2017
Last Modified: November 16, 2017
License: MIT
*/
extern crate time;
extern crate num_cpus;

use std::env;
use std::path;
use std::f64;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use raster::*;
use std::io::{Error, ErrorKind};
use tools::*;

pub struct LineDetectionFilter {
    name: String,
    description: String,
    parameters: Vec<ToolParameter>,
    example_usage: String,
}

impl LineDetectionFilter {
    pub fn new() -> LineDetectionFilter {
        // public constructor
        let name = "LineDetectionFilter".to_string();

        let description = "Performs a line-detection filter on an image.".to_string();

        // let mut parameters = "-i, --input   Input raster file.\n".to_owned();
        // parameters.push_str("-o, --output  Output raster file.\n");
        // parameters.push_str("--variant     Optional variant value. Options include 'v' (vertical), 'h' (horizontal), '45', and '135' (default is 'v').\n");
        // parameters.push_str("--absvals     Optional flag indicating whether outputs should be absolute values.\n");
        // parameters.push_str("--clip        Optional amount to clip the distribution tails by, in percent (default is 0.0).\n");

        let mut parameters = vec![];
        parameters.push(ToolParameter{
            name: "Input File".to_owned(), 
            flags: vec!["-i".to_owned(), "--input".to_owned()], 
            description: "Input raster file.".to_owned(),
            parameter_type: ParameterType::ExistingFile(ParameterFileType::Raster),
            default_value: None,
            optional: false
        });

        parameters.push(ToolParameter{
            name: "Output File".to_owned(), 
            flags: vec!["-o".to_owned(), "--output".to_owned()], 
            description: "Output raster file.".to_owned(),
            parameter_type: ParameterType::NewFile(ParameterFileType::Raster),
            default_value: None,
            optional: false
        });

        parameters.push(ToolParameter{
            name: "Variant".to_owned(), 
            flags: vec!["--variant".to_owned()], 
            description: "Optional variant value. Options include 'v' (vertical), 'h' (horizontal), '45', and '135' (default is 'v').".to_owned(),
            parameter_type: ParameterType::OptionList(vec!["vertical".to_owned(), "horizontal".to_owned(), "45".to_owned(), "135".to_owned()]),
            default_value: Some("vertical".to_owned()),
            optional: true
        });

        parameters.push(ToolParameter{
            name: "Output absolute values?".to_owned(), 
            flags: vec!["--absvals".to_owned()], 
            description: "Optional flag indicating whether outputs should be absolute values.".to_owned(),
            parameter_type: ParameterType::Boolean,
            default_value: None,
            optional: true
        });

        parameters.push(ToolParameter{
            name: "Distribution Tail Clip Amount (%)".to_owned(), 
            flags: vec!["--clip".to_owned()], 
            description: "Optional amount to clip the distribution tails by, in percent.".to_owned(),
            parameter_type: ParameterType::Float,
            default_value: Some("0.0".to_owned()),
            optional: true
        });

        let sep: String = path::MAIN_SEPARATOR.to_string();
        let p = format!("{}", env::current_dir().unwrap().display());
        let e = format!("{}", env::current_exe().unwrap().display());
        let mut short_exe = e.replace(&p, "")
            .replace(".exe", "")
            .replace(".", "")
            .replace(&sep, "");
        if e.contains(".exe") {
            short_exe += ".exe";
        }
        let usage = format!(">>.*{} -r={} --wd=\"*path*to*data*\" -i=image.dep -o=output.dep --variant=h --clip=1.0", short_exe, name).replace("*", &sep);

        LineDetectionFilter {
            name: name,
            description: description,
            parameters: parameters,
            example_usage: usage,
        }
    }
}

impl WhiteboxTool for LineDetectionFilter {
    fn get_source_file(&self) -> String {
        String::from(file!())
    }
    
    fn get_tool_name(&self) -> String {
        self.name.clone()
    }

    fn get_tool_description(&self) -> String {
        self.description.clone()
    }

    fn get_tool_parameters(&self) -> String {
        match serde_json::to_string(&self.parameters) {
            Ok(json_str) => return format!("{{\"parameters\":{}}}", json_str),
            Err(err) => return format!("{:?}", err),
        }
    }

    fn get_example_usage(&self) -> String {
        self.example_usage.clone()
    }

    fn run<'a>(&self,
               args: Vec<String>,
               working_directory: &'a str,
               verbose: bool)
               -> Result<(), Error> {
        if args.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidInput,
                                  "Tool run with no paramters."));
        }

        let mut input_file = String::new();
        let mut output_file = String::new();
        let mut variant = "v".to_string();
        let mut absvals = false;
        let mut clip_amount = 0.0;
        for i in 0..args.len() {
            let mut arg = args[i].replace("\"", "");
            arg = arg.replace("\'", "");
            let cmd = arg.split("="); // in case an equals sign was used
            let vec = cmd.collect::<Vec<&str>>();
            let mut keyval = false;
            if vec.len() > 1 {
                keyval = true;
            }
            if vec[0].to_lowercase() == "-i" || vec[0].to_lowercase() == "--input" {
                if keyval {
                    input_file = vec[1].to_string();
                } else {
                    input_file = args[i + 1].to_string();
                }
            } else if vec[0].to_lowercase() == "-o" || vec[0].to_lowercase() == "--output" {
                if keyval {
                    output_file = vec[1].to_string();
                } else {
                    output_file = args[i + 1].to_string();
                }
            } else if vec[0].to_lowercase() == "-variant" || vec[0].to_lowercase() == "--variant" {
                if keyval {
                    variant = vec[1].to_string();
                } else {
                    variant = args[i + 1].to_string();
                }
                if variant.to_lowercase().contains("v") {
                    variant = "v".to_string();
                } else if variant.to_lowercase().contains("h") {
                    variant = "h".to_string();
                } else if variant.to_lowercase().contains("45") {
                    variant = "45".to_string();
                } else if variant.to_lowercase().contains("135") {
                    variant = "135".to_string();
                }
            } else if vec[0].to_lowercase() == "-absvals" || vec[0].to_lowercase() == "--absvals" {
                absvals = true;
            } else if vec[0].to_lowercase() == "-clip" || vec[0].to_lowercase() == "--clip" {
                if keyval {
                    clip_amount = vec[1].to_string().parse::<f64>().unwrap();
                } else {
                    clip_amount = args[i + 1].to_string().parse::<f64>().unwrap();
                }
                if clip_amount < 0.0 {
                    clip_amount == 0.0;
                }
            }
        }

        if verbose {
            println!("***************{}", "*".repeat(self.get_tool_name().len()));
            println!("* Welcome to {} *", self.get_tool_name());
            println!("***************{}", "*".repeat(self.get_tool_name().len()));
        }

        let sep: String = path::MAIN_SEPARATOR.to_string();

        if !input_file.contains(&sep) {
            input_file = format!("{}{}", working_directory, input_file);
        }
        if !output_file.contains(&sep) {
            output_file = format!("{}{}", working_directory, output_file);
        }

        let mut progress: usize;
        let mut old_progress: usize = 1;

        if verbose {
            println!("Reading data...")
        };

        let input = Arc::new(Raster::new(&input_file, "r")?);

        let start = time::now();

        let rows = input.configs.rows as isize;
        let columns = input.configs.columns as isize;
        let nodata = input.configs.nodata;

        let mut output = Raster::initialize_using_file(&output_file, &input);

        let num_procs = num_cpus::get() as isize;
        let (tx, rx) = mpsc::channel();
        for tid in 0..num_procs {
            let input = input.clone();
            let variant = variant.clone();
            let tx1 = tx.clone();
            thread::spawn(move || {
                let mut sum: f64;
                let mut z: f64;
                let mut zn: f64;
                let dx = [-1, 0, 1, -1, 0, 1, -1, 0, 1];
                let dy = [-1, -1, -1, 0, 0, 0, 1, 1, 1];
                let mut weights = [-1.0, 2.0, -1.0, -1.0, 2.0, -1.0, -1.0, 2.0, -1.0]; // 'v'
                if variant == "h" {
                    weights = [-1.0, -1.0, -1.0, 2.0, 2.0, 2.0, -1.0, -1.0, -1.0];
                } else if variant == "135" {
                    weights = [2.0, -1.0, -1.0, -1.0, 2.0, -1.0, -1.0, -1.0, 2.0];
                } else if variant == "45" {
                    weights = [-1.0, -1.0, 2.0, -1.0, 2.0, -1.0, 2.0, -1.0, -1.0];
                }
                let num_pixels_in_filter = dx.len();

                if !absvals {
                    for row in (0..rows).filter(|r| r % num_procs == tid) {
                        let mut data = vec![nodata; columns as usize];
                        for col in 0..columns {
                            z = input[(row, col)];
                            if z != nodata {
                                sum = 0.0;
                                for i in 0..num_pixels_in_filter {
                                    zn = input[(row + dy[i], col + dx[i])];
                                    if zn == nodata {
                                        zn = z; // replace it with z
                                    }
                                    sum += zn * weights[i];
                                }
                                data[col as usize] = sum;
                            }
                        }
                        tx1.send((row, data)).unwrap();
                    }
                } else {
                    for row in (0..rows).filter(|r| r % num_procs == tid) {
                        let mut data = vec![nodata; columns as usize];
                        for col in 0..columns {
                            z = input[(row, col)];
                            if z != nodata {
                                sum = 0.0;
                                for i in 0..num_pixels_in_filter {
                                    zn = input[(row + dy[i], col + dx[i])];
                                    if zn == nodata {
                                        zn = z; // replace it with z
                                    }
                                    sum += zn * weights[i];
                                }
                                data[col as usize] = sum.abs();
                            }
                        }
                        tx1.send((row, data)).unwrap();
                    }
                }
            });
        }

        for row in 0..rows {
            let data = rx.recv().unwrap();
            output.set_row_data(data.0, data.1);
            if verbose {
                progress = (100.0_f64 * row as f64 / (rows - 1) as f64) as usize;
                if progress != old_progress {
                    println!("Progress: {}%", progress);
                    old_progress = progress;
                }
            }
        }

        if clip_amount > 0.0 {
            println!("Clipping output...");
            output.clip_min_and_max_by_percent(clip_amount);
        }

        let end = time::now();
        let elapsed_time = end - start;
        output.configs.palette = "grey.plt".to_string();
        output.add_metadata_entry(format!("Created by whitebox_tools\' {} tool",
                                          self.get_tool_name()));
        output.add_metadata_entry(format!("Input file: {}", input_file));
        output.add_metadata_entry(format!("Variant: {}", variant));
        output.add_metadata_entry(format!("Absolute values: {}", absvals));
        output.add_metadata_entry(format!("Clip amount: {}", clip_amount));
        output.add_metadata_entry(format!("Elapsed Time (excluding I/O): {}", elapsed_time)
                                      .replace("PT", ""));

        if verbose {
            println!("Saving data...")
        };
        let _ = match output.write() {
            Ok(_) => {
                if verbose {
                    println!("Output file written")
                }
            }
            Err(e) => return Err(e),
        };

        if verbose {
            println!("{}",
                     &format!("Elapsed Time (excluding I/O): {}", elapsed_time).replace("PT", ""));
        }

        Ok(())
    }
}