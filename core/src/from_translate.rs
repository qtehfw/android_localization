use android_string::AndroidString;
use constants;
use helper::xml_read_helper;
use ops::dedup;
use ops::extract;
use ops::filter;
use ops::merge;
use reader::csv_reader;
use reader::xml_reader;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::ops::Add;
use std::path::Path;
use std::path::PathBuf;
use writer::xml_writer;

pub fn do_the_thing<S: ::std::hash::BuildHasher>(
    res_dir_path: &str,
    translated_text_input_dir_path: &str,
    human_friendly_name_to_lang_id_mapping: HashMap<String, String, S>,
) -> Result<(), Error> {
    if human_friendly_name_to_lang_id_mapping.is_empty() {
        return Err(Error::ArgError(String::from(
            "Human friendly name to language ID mapping can't be empty",
        )));
    }

    // Read default strings
    let res_dir_path = Path::new(res_dir_path);
    let mut translatable_default_strings = filter::find_translatable_strings(
        xml_read_helper::read_default_strings(res_dir_path).map_err(Error::from)?,
    );

    // For all languages, handle translations
    for (human_friendly_name, lang_id) in human_friendly_name_to_lang_id_mapping {
        handle_translations(
            res_dir_path,
            &lang_id,
            translated_text_input_dir_path,
            &human_friendly_name,
            &mut translatable_default_strings,
        )?;
    }

    Ok(())
}

fn handle_translations(
    res_dir_path: &Path,
    lang_id: &str,
    translated_text_input_dir_path: &str,
    file_name: &str,
    translatable_default_strings: &mut Vec<AndroidString>,
) -> Result<(), Error> {
    // Read already translated foreign strings
    let mut already_translated_foreign_strings = filter::find_translatable_strings(
        xml_read_helper::read_foreign_strings(res_dir_path, lang_id).map_err(Error::from)?,
    );

    // Read newly translated foreign strings
    let mut translated_text_file_path = PathBuf::from(translated_text_input_dir_path);
    translated_text_file_path.push(file_name);
    translated_text_file_path.set_extension(constants::extn::CSV);
    let mut new_translated_foreign_strings =
        csv_reader::read(File::open(translated_text_file_path).map_err(Error::IoError)?)
            .map_err(Error::CsvError)?;

    // Extract android strings out of the newly translated strings
    let mut new_translated_foreign_strings = extract::extract_android_strings_from_translated(
        &mut new_translated_foreign_strings,
        translatable_default_strings,
    );

    // Merge & dedup foreign strings
    let to_be_written_foreign_strings =
        dedup::dedup_grouped_strings(merge::merge_and_group_strings(
            &mut new_translated_foreign_strings,
            &mut already_translated_foreign_strings,
        ));

    // Write out foreign strings back to file
    let mut file =
        writable_empty_foreign_strings_file(res_dir_path, lang_id).map_err(Error::IoError)?;
    xml_writer::write(&mut file, to_be_written_foreign_strings).map_err(Error::XmlWriteError)
}

fn writable_empty_foreign_strings_file(
    res_dir_path: &Path,
    lang_id: &str,
) -> Result<File, io::Error> {
    let values_dir_name = String::from(constants::fs::BASE_VALUES_DIR_NAME);
    let values_dir_name = values_dir_name.add(&format!("-{}", lang_id));

    let mut strings_file_path = res_dir_path.to_path_buf();
    strings_file_path.push(values_dir_name);
    strings_file_path.push(constants::fs::STRING_FILE_NAME);
    File::create(strings_file_path) // empties out the file if it has any content
}

#[derive(Debug)]
pub enum Error {
    ArgError(String),
    CsvError(csv_reader::Error),
    IoError(io::Error),
    XmlReadError(xml_reader::Error),
    XmlWriteError(xml_writer::Error),
}

impl From<xml_read_helper::Error> for Error {
    fn from(error: xml_read_helper::Error) -> Self {
        match error {
            xml_read_helper::Error::IoError(e) => Error::IoError(e),
            xml_read_helper::Error::XmlError(e) => Error::XmlReadError(e),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::ArgError(_message) => None,
            Error::CsvError(error) => Some(error),
            Error::IoError(error) => Some(error),
            Error::XmlReadError(error) => Some(error),
            Error::XmlWriteError(error) => Some(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ArgError(message) => fmt::Display::fmt(message, f),
            Error::CsvError(error) => fmt::Display::fmt(error, f),
            Error::IoError(error) => fmt::Display::fmt(error, f),
            Error::XmlReadError(error) => fmt::Display::fmt(error, f),
            Error::XmlWriteError(error) => fmt::Display::fmt(error, f),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;

    use android_string::AndroidString;
    use helper::xml_read_helper;
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::io::Write;
    use writer::xml_writer;

    #[test]
    fn do_the_thing_errors_for_empty_human_friendly_name_to_lang_id_mapping() {
        let error = super::do_the_thing("", "", HashMap::new());
        assert_eq!(
            error.unwrap_err().to_string(),
            "Human friendly name to language ID mapping can't be empty"
        )
    }

    #[test]
    fn do_the_thing() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Build paths
        let mut res_dir_path = temp_dir.path().to_path_buf();
        res_dir_path.push("res");
        let mut default_values_dir_path = res_dir_path.clone();
        default_values_dir_path.push("values");
        let mut default_strings_file_path = default_values_dir_path.clone();
        default_strings_file_path.push("strings.xml");

        let mut fr_values_dir_path = res_dir_path.clone();
        fr_values_dir_path.push("values-fr");
        let mut fr_strings_file_path = fr_values_dir_path.clone();
        fr_strings_file_path.push("strings.xml");

        let mut translations_dir_path = temp_dir.path().to_path_buf();
        translations_dir_path.push("translations");
        let mut fr_translations_file_path = translations_dir_path.clone();
        fr_translations_file_path.push("french.csv");

        // Create required dirs & files with content
        fs::create_dir_all(default_values_dir_path.clone()).unwrap();
        fs::create_dir_all(fr_values_dir_path.clone()).unwrap();
        fs::create_dir_all(translations_dir_path.clone()).unwrap();
        let mut default_strings_file = File::create(default_strings_file_path).unwrap();
        let mut fr_strings_file = File::create(fr_strings_file_path).unwrap();
        let mut fr_translations_file = File::create(fr_translations_file_path).unwrap();

        // Write out required contents into files
        xml_writer::write(
            &mut default_strings_file,
            vec![
                AndroidString::new(String::from("s1"), String::from("english value 1"), true),
                AndroidString::new(String::from("s2"), String::from("english value 2"), true),
            ],
        ).unwrap();

        xml_writer::write(
            &mut fr_strings_file,
            vec![
                AndroidString::new(String::from("s1"), String::from("french old value 1"), true),
                AndroidString::new(String::from("s2"), String::from("french old value 2"), true),
            ],
        ).unwrap();

        fr_translations_file
            .write("s1, english value 1, french new value 1".as_bytes())
            .unwrap();

        // Perform action
        let mut map = HashMap::new();
        map.insert(String::from("french"), String::from("fr"));
        super::do_the_thing(
            res_dir_path.clone().to_str().unwrap(),
            translations_dir_path.to_str().unwrap(),
            map,
        ).unwrap();

        // Assert appropriate output
        assert_eq!(
            xml_read_helper::read_foreign_strings(&res_dir_path, "fr").unwrap(),
            vec![
                AndroidString::new(String::from("s1"), String::from("french new value 1"), true),
                AndroidString::new(String::from("s2"), String::from("french old value 2"), true),
            ]
        )
    }

    #[test]
    fn writable_empty_foreign_strings_file() {
        let res_dir = tempfile::tempdir().unwrap();

        let mut values_dir_path = res_dir.path().to_path_buf();
        values_dir_path.push("values-fr");

        let mut strings_file_path = values_dir_path.clone();
        strings_file_path.push("strings.xml");

        fs::create_dir(values_dir_path).unwrap();
        let mut file_with_old_content: File = File::create(strings_file_path.clone()).unwrap();
        file_with_old_content
            .write("example old content".as_bytes())
            .unwrap();

        let mut file_with_new_content: File =
            super::writable_empty_foreign_strings_file(res_dir.path(), "fr").unwrap();
        file_with_new_content
            .write("example new content".as_bytes())
            .unwrap();

        let mut file_contents = String::new();
        File::open(strings_file_path)
            .unwrap()
            .read_to_string(&mut file_contents)
            .unwrap();

        assert_eq!(file_contents, "example new content");
    }
}