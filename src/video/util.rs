const ENDINGS: [&'static str; 11] = [
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "mpg", "mpeg", "m4v", "3gp", "webm",
];

pub fn valid_video_filename_ending<T: Into<String>>(filename: T) -> bool {
    let ending = match get_filename_ending(filename) {
        Some(ending) => ending,
        None => return false,
    };

    ENDINGS.contains(&ending.as_str())
}

pub fn get_filename_ending<T: Into<String>>(filename: T) -> Option<String> {
    let filename = filename.into();
    let split = filename.split('.').collect::<Vec<&str>>();

    if split.len() > 1 {
        Some(split[split.len() - 1].to_string())
    } else {
        None
    }
}
