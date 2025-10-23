#[cfg(test)]
mod test {
    use crate::audio::codec::{extract_title_from_filename, get_data};

    #[test]
    fn test_extract_title_from_filename() {
        assert_eq!(extract_title_from_filename("test_file.mp3"), "test file");
        assert_eq!(extract_title_from_filename("my-song_title.m4a"), "my song title");
        assert_eq!(extract_title_from_filename("simple.mp3"), "simple");
        assert_eq!(extract_title_from_filename("no_ext"), "no ext");
        assert_eq!(extract_title_from_filename("path/to/song.mp3"), "song");
        assert_eq!(extract_title_from_filename("song-with-dashes.mp3"), "song with dashes");

        // Edge cases
        assert_eq!(extract_title_from_filename("a-b_c-d_e.mp3"), "a b c d e");
        assert_eq!(extract_title_from_filename(".hidden"), ".hidden");
        assert_eq!(extract_title_from_filename(".mp3"), ".mp3");
    }

    #[test]
    fn test_get_data_invalid_file() {
        let result = get_data("nonexistent_file.mp3".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_data_invalid_extension() {
        let result = get_data("Cargo.toml".to_string());
        assert!(result.is_err());
    }
}
