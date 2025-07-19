#[cfg(feature = "cli")]
mod cli {
    use std::borrow::Cow;
    use std::fs::read_dir;
    use std::path::PathBuf;

    use chrono::{Local, Timelike};
    use clap::Parser;
    use itertools::Itertools;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    use wonfy_tools::tool::stitcher::{CheckDirection, ImageStitcherBuilder, MatchMode, Order};

    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    struct Args {
        /// List of files to stitch. Can be a directory or multiple files. Has to contain at least two files.
        #[arg(short, long, required = true, num_args = 1..)]
        files_to_stitch: Vec<String>,
        /// Direction of the stitch operation.
        #[arg(short, long, value_enum)]
        direction: CheckDirection,
        /// Are the files ordered or not, defaults to Ordered.
        #[arg(short, long, value_enum)]
        order: Option<Order>,
        /// Output file path. If not provided, a default name will be generated.
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Number of rows to match at once, defaults to 1.
        #[arg(short, long)]
        window_size: Option<usize>,
        /// Match Mode, defaults to Edges.
        #[arg(short, long, value_enum)]
        match_mode: Option<MatchMode>,
        /// Number of pixels to crop the images while matching, defaults to 0.
        #[arg(short, long)]
        crop_padding: Option<u32>,
    }

    pub fn tool_main() {
        let args = Args::parse();

        let files_to_stitch: Result<_, Cow<'_, str>> = match args.files_to_stitch.len() {
            0 => Err(Cow::Borrowed("Need at least two files to stitch.")),
            1 => {
                let path = PathBuf::from(&args.files_to_stitch[0]);

                if path.is_dir() {
                    let contents = read_dir(&path)
                        .map_err(|err| Cow::Owned(format!("Failed to read directory: {}", err)));

                    match contents {
                        Ok(contents) => {
                            let files: Vec<_> = contents
                                .into_iter()
                                .filter_map(|c| match c {
                                    Ok(entry) if entry.path().is_file() => Some(entry.path()),
                                    _ => None,
                                })
                                .collect();

                            if files.len() > 1 {
                                Ok(files)
                            } else {
                                Err(Cow::Borrowed(&format!(
                                    "Directory [{}] must contain at least two files to stitch.",
                                    path.display()
                                )))
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Err(Cow::Borrowed("Need at least two files to stitch."))
                }
            }
            _ => {
                let files: Vec<_> = args
                    .files_to_stitch
                    .iter()
                    .map(PathBuf::from)
                    .filter(|e| e.is_file())
                    .collect();

                if files.len() > 1 {
                    Ok(files)
                } else {
                    Err(Cow::Borrowed("Need at least two files to stitch."))
                }
            }
        };

        let files_to_stitch = match files_to_stitch {
            Ok(files) => files,
            Err(err) => {
                eprintln!("Error: {}", err);
                return;
            }
        };

        let images: Result<Vec<_>, _> = files_to_stitch
            .par_iter()
            .map(|path| {
                image::open(path)
                    .map(|img| img.to_rgba8())
                    .map_err(|err| (path.to_string_lossy(), err))
            })
            .collect();

        let images = match images {
            Ok(images) => images,
            Err((path, err)) => {
                eprintln!("Failed to load image [{}]: {:#?}", path, err);
                return;
            }
        };

        let window_size = args.window_size.unwrap_or(6);
        let match_mode = args.match_mode.unwrap_or(MatchMode::Edges);

        let time = Local::now();

        let output_file_path = args.output_dir.unwrap_or_else(|| {
            PathBuf::from(format!(
                "./stitched-{}-{}-{}_{}.png",
                time.date_naive(),
                time.hour(),
                time.minute(),
                time.second()
            ))
        });

        let order = args.order.unwrap_or(Order::Ordered);

        println!("Running with config: ");
        println!("Number of Files: {:#?}", images.len());
        println!("Direction: {:#?}", args.direction);
        println!("Order: {:#?}", order);
        println!("Window Size: {:#?}", window_size);
        println!("Match Mode: {:#?}", match_mode);
        println!("Output Path: {}", output_file_path.display());
        print!("Stitching Files in the following order: ");

        for (position, file) in files_to_stitch.iter().with_position() {
            use itertools::Position::*;

            let end = match position {
                Last | Only => "\n",
                _ => ", ",
            };

            print!("{}{}", file.file_name().unwrap().to_string_lossy(), end);
        }

        let stitcher = ImageStitcherBuilder::new()
            .images(images)
            .direction(args.direction)
            .order(order)
            .window_size(window_size)
            .match_mode(match_mode)
            .crop(args.crop_padding)
            .build()
            .unwrap();

        let (final_image, stitch_regions) = stitcher.stitch();

        for region in stitch_regions {
            println!("Stitch region: {:#?}", region);
        }

        final_image.save(output_file_path).unwrap();
    }
}

#[cfg(not(feature = "cli"))]
mod no_cli {
    pub fn tool_main() {
        compile_error!("Compile cli binary with cli feature you silly");
    }
}

#[cfg(feature = "cli")]
use cli::tool_main;
#[cfg(not(feature = "cli"))]
use no_cli::tool_main;

fn main() {
    tool_main();
}
