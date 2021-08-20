use rand::Rng;
use regex::Regex;
//use std::collections::HashSet;
//use std::fmt;
//use std::fs;
//use std::fs::metadata;
use std::fs::File;
//use std::io::BufRead;
//use std::io::Read;
use std::io::Write;
//use std::io::{Error, ErrorKind};
//use std::path::Path;
//use std::path::PathBuf;
use std::str::FromStr;
//use uuid::Uuid;
use uuid::{Builder, Variant, Version};

struct Entry {
    kind: String,
    file: String,
    filter: Option<String>,
    used: bool,
}

struct Filter {
    path: String,
    unique_id: String,
    used: bool,
}

struct Group {
    entries: Vec<Entry>,
    ext: String,
}

impl Group {
    fn new() -> Group {
        Group {
            entries: vec![],
            ext: String::new(),
        }
    }

    fn from_vec(vec: Vec<Entry>) -> Group {
        let ext = get_extension(vec[0].file.as_str()).unwrap();
        Group {
            entries: vec,
            ext: ext,
        }
    }
}
struct Filters {
    filters: Vec<Filter>,
    item_groups: Vec<Group>,
}

fn read_filters<R: std::io::BufRead>(stream: &mut R) -> Result<Filters, std::io::Error> {
    let re_filter = Regex::new("\\s*<Filter Include=\"(.*)\">").unwrap();
    let re_uid = Regex::new("\\s*<UniqueIdentifier>\\{(.*)\\}</UniqueIdentifier>").unwrap();
    let re_generic_file = Regex::new("\\s*<(.*) Include=\"(.*)\"\\s?/?>").unwrap();
    let re_generic_filter = Regex::new("\\s*<Filter>(.*)</Filter>").unwrap();

    let mut data = Filters {
        filters: vec![],
        item_groups: vec![],
    };
    let mut buf = String::new();
    loop {
        stream.read_line(&mut buf)?;
        if buf.starts_with("<Project") {
            break;
        }
        buf.clear();
    }
    buf.clear();

    // Read path item group
    stream.read_line(&mut buf)?;
    if buf.trim() == "<ItemGroup>" {
        let mut uid = String::new();
        let mut path = String::new();

        // Filters path
        loop {
            buf.clear();
            stream.read_line(&mut buf)?;
            if buf.trim() == "</ItemGroup>" {
                break;
            }
            match re_filter.captures(&buf) {
                Some(caps) => path = caps[1].to_string(),
                None => match re_uid.captures(&buf) {
                    Some(caps) => uid = caps[1].to_string(),
                    None => {
                        if buf.trim() == "</Filter>" {
                            data.filters.push(Filter {
                                path: path.to_string(),
                                unique_id: uid.to_string(),
                                used: false,
                            });
                        }
                    }
                },
            }
        }
        let mut gen_file: Option<String> = None;
        let mut gen_kind: Option<String> = None;
        let mut gen_filter: Option<String> = None;
        loop {
            buf.clear();
            stream.read_line(&mut buf)?;
            if buf.trim() == "</Project>" {
                break;
            }
            if buf.trim() == "<ItemGroup>" {
                data.item_groups.push(Group::new());
                loop {
                    buf.clear();
                    stream.read_line(&mut buf)?;
                    if buf.trim() == "</ItemGroup>" {
                        break;
                    }

                    match re_generic_file.captures(&buf) {
                        Some(caps) => {
                            gen_kind = Some(caps[1].to_string());
                            gen_file = Some(caps[2].to_string());
                            if buf.trim().ends_with("/>") {
                                data.item_groups.last_mut().unwrap().entries.push(Entry {
                                    kind: gen_kind.unwrap(),
                                    file: gen_file.unwrap(),
                                    filter: gen_filter,
                                    used: false,
                                });
                                gen_kind = None;
                                gen_file = None;
                                gen_filter = None;
                            }
                        }
                        None => match re_generic_filter.captures(&buf) {
                            Some(caps) => gen_filter = Some(caps[1].to_string()),
                            None => {
                                if gen_kind.is_some()
                                    && buf.trim() == format!("</{}>", gen_kind.as_ref().unwrap())
                                {
                                    data.item_groups.last_mut().unwrap().entries.push(Entry {
                                        kind: gen_kind.unwrap(),
                                        file: gen_file.unwrap(),
                                        filter: gen_filter,
                                        used: false,
                                    });
                                    gen_kind = None;
                                    gen_file = None;
                                    gen_filter = None;
                                }
                            }
                        },
                    }
                }
            }
        }
    }

    Ok(data)
}

fn read_vcxproj<R: std::io::BufRead>(stream: &mut R) -> Result<Vec<Entry>, std::io::Error> {
    let re_file = Regex::new("\\s*<(.*) Include=\"(.*)\"").unwrap();

    let mut res: Vec<Entry> = vec![];

    let mut buf = String::new();
    loop {
        buf.clear();
        if stream.read_line(&mut buf).unwrap_or(0) == 0 {
            break;
        }
        match re_file.captures(&buf) {
            Some(caps) => {
                let kind = caps[1].to_string();
                let file = caps[2].to_string();
                if kind == "ProjectReference" || kind == "ProjectConfiguration" {
                    continue;
                }
                match file.rfind('\\') {
                    Some(val) => res.push(Entry {
                        kind: kind.clone(),
                        file: file.clone(),
                        filter: Some(String::from_str(&file[0..val]).unwrap()),
                        used: false,
                    }),
                    None => res.push(Entry {
                        kind: kind.clone(),
                        file: file.clone(),
                        filter: Option::None,
                        used: false,
                    }),
                }
            }
            None => {}
        }
    }

    Ok(res)
}

impl Filters {
    fn add_filter_only(&mut self, filter: &str) {
        let mut found = false;
        for f in &mut self.filters {
            if f.path.eq(filter) {
                f.used = true;
                found = true;
                break;
            }
        }
        match filter.rfind('\\') {
            Some(val) => self.add_filter_only(&String::from_str(&filter[0..val]).unwrap()),
            None => {}
        }

        if !found {
            self.filters.push(Filter {
                path: filter.to_string(),
                unique_id: urn_uuid(),
                used: true,
            });
        }
    }

    fn add_filter(&mut self, entry: &Entry) {
        let a = entry.filter.as_ref().unwrap();
        self.add_filter_only(&a);

        let mut found = false;
        for group in &mut self.item_groups {
            for e in &mut group.entries {
                if e.file == entry.file {
                    e.used = true;
                    e.filter = entry.filter.clone();
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            found = false;
            let ext = get_extension(entry.file.as_str()).unwrap();
            for group in &mut self.item_groups {
                if ext == get_extension(group.entries[0].file.as_str()).unwrap() {
                    group.entries.push(Entry {
                        kind: entry.kind.clone(),
                        file: entry.file.clone(),
                        filter: entry.filter.clone(),
                        used: true,
                    });
                    found = true;
                }
            }
            if !found {
                self.item_groups.push(Group::new());
                self.item_groups.last_mut().unwrap().entries.push(Entry {
                    kind: entry.kind.clone(),
                    file: entry.file.clone(),
                    filter: entry.filter.clone(),
                    used: true,
                });
            }
        }
    }
}

fn get_extension(filename: &str) -> Option<String> {
    match filename.rfind('.') {
        Some(val) => Some(String::from_str(&filename[val..]).unwrap()),
        None => None,
    }
}

fn generate_new_filters(
    old_filters: &mut Filters,
    files: &Vec<Entry>,
) -> Result<Filters, std::io::Error> {
    let mut new_filters = Filters {
        filters: vec![],
        item_groups: vec![],
    };

    for entry in files {
        if entry.filter.is_some() {
            old_filters.add_filter(entry);
        }
    }

    for filter in &old_filters.filters {
        if filter.used {
            new_filters.filters.push(Filter {
                path: filter.path.clone(),
                unique_id: filter.unique_id.clone(),
                used: filter.used,
            });
        }
    }

    for group in &old_filters.item_groups {
        let mut v: Vec<Entry> = vec![];
        for e in &group.entries {
            if e.used {
                v.push(Entry {
                    kind: e.kind.clone(),
                    file: e.file.clone(),
                    filter: e.filter.clone(),
                    used: true,
                });
            }
        }
        if !v.is_empty() {
            v.sort_by(|a, b| a.file.cmp(&b.file));
            new_filters.item_groups.push(Group::from_vec(v));
        }
    }

    new_filters.item_groups.sort_by(|a, b| a.ext.cmp(&b.ext));

    new_filters.filters.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(new_filters)
}

fn write_new_filters(filters: &Filters, filename: &str) -> Result<(), std::io::Error> {
    let file = File::create(filename)?;

    let mut buffer = std::io::BufWriter::new(file);
    let bom: [u8; 3] = [0xef, 0xbb, 0xbf];
    buffer.write(&bom)?;
    write!(buffer, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n")?;
    write!(buffer, "<Project ToolsVersion=\"4.0\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\r\n")?;
    write!(buffer, "  <ItemGroup>\r\n")?;
    for filter in &filters.filters {
        if filter.used {
            write!(buffer, "    <Filter Include=\"{}\">\r\n", filter.path)?;
            write!(
                buffer,
                "      <UniqueIdentifier>{{{}}}</UniqueIdentifier>\r\n",
                filter.unique_id
            )?;
            write!(buffer, "    </Filter>\r\n")?;
        }
    }
    write!(buffer, "  </ItemGroup>\r\n")?;

    for group in &filters.item_groups {
        write!(buffer, "  <ItemGroup>\r\n")?;
        for e in &group.entries {
            if e.used {
                match &e.filter {
                    Some(val) => {
                        write!(buffer, "    <{} Include=\"{}\">\r\n", e.kind, e.file)?;
                        write!(buffer, "      <Filter>{}</Filter>\r\n", val)?;
                        write!(buffer, "    </{}>\r\n", e.kind)?;
                    }
                    None => write!(buffer, "    <{} Include=\"{}\" />\r\n", e.kind, e.file)?,
                }
            }
        }
        write!(buffer, "  </ItemGroup>\r\n")?;
    }
    write!(buffer, "</Project>\r\n")?;

    Ok(())
}

fn urn_uuid() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 16]>();
    let uuid = Builder::from_bytes(random_bytes)
        .set_variant(Variant::RFC4122)
        .set_version(Version::Random)
        .build();
    let mut buf = [b'!'; 49];
    uuid.to_urn().encode_lower(&mut buf);

    String::from_utf8(buf[9..41].to_vec()).unwrap()
}

pub fn manage(
    vcxproj_file: &str,
    filters_file: Option<&str>,
    new_filters_file: Option<&str>,
) -> Result<(), std::io::Error> {

    let filters_file = match filters_file {
        Some(val)=> String::from_str(val).unwrap(),
        None => format!("{}.filters", vcxproj_file),
    };

    let output_file = match new_filters_file {
        Some(val)=> String::from_str(val).unwrap(),
        None => filters_file.to_string(),
    };

    let file = File::open(filters_file)?;

    let mut u = read_filters(&mut std::io::BufReader::new(file))?;
    let v = read_vcxproj(&mut std::io::BufReader::new(File::open(vcxproj_file)?))?;

    let res = generate_new_filters(&mut u, &v)?;

    write_new_filters(&res, output_file.as_str())?;
    Ok(())
}
