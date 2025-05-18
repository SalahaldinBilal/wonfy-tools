#[cfg(feature = "cli")]
mod cli {
    use chrono::{Local, Timelike};
    use clap::Parser;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    use wonfy_tools::tool::stitcher::{CheckDirection, ImageStitcherBuilder, MatchMode, Order};

    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    struct Args {
        #[arg(short, long, required = true, num_args = 1..)]
        files_to_stitch: Vec<String>,
        #[arg(short, long)]
        direction: CheckDirection,
        #[arg(short, long)]
        order: Order,
        #[arg(long)]
        output_dir: Option<String>,
        #[arg(short, long)]
        window_size: Option<usize>,
        #[arg(short, long)]
        match_mode: Option<MatchMode>,
        #[arg(short, long)]
        crop_padding: Option<u32>,
    }

    pub fn tool_main() {
        let args = Args::parse();

        if args.files_to_stitch.len() < 1 {
            eprintln!("need at least two files to stitch.");
            return;
        }

        let images: Result<Vec<_>, _> = args
            .files_to_stitch
            .par_iter()
            .map(|path| image::open(path).map(|img| img.to_rgba8()))
            .collect();

        let images = match images {
            Ok(images) => images,
            Err(err) => {
                eprintln!("Failed to load file: {:#?}", err);
                return;
            }
        };

        let window_size = args.window_size.unwrap_or(6);
        let match_mode = args.match_mode.unwrap_or(MatchMode::Edges);

        let time = Local::now();

        let output_file_path = args.output_dir.unwrap_or_else(|| {
            format!(
                "./stitched-{}-{}-{}_{}.png",
                time.date_naive(),
                time.hour(),
                time.minute(),
                time.second()
            )
        });

        println!("Running with config: ");
        println!("Number of Files: {:#?}", images.len());
        println!("Direction: {:#?}", args.direction);
        println!("Order: {:#?}", args.order);
        println!("Window Size: {:#?}", window_size);
        println!("Match Mode: {:#?}", match_mode);
        println!("Output Path: {}", output_file_path);

        let stitcher = ImageStitcherBuilder::new()
            .images(images)
            .direction(args.direction)
            .order(args.order)
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
