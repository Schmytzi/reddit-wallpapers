# Reddit Wallpaper Getter for Windows
## Description
This is a small utility which queries a subreddit and sets the first image it can find as desktop wallpaper.
It allows for setting a minimum width and height of the selected image (smaller images are skipped).
The default for this is FHD resolution.
Currently, the resolution must be supplied in square brackets in the title.
Subreddits which don't follow this convention will therefore not work.

This tool includes a small library for querying a subreddit and listing its posts.
Currently, it can only parse the link, the author and the title.

## Usage
```
reddit-wallpapers.exe
  -h,--help                show this help message and exit
  -w,--width WIDTH         the minimum width of the image in pixels. Default: 1920
  -H,--height HEIGHT       the minumum height of the image in pixels. Default: 1080
  -s,--subreddit SUBREDDIT The subreddit to get (works with combined subreddits, as well). Default: earthporn
```