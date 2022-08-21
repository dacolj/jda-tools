use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str;
use std::str::FromStr;
use uuid::{Builder, Variant, Version};
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};
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

fn get_include_value(attributes: &Vec<OwnedAttribute>) -> Result<String, std::io::Error> {
    match attributes.iter().find(|&r| r.name.local_name == "Include") {
        Some(val) => Ok(val.value.to_string().clone()),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No include in filter element",
        )),
    }
}

fn read_filters<R: std::io::BufRead>(stream: &mut R) -> Result<Filters, std::io::Error> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;

    // Remove the BOM, waiting for better implementation
    if buf.starts_with(&[0xef, 0xbb, 0xbf]) {
        buf.drain(0..3);
    }

    let tmp_data = String::from_utf8(buf).unwrap_or_default();
    let mut parser = EventReader::from_str(tmp_data.as_str());

    let mut data = Filters {
        filters: vec![],
        item_groups: vec![],
    };

    #[derive(PartialEq)]
    enum State {
        Unknown,
        NewGroup,
        FilterGroup,
        FilterItem,
        FilterUID,
        ItemGroup,
        ItemGroupItem(String),
        ItemGroupItemFilter(String),
    }

    let mut state = State::Unknown;
    loop {
        let e = parser.next();
        match e {
            Ok(XmlEvent::StartElement {
                ref name,
                ref attributes,
                ..
            }) => {
                if state == State::NewGroup {
                    if name.local_name == "Filter" {
                        state = State::FilterGroup;
                    } else {
                        state = State::ItemGroup;
                        data.item_groups.push(Group::new());
                    }
                }
                match state {
                    State::Unknown => {
                        if name.local_name == "ItemGroup" {
                            state = State::NewGroup;
                        }
                    }
                    State::FilterGroup => {
                        state = State::FilterItem;
                        data.filters.push(Filter::new());
                        data.filters.last_mut().unwrap().path = get_include_value(&attributes)?;
                    }
                    State::NewGroup => { // managed before the match
                    }
                    State::FilterItem => {
                        if name.local_name == "UniqueIdentifier" {
                            state = State::FilterUID;
                        }
                    }
                    State::FilterUID => {}
                    State::ItemGroup => {
                        state = State::ItemGroupItem(name.local_name.to_string().clone());
                        data.item_groups
                            .last_mut()
                            .unwrap()
                            .entries
                            .push(Entry::new(
                                name.local_name.as_str(),
                                get_include_value(&attributes)?.as_str(),
                            ));
                    }
                    State::ItemGroupItem(ref item) => {
                        if name.local_name == "Filter" {
                            state = State::ItemGroupItemFilter(item.to_string().clone());
                        }
                    }
                    State::ItemGroupItemFilter(..) => {}
                };
            }
            Ok(XmlEvent::Characters(ref chars)) => match state {
                State::FilterUID => {
                    data.filters.last_mut().unwrap().unique_id = chars.to_string().clone()
                }
                State::ItemGroupItemFilter(..) => {
                    data.item_groups
                        .last_mut()
                        .unwrap()
                        .entries
                        .last_mut()
                        .unwrap()
                        .filter = Some(chars.to_string().clone())
                }
                _ => {}
            },
            Ok(XmlEvent::EndElement { ref name }) => {
                match state {
                    State::FilterGroup => {
                        if name.local_name == "ItemGroup" {
                            state = State::Unknown;
                        }
                    }
                    State::NewGroup => {
                        state = State::Unknown;
                    }
                    State::FilterItem => {
                        if name.local_name == "Filter" {
                            state = State::FilterGroup;
                        }
                    }
                    State::FilterUID => {
                        if name.local_name == "UniqueIdentifier" {
                            state = State::FilterItem;
                        }
                    }
                    State::ItemGroup => {
                        if name.local_name == "ItemGroup" {
                            state = State::Unknown;
                        }
                    }
                    State::ItemGroupItem(ref item) => {
                        if name.local_name == item.as_str() {
                            state = State::ItemGroup;
                        }
                    }
                    State::ItemGroupItemFilter(ref item) => {
                        state = State::ItemGroupItem(item.to_string().clone());
                    }
                    State::Unknown => {}
                };
            }
            Ok(XmlEvent::EndDocument) => break,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.clone(),
                ))
            }
            _ => {}
        }
    }

    Ok(data)
}

fn file_filter(file: &str) -> Option<String> {
    match file.rfind('\\') {
        Some(val) => Some(String::from_str(&file[0..val]).unwrap()),
        None => Option::None,
    }
}

fn read_vcxproj<R: std::io::BufRead>(stream: &mut R) -> Result<Vec<Entry>, std::io::Error> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;

    // Remove the BOM, waiting better implementation
    if buf.starts_with(&[0xef, 0xbb, 0xbf]) {
        buf.drain(0..3);
    }

    let a = String::from_utf8(buf).unwrap_or_default();
    let parser = EventReader::from_str(a.as_str());

    let mut res: Vec<Entry> = vec![];

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                for a in &attributes {
                    if a.name.local_name == "Include" {
                        if name.local_name != "ProjectReference"
                            && name.local_name != "ProjectConfiguration"
                        {
                            res.push(Entry::new(name.local_name.as_str(), a.value.as_str()));
                        }
                    }
                }
            }
            Err(e) => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.clone());
                break;
            }
            _ => {}
        }
    }

    Ok(res)
}

impl Entry {
    fn new(kind: &str, filename: &str) -> Entry {
        Entry {
            kind: String::from_str(kind).unwrap(),
            file: String::from_str(filename).unwrap(),
            filter: file_filter(filename),
            used: true,
        }
    }
}

impl Filter {
    fn new() -> Filter {
        Filter {
            path: String::new(),
            unique_id: String::new(),
            used: false,
        }
    }
}

impl Filters {
    fn new() -> Filters {
        Filters {
            filters: vec![],
            item_groups: vec![],
        }
    }

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
        if entry.filter.as_ref().is_some(){
            let a = entry.filter.as_ref().unwrap();
            self.add_filter_only(&a);
        }

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
    sort: bool,
) -> Result<Filters, std::io::Error> {
    let mut new_filters = Filters {
        filters: vec![],
        item_groups: vec![],
    };

    for entry in files {
        old_filters.add_filter(entry);
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
            if sort {
                v.sort_by(|a, b| a.file.cmp(&b.file));
            }
            new_filters.item_groups.push(Group::from_vec(v));
        }
    }

    if sort {
        new_filters.item_groups.sort_by(|a, b| a.ext.cmp(&b.ext));

        new_filters.filters.sort_by(|a, b| a.path.cmp(&b.path));
    }

    Ok(new_filters)
}

fn write_filters(filters: &Filters, filename: &str) -> Result<(), std::io::Error> {
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
                "      <UniqueIdentifier>{}</UniqueIdentifier>\r\n",
                filter.unique_id
            )?;
            write!(buffer, "    </Filter>\r\n")?;
        }
    }
    write!(buffer, "  </ItemGroup>\r\n")?;

    for group in &filters.item_groups {
        if group.entries.is_empty(){
            continue;
        }
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

    format!("{{{}}}", str::from_utf8(&buf[9..41]).unwrap())
}

pub fn apply(
    vcxproj_file: &str,
    filters_file: Option<String>,
    new_filters_file: Option<String>,
    sort: bool,
    verbose: bool,
) -> Result<(), std::io::Error> {
    let output_file = match new_filters_file {
        Some(val) => val,
        None => format!("{}.filters", vcxproj_file),
    };

    let mut input_filters = match filters_file {
        Some(filename) => match Path::new(filename.as_str()).is_file() {
            true => {
                if verbose {
                    println!("Reading filters file \"{}\"...", filename);
                }
                read_filters(&mut std::io::BufReader::new(File::open(filename)?))?
            }
            false => {
                println!(
                    "Filters file \"{}\" doesn't exist, will starting from empty filters",
                    filename
                );
                Filters::new()
            }
        },
        None => {
            if verbose {
                println!("Starting from empty filters");
            }
            Filters::new()
        }
    };

    if verbose {
        println!("Reading vcproj file \"{}\"...", vcxproj_file);
    }

    let vcxproj_files = read_vcxproj(&mut std::io::BufReader::new(File::open(vcxproj_file)?))?;

    if verbose {
        println!("Generating new filters...");
    }
    let new_filters = generate_new_filters(&mut input_filters, &vcxproj_files, sort)?;

    if verbose {
        println!("Writing new filters to file \"{}\"...", output_file);
    }
    write_filters(&new_filters, output_file.as_str())?;

    Ok(())
}

pub fn find_vcxproj(verbose: bool) -> Vec<String> {
    if verbose {
        println!("Searching for vcxproj files in current folder");
    }

    let mut result: Vec<String> = vec![];
    let paths = fs::read_dir(".").unwrap();
    for path in paths {
        if verbose {}
        if path.is_ok() {
            let value = path.unwrap().path();
            if value.is_file() && value.extension().unwrap_or_default() == "vcxproj" {
                println!("Found vcxproj file \"{}\"", value.to_str().unwrap());
                result.push(String::from_str(value.to_str().unwrap()).unwrap())
            } else if verbose {
                println!("Ignoring entry \"{}\"", value.to_str().unwrap());
            }
        }
    }

    result
}
