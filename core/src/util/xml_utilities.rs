use std::fs::File;
use std::ops::Add;
use std::path::Path;

use crate::android_string::AndroidString;
use crate::constants;
use crate::error::Error;
use crate::error::ResultExt;
use crate::reader::xml_reader;

type FileWithPath = (File, String);

pub fn read_default_strings(res_dir_path: &Path) -> Result<StringsWithPath, Error> {
    read_strings(open_default_strings_file(res_dir_path)?)
}

pub fn read_foreign_strings(
    res_dir_path: &Path,
    locale_id: &str,
) -> Result<StringsWithPath, Error> {
    read_strings(open_foreign_strings_file(res_dir_path, locale_id)?)
}

fn read_strings(file_with_path: FileWithPath) -> Result<StringsWithPath, Error> {
    let (file, path) = file_with_path;
    xml_reader::read(file)
        .with_context(path.clone())
        .map(|strings| StringsWithPath { path, strings })
}

fn open_default_strings_file(res_dir_path: &Path) -> Result<FileWithPath, Error> {
    open_strings_file(res_dir_path, constants::fs::BASE_VALUES_DIR_NAME)
}

fn open_foreign_strings_file(res_dir_path: &Path, locale_id: &str) -> Result<FileWithPath, Error> {
    let values_dir_name = String::from(constants::fs::BASE_VALUES_DIR_NAME);
    let values_dir_name = values_dir_name.add(&format!("-{}", locale_id));
    open_strings_file(res_dir_path, &values_dir_name)
}

fn open_strings_file(res_dir_path: &Path, values_dir_name: &str) -> Result<FileWithPath, Error> {
    let mut strings_file_path = res_dir_path.to_path_buf();
    strings_file_path.push(values_dir_name);
    strings_file_path.push(constants::fs::STRING_FILE_NAME);

    let path = String::from(strings_file_path.to_string_lossy());
    File::open(strings_file_path)
        .with_context(path.clone())
        .map(|file| (file, path))
}

pub struct StringsWithPath {
    path: String,
    strings: Vec<AndroidString>,
}

impl StringsWithPath {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn strings(&self) -> &[AndroidString] {
        &self.strings
    }

    pub fn into_strings(self) -> Vec<AndroidString> {
        self.strings
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use test_utilities;

    use crate::error;

    #[test]
    fn open_strings_file_errors_if_values_dir_is_missing() {
        let res_dir = tempfile::tempdir().unwrap();
        let error = super::open_strings_file(res_dir.path(), "values");
        match error.unwrap_err().kind {
            error::ErrorKind::Io(_) => {}
            error_kind => panic!("Expected IO error. Received: {:?}", error_kind),
        }
    }

    #[test]
    fn open_strings_file_errors_if_strings_file_is_missing() {
        let res_dir = tempfile::tempdir().unwrap();
        test_utilities::res::setup_values_dir_for_default_locale(res_dir.path());

        let error = super::open_strings_file(res_dir.path(), "values");
        match error.unwrap_err().kind {
            error::ErrorKind::Io(_) => {}
            error_kind => panic!("Expected IO error. Received: {:?}", error_kind),
        }
    }

    #[test]
    fn open_default_strings_file_opens() {
        let res_dir = tempfile::tempdir().unwrap();

        let strings_file_path =
            test_utilities::res::setup_empty_strings_for_default_locale(res_dir.path()).path;
        test_utilities::file::write_content(strings_file_path.clone(), "example content");

        let mut file_contents = String::new();
        let (mut file, file_path) = super::open_default_strings_file(res_dir.path()).unwrap();
        file.read_to_string(&mut file_contents).unwrap();

        assert_eq!(file_contents, "example content");
        assert_eq!(file_path, strings_file_path);
    }

    #[test]
    fn open_foreign_strings_file_opens() {
        let res_dir = tempfile::tempdir().unwrap();

        let strings_file_path =
            test_utilities::res::setup_empty_strings_for_locale(res_dir.path(), "fr").path;
        test_utilities::file::write_content(strings_file_path.clone(), "example content");

        let mut file_contents = String::new();
        let (mut file, file_path) = super::open_foreign_strings_file(res_dir.path(), "fr").unwrap();
        file.read_to_string(&mut file_contents).unwrap();

        assert_eq!(file_contents, "example content");
        assert_eq!(file_path, strings_file_path);
    }
}
