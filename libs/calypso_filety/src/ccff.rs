use indexmap::{map::IntoIter, IndexMap};

use std::mem;

mod parse;

/// A CCFF container file.
///
/// This will contain all the sections of data (somewhat
/// similar in theory to ELF sections) as well as some basic metadata about the
/// file: its [ABI version](Self::set_abiver) and
/// [file type](Self::set_filety), both of which are user-defined.
///
/// It also stores a list of [`Section`]s, which can be accessed and modified
/// via methods on this structure such as [`get_section`][Self::get_section]
/// and [`add_section`][Self::add_section]. The order of these sections is
/// stored, however this is not guaranteed or required and may be changed in
/// the future.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerFile {
    abiver: u16,
    filety: u8,
    sections: IndexMap<String, Section>,
}

type NomError = nom::Err<nom::error::Error<Vec<u8>>>;

impl ContainerFile {
    /// Create a new container file. The ABI version (`abiver`) and file type
    /// (`filety`) may be any arbitrary user-defined value.
    #[must_use]
    pub fn new(abiver: u16, filety: u8) -> Self {
        Self {
            abiver,
            filety,
            sections: IndexMap::new(),
        }
    }

    /// Set the ABI version of the container file. This may be any arbitrary
    /// user-defined value.
    pub fn set_abiver(&mut self, abiver: u16) {
        self.abiver = abiver;
    }

    /// Get the ABI version of the container file.
    #[must_use]
    pub fn get_abiver(&self) -> u16 {
        self.abiver
    }

    /// Set the file type of the container file. This may be any arbitrary
    /// user-defined value.
    pub fn set_filety(&mut self, filety: u8) {
        self.filety = filety;
    }

    /// Get the file type of the container file.
    #[must_use]
    pub fn get_filety(&self) -> u8 {
        self.filety
    }

    /// Add a section to the container file. If there was already a section
    /// with this name present, it will be replaced and returned.
    ///
    /// # Panics
    ///
    /// This function will panic if the name of the section was longer than 255
    /// characters, if the name of the section contained non-printable ASCII
    /// characters (less than `0x21` or greater than `0x7E`), or if there were
    /// already 255 sections in the container file.
    pub fn add_section(&mut self, name: String, section: Section) -> Option<Section> {
        let name_as_str = name.as_str();
        assert!(
            name_as_str.len() <= 255,
            "section name must be shorter than 256 characters"
        );
        assert!(
            name_as_str.chars().all(|c| c.is_ascii_graphic()),
            "section name must not contain non-printable ASCII characters (less than `0x21` or greater than `0x7E`)"
        );
        assert!(
            self.sections.len() <= 255,
            "container file can only contain up to 255 sections"
        );
        self.sections.insert(name, section)
    }

    /// Remove a section from the container file. The removed section, if any,
    /// will be returned.
    pub fn remove_section(&mut self, name: &str) -> Option<Section> {
        self.sections.shift_remove(name)
    }

    /// Get a reference to a section in the container file.
    #[must_use]
    pub fn get_section(&self, name: &str) -> Option<&Section> {
        self.sections.get(name)
    }

    /// Get a mutable reference to a section in the container file.
    pub fn get_section_mut(&mut self, name: &str) -> Option<&mut Section> {
        self.sections.get_mut(name)
    }

    /// Iterate over the sections in the container file.
    pub fn sections(&self) -> impl Iterator<Item = (&String, &Section)> {
        self.sections.iter()
    }

    /// Iterate mutably over the sections in the container file.
    pub fn sections_mut(&mut self) -> impl Iterator<Item = (&String, &mut Section)> {
        self.sections.iter_mut()
    }

    /// Get the size of the entire container file.
    #[must_use]
    pub fn size(&self) -> usize {
        4 // magic bytes
        + mem::size_of::<u16>() // abiver
        + mem::size_of::<u8>() // filety
        + mem::size_of::<u8>() // len(sections)
        + self.sections().map(|(name, _)| Section::sizeof(name)).sum::<usize>()
        + self.sections().map(|(_, section)| section.get_data().len()).sum::<usize>()
    }

    /// Decode this container file from the buffer provided.
    ///
    /// # Errors
    ///
    /// This function will return an error if the input fails to
    /// parse.
    pub fn decode(buf: &'_ [u8]) -> Result<Self, NomError> {
        Ok(parse::container_file(buf)
            .map_err(
                // This appears to be erroneously triggering here.
                #[allow(clippy::redundant_closure_for_method_calls)]
                |e| e.to_owned(),
            )?
            .1)
    }

    /// Encode this container file to the buffer provided. To allocate a
    /// sufficiently sized buffer, use [`Vec::with_capacity`] using the size
    /// given by [`ContainerFile::size`].
    ///
    /// # Panics
    ///
    /// This function will panic if a section was too large (larger than
    /// [`u32::MAX`] in bytes) or if there was too much data in the container
    /// file (due to architectural limitations, they are capped at around 4GiB)
    // We know that sections.len() will be <=255 as we do not allow adding
    // sections if there are already that amount.
    #[allow(clippy::cast_possible_truncation)]
    pub fn encode_to(self, buf: &mut Vec<u8>) {
        buf.extend(b"CCFF");
        buf.extend(self.abiver.to_le_bytes());
        buf.push(self.filety);
        buf.push(self.sections.len() as u8);

        let shdrs_size = self
            .sections()
            .map(|(name, _)| Section::sizeof(name))
            .sum::<usize>();

        let mut data = Vec::with_capacity(
            self.sections()
                .map(|(_, section)| section.get_data().len())
                .sum(),
        );

        self.sections.into_iter().fold(
            (buf.len() + shdrs_size) as u32,
            |data_offset, (name, section)| {
                let data_size = section.data.len();
                assert!(
                    data_size < u32::MAX as usize,
                    "section data must be less than 4GiB in size"
                );
                let name = name.as_str();
                data.extend(section.data);
                buf.push(section.stype);
                buf.extend(section.flags.to_le_bytes());
                buf.extend(data_offset.to_le_bytes());
                buf.extend((data_size as u32).to_le_bytes());
                buf.push(name.len() as u8);
                buf.extend(name.as_bytes());

                data_offset + data_size as u32
            },
        );
        buf.extend(data);
    }

    /// Encode this container file to a newly allocated buffer.
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.size());
        self.encode_to(&mut buf);
        buf
    }
}

impl IntoIterator for ContainerFile {
    type IntoIter = IntoIter<String, Section>;
    type Item = (String, Section);

    fn into_iter(self) -> Self::IntoIter {
        self.sections.into_iter()
    }
}

/// The section is the actual container of data with in a CCFF container file.
///
/// It stores some basic metadata: a [section type](Section::set_type) and
/// [flags](Section::set_flags), both of which are user-defined. The data can
/// be accessed and modified via the [`get_data`](Self::get_data),
/// [`get_data_mut`](Self::get_data_mut), and [`set_data`](Self::set_data)
/// functions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    stype: u8,
    flags: u32,
    offset: Option<u32>,
    data: Vec<u8>,
}

impl Section {
    /// Create a section. The section type (`stype`) or flags may be any
    /// arbitrary user-defined value.
    #[must_use]
    pub fn new(stype: u8, flags: u32) -> Self {
        Self {
            stype,
            flags,
            offset: None,
            data: Vec::new(),
        }
    }

    /// Set the type of the section. This may be any arbitrary user-defined
    /// value.
    pub fn set_type(&mut self, stype: u8) {
        self.stype = stype;
    }

    /// Get the type of the section.
    #[must_use]
    pub fn get_type(&self) -> u8 {
        self.stype
    }

    /// Set the flags of the section. This may be any arbitrary user-defined
    /// value.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Get the flags of the section.
    #[must_use]
    pub fn get_flags(&self) -> u32 {
        self.flags
    }

    /// Set the data of the section. This may be any arbitrary user-defined
    /// data. The previous data will be returned.
    pub fn set_data(&mut self, data: Vec<u8>) -> Vec<u8> {
        mem::replace(&mut self.data, data)
    }

    /// Get a reference to the data of the section.
    #[must_use]
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the data of the section.
    pub fn get_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Get the offset of the data in the container file. This is only present
    /// when loading from a file and cannot be set manually in order to prevent
    /// errors.
    #[must_use]
    pub fn get_offset(&self) -> Option<u32> {
        self.offset
    }

    fn sizeof(name: &str) -> usize {
        mem::size_of::<u8>() // type
            + mem::size_of::<u32>() // flags
            + mem::size_of::<u32>() // offset
            + mem::size_of::<u32>() // size
            + mem::size_of::<u8>() // sizeof(name)
            + name.len() // name
    }
}
