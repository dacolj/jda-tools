# vcxproj-filters

Naive vcxproj filters generation and update utility. Filters are computed according filesystem organisation and sorted to limit VCS conflicts.

## Build

Run `cargo build --release`.

## Usage

```
filters.exe [FLAGS] [OPTIONS] [input]...

FLAGS:
    -h, --help               Prints help information
    -i, --ignore-existing    Ignore existing filter file
    -k, --keep-order         Don't sort file and groups
    -V, --version            Prints version information
    -v, --verbose            Verbose output

OPTIONS:
    -o, --output <output>    Output filters file if you don't want to override input (requires a single vcxproj as input)

ARGS:
    <input>...    Input vcxproj file(s)
```

## Limitations

- Generated filters can only match filesystem organisation.
- _Extension_ tag of _Filter_ is not managed, for example a ``<Extensions>h;hh;hpp;hxx;h++;hm;inl;inc;ipp;xsd</Extensions>`` line will be ignored.
- Limited to the XML version of VS filters used by the author (but not sure they are different versions).
- And of course no guarantee that it will works for you.
