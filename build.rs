use std::{path::{Path, PathBuf}, fs::{read_dir, File, create_dir_all}, io::{Write, BufWriter, BufReader}};

use serde_yaml as yaml;

fn render_file<W: Write>(input: &yaml::Mapping, mut output: BufWriter<W>) -> BufWriter<W> {
    for (k, v) in input {
        let key = k.as_str().unwrap();
        match v {
            yaml::Value::String(val) => {
                write!(output, r##"
                    #[allow(dead_code)]
                    #[allow(non_upper_case_globals)]
                    const {}: &str = {:?};
                "##, key, val).unwrap();
            }
            yaml::Value::Mapping(map) => {
                write!(output, r##"
                    #[allow(dead_code)]
                    #[allow(non_snake_case)]
                    mod {} {{
                "##, key).unwrap();
                output = render_file(map, output);
                write!(output, r##"
                    }}
                "##).unwrap();
            }
            _ => panic!("invalid yaml"),
        }
    }
    output
}

fn main() {
    // this code badly needs to be cleaned up...
    let src_dir = Path::new("./src/langs");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let out_dir_s = out_dir.to_str().unwrap();
    let langs_dir = out_dir.join("langs");

    create_dir_all(&langs_dir).unwrap();

    let mod_rs = File::create(langs_dir.join("mod.rs")).unwrap();
    let mut mod_rs = BufWriter::new(mod_rs);
    let mut langs = vec![];

    for entry in read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        match entry.path().extension() {
            Some(e) if e == "yaml" || e == "yml" => (),
            _ => continue,
        }
        let lang_name = entry.path().file_stem().unwrap().to_str().unwrap().to_owned();

        writeln!(mod_rs, r###"
            #[cfg(feature="lang_{lang}")]
            #[path="{out_dir}/langs/{lang}.rs"]
            mod {lang};
            #[cfg(feature="lang_{lang}")]
            pub use {lang}::*;

        "###, lang=lang_name, out_dir=out_dir_s).unwrap();

        langs.push(lang_name.clone());
        let lang_file = File::open(entry.path()).unwrap();
        let lang_file = BufReader::new(lang_file);
        let lang_data: yaml::Value = yaml::from_reader(lang_file).unwrap();

        let lang_rs_path = langs_dir.join(format!("{}.rs", lang_name));
        let lang_rs = File::create(lang_rs_path).unwrap();
        let lang_rs = BufWriter::new(lang_rs);

        render_file(lang_data.as_mapping().unwrap(), lang_rs);
    }

    writeln!(mod_rs, r##"
        #[cfg(any(
    "##).unwrap();
    for i in 0..(langs.len()-1) {
        let word = &langs[i];
        write!(mod_rs, r##"all(feature="lang_{}", any("##, word).unwrap();
        for other in &langs[i..] {
            write!(mod_rs, r##"feature="lang_{}","##, other).unwrap();
        }
        write!(mod_rs, "))").unwrap();
    }
    writeln!(mod_rs, r##"
        ))]
        compile_error!("must enable exactly one language");
    "##).unwrap();
    writeln!(mod_rs, r##"
        #[cfg(not(any(
    "##).unwrap();
    for lang in langs {
        write!(mod_rs, r##"
            feature="lang_{}",
        "##, lang).unwrap();
    }
    writeln!(mod_rs, r##"
        )))]
        compile_error!("must enable exactly one language");
    "##).unwrap()
}
