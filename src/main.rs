// #![allow(unused)]

use core::panic;
use std::{path::Path, process};

use image::{DynamicImage, GenericImage, GenericImageView};
use walkdir::WalkDir;

extern crate clap;
use clap::{App, Arg, ArgMatches};

const MAX_COLS: &str = "MAX_COLS";
const OUTPUT: &str = "OUTPUT";
const INPUT_PATH: &str = "INPUT_PATH";
const FORCE_OUTPUT_OVERRIDE: &str = "FORCE_OUTPUT_OVERRIDE";
const IGNORE_IMAGES: &str = "IGNORE";

fn main() {

  let matches = App::new(env!("CARGO_PKG_NAME"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .version(env!("CARGO_PKG_VERSION"))
    .about("Bundles all images in a folder into a single image. The size of the largest image will be used for tile size")
      .arg(Arg::with_name(INPUT_PATH)
        .required(true)
        .index(1)
        .help("Folder that contains images will be bundled")
      )
      .arg(Arg::with_name(MAX_COLS)
        .value_name("NUMBER")
        .short("m")
        .long(MAX_COLS)
        .help("Maximum number columns of tiles")
      )
      .arg(Arg::with_name(OUTPUT)
        .value_name("PATH")
        .short("o")
        .long(OUTPUT)
        .help("Output file name")
      )
      .arg(Arg::with_name(IGNORE_IMAGES)
        .value_name("PATH PATH2")
        .short("i")
        .long(IGNORE_IMAGES)
        .multiple(true)
        .help("Images to be ignored")
      )
      .arg(Arg::with_name(FORCE_OUTPUT_OVERRIDE)
        .alias("f")
        .short("f")
        .help("Forces output override")
      )
    .get_matches();

  let max_cols = get_max_cols(&matches);
  let output = get_output(&matches);
  let input = matches.value_of(INPUT_PATH).unwrap();

  if !Path::new(input).exists() {
    println!("Path <{}> doesn't exists", input);
    process::exit(1);
  }
  
  let mut col = 0;
  let mut line = 0;  
  let mut files: Vec<String> = vec![];
  let mut tile_w = 0u32;
  let mut tile_h = 0u32;

  let mut ignored_images: Vec<String> = Vec::new();

  if let Some(ignored_images_res) = matches.values_of(IGNORE_IMAGES) {
    ignored_images = ignored_images_res.into_iter().map(|a| a.to_string()).collect();
  }
  
  for entry in WalkDir::new(input) {
    match entry {
      Ok(entry) => {
        if entry.file_type().is_dir() { continue; }
        let filename = entry.file_name().to_os_string().into_string().unwrap();
        if !filename.ends_with(".png") && !filename.ends_with(".bmp") && !filename.ends_with(".jpeg") {
          continue;
        }
        let file = entry.path().to_str().unwrap().to_string();
        let img = image::open(&file).unwrap();
        let img_w = img.width();
        let img_h = img.height();

        if img_w > tile_w {
          tile_w = img_w;
        }
        if img_h > tile_h {
          tile_h = img_h;
        }
        
        if !ignored_images.contains(&file) {
          files.push(file);
        }
      },
      Err(e) => panic!("Error found {}", e),
    }
  }

  let max_lines =  {
    if (files.len() as u32) < max_cols {
      1
    } else {
      (files.len() as f32 / max_cols as f32).ceil() as u32
    }
  };

  let mut big_img = DynamicImage::new_rgba8(tile_w * max_cols, tile_h * max_lines);

  for file in files.iter() {
    let img = image::open(file).unwrap();
    if col != 0 && col % (max_cols) == 0 {
      col = 0;
      line += 1;
    }
    let mut subimg = image::imageops::crop(&mut big_img, tile_w * col, line * tile_h, tile_w, tile_h);
    subimg.copy_from(&img, 0, 0).unwrap();
    col += 1;
  }
  big_img.save(output).unwrap();
}

fn get_output(matches: &ArgMatches) -> String {
  let output = matches.value_of(OUTPUT).unwrap_or("spritesheet_out.png").to_string();
  let forced = matches.occurrences_of(FORCE_OUTPUT_OVERRIDE) > 0;

  if Path::new(&output).exists() && !forced {
    println!("Output <{}> alread exists. Use -f flag to force output override", output);
    process::exit(1);
  }
  output
}

fn get_max_cols(matches: &ArgMatches) -> u32 {
  let max_col_str = matches.value_of(MAX_COLS).unwrap_or("10");  
  match max_col_str.parse::<u32>() {
    Ok(v) => {
      if v == 0 {
        v + 1
      } else {
        v
      }
    },
    Err(_) => {
      println!("MAX_COL must be a positive INTEGER");
      process::exit(1);
    },
  }
}