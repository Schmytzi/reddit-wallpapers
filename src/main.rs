extern crate argparse;
extern crate imagefmt;
extern crate reddit_wallpapers;
extern crate regex;
extern crate reqwest;
extern crate user32;

use argparse::{ArgumentParser, Store};
use imagefmt::{ColFmt, ColType};
use reddit_wallpapers::reddit;
use regex::Regex;
use reqwest::{Client, Proxy};
use std::ffi::OsStr;
use std::io::SeekFrom::{Current, End, Start};
use std::io::{Read, Seek, SeekFrom};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use user32::SystemParametersInfoW;

const SPI_SET_DESK_WALLPAPER: u32 = 20;
const SPIF_UPDATE_INI_FILE: u32 = 1;
const SPIF_SEND_WIN_INI_CHANGE: u32 = 2;

// Simple adapter to use a Vec as a read-only buffer. Not ready for use outside of the tool.
struct VecBuffer {
    pos: usize,
    inner: Vec<u8>,
}

impl Read for VecBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let bytes = if self.pos + buf.len() > self.inner.len() {
            self.inner.len() - self.pos
        } else {
            buf.len()
        };
        for i in 0..bytes {
            buf[i] = self.inner[self.pos];
            self.pos += 1;
        }
        Ok(bytes)
    }
}

impl Seek for VecBuffer {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        // If someone knows how to do this w/o all that casting, pray tell
        self.pos = match pos {
            Start(n) => n as usize,
            End(n) => {
                if -n > (self.inner.len() as i64) {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                } else {
                    ((self.inner.len() as i64) + n) as usize
                }
            }
            Current(n) => {
                if -n > (self.pos as i64) {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                } else {
                    ((self.pos as i64) + n) as usize
                }
            }
        };
        Ok(self.pos as u64)
    }
}

fn main() {
    let mut min_width = 1920;
    let mut min_height = 1080;
    let mut subreddit = String::from("earthporn");
    let mut proxy = String::from("");
    // Open block s.t. the argument parser's references do not live too long
    {
        let mut ap = ArgumentParser::new();
        ap.set_description(
            "Downloads the topmost image from a subreddit and sets it as desktop wallpaper",
        );
        ap.refer(&mut min_width).add_option(
            &["-w", "--width"],
            Store,
            "the minimum width of the image in pixels. Default: 1920",
        );
        ap.refer(&mut min_height).add_option(
            &["-H", "--height"],
            Store,
            "the minumum height of the image in pixels. Default: 1080",
        );
        ap.refer(&mut subreddit).add_option(
            &["-s", "--subreddit"],
            Store,
            "The subreddit to get (works with combined subreddits, as well). Default: earthporn",
        );
        ap.refer(&mut proxy)
            .add_option(&["-p", "--proxy"], Store, "Proxy to use for connection");
        ap.parse_args_or_exit();
    }
    let client = if proxy.is_empty(){
        Client::new()
    } else {
        Client::builder()
        .proxy(Proxy::http(&proxy).expect("Proxy url is wrong"))
        .build().expect("Could not reach proxy")
    };
    let subreddit = match reddit::get_subreddit_links(&subreddit, &client) {
        Ok(s) => s,
        Err(message) => panic!("{}", &message),
    };
    for link in subreddit {
        // Search for resolution in title
        let res_pattern = Regex::new(r"\[(\d+) ?x ?(\d+)\]").unwrap();
        for caps in res_pattern.captures_iter(&link.title) {
            // We expect at most one match, but this loop allows for graceful skipping using break
            let width: u16 = caps.get(1).unwrap().as_str().parse().unwrap();
            let height: u16 = caps.get(2).unwrap().as_str().parse().unwrap();
            // Simple unwrapping because the regex guarantees there are numbers in the groups
            if min_width > width || min_height > height {
                break;
            }
            // Only use direct links to images
            if !link.url.ends_with(".jpg") {
                break;
            }
            // Skip broken links
            let mut response = match reqwest::get(&link.url) {
                Ok(res) => res,
                _ => {
                    println!("Could not get image {}", &link.url);
                    break;
                }
            };
            println!("{}", "Writing file");
            // Windows only allows bitmaps as backgrounds. The bitmap will be saved here:
            let bmp = Path::new("image.bmp");
            let mut buffer = VecBuffer {
                pos: 0,
                inner: Vec::new(),
            };
            response
                .copy_to(&mut buffer.inner)
                .expect("Could not write JPG to buffer");
            // Convert JPG to BMP
            let jpg = imagefmt::read_from(&mut buffer, ColFmt::Auto).unwrap();
            imagefmt::write("image.bmp", jpg.w, jpg.h, jpg.fmt, &jpg.buf, ColType::Color)
                .expect("Could not convert image to BMP");
            // Get full path and convert it to UTF-16 so we can pass it to Win32
            // We append a 0 to the UTF-16 representation in case it isn't terminated properly
            let mut full_path: Vec<u16> = OsStr::new(bmp.canonicalize().unwrap().to_str().unwrap())
                .encode_wide()
                .chain(once(0))
                .collect();
            unsafe {
                // Set desktop wallpaper
                SystemParametersInfoW(
                    SPI_SET_DESK_WALLPAPER,
                    0,
                    // Windows expects PVOID for some reason
                    full_path.as_ptr() as *mut std::os::raw::c_void,
                    SPIF_SEND_WIN_INI_CHANGE | SPIF_UPDATE_INI_FILE,
                );
            }
            return;
        }
    }
}
