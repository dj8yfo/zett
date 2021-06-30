use crate::common::*;

#[derive(Debug)]
pub struct Note {
  /// The notes timestamp prefix and name.
  pub id: NoteId,

  /// Where the note is currently stored.
  pub path: PathBuf,

  /// Yaml frontmatter.
  pub matter: Matter,

  /// The notes content as a String.
  pub content: String,
}

impl Note {
  pub fn new(path: PathBuf) -> Self {
    let id = NoteId::parse(path.file_name().unwrap().to_str().unwrap()).unwrap();

    let (matter, content) = matter::matter(&fs::read_to_string(&path).unwrap()).unwrap();

    let matter = Matter::from(matter.as_str());

    Self {
      id,
      path,
      content,
      matter,
    }
  }

  /// Checks if a link exists between the current note and `name`.
  pub fn has_link(&self, name: &str) -> bool {
    if let Some(links) = &self.matter.links {
      if links.contains(&name.to_owned()) {
        return true;
      }
    }
    false
  }

  /// Checks if a tag `name` exists within this notes tags.
  pub fn has_tag(&self, name: &str) -> bool {
    if let Some(tags) = &self.matter.tags {
      if tags.contains(&name.to_owned()) {
        return true;
      }
    }
    false
  }

  /// Attempts to add `name` as a link to the current note.
  pub fn add_link(&self, name: &str) -> Result<(), Error> {
    if self.has_link(name) {
      return Ok(());
    }

    let mut new = Vec::new();

    if let Some(links) = &self.matter.links {
      for link in links {
        new.push(link.to_string());
      }
    }

    new.push(name.to_string());

    let mut file = File::create(&self.path).unwrap();

    file
      .write_all(&Matter::build(&self.matter.name, &self.matter.tags, &Some(new)).as_bytes())
      .unwrap();

    file.write_all(&self.content.as_bytes()).unwrap();

    Ok(())
  }

  /// Attempts to remove `name` as a link from the current note.
  pub fn remove_link(&self, name: &str) -> Result<(), Error> {
    if !self.has_link(name) {
      return Ok(());
    }

    let mut new = Vec::new();

    if let Some(links) = &self.matter.links {
      for link in links {
        if link == name {
          continue;
        }
        new.push(link.to_string());
      }
    }

    let mut file = File::create(&self.path).unwrap();

    file
      .write_all(&Matter::build(&self.matter.name, &self.matter.tags, &Some(new)).as_bytes())
      .unwrap();

    file.write_all(&self.content.as_bytes()).unwrap();

    Ok(())
  }

  /// Attempts to add `name` as a tag to the current note.
  pub fn add_tag(&self, name: &str) -> Result<(), Error> {
    if self.has_tag(name) {
      return Ok(());
    }

    let mut new = Vec::new();

    if let Some(tags) = &self.matter.tags {
      for tag in tags {
        new.push(tag.to_string());
      }
    }

    new.push(name.to_string());

    let mut file = File::create(&self.path).unwrap();

    file
      .write_all(&Matter::build(&self.matter.name, &Some(new), &self.matter.links).as_bytes())
      .unwrap();

    file.write_all(&self.content.as_bytes()).unwrap();

    Ok(())
  }

  /// Attempts to remove `name` as a tag from the current note.
  pub fn remove_tag(&self, name: &str) -> Result<(), Error> {
    if !self.has_tag(name) {
      return Ok(());
    }

    let mut new = Vec::new();

    if let Some(tags) = &self.matter.tags {
      for tag in tags {
        if tag == name {
          continue;
        }
        new.push(tag.to_string());
      }
    }

    let mut file = File::create(&self.path).unwrap();

    file
      .write_all(&Matter::build(&self.matter.name, &self.matter.tags, &Some(new)).as_bytes())
      .unwrap();

    file.write_all(&self.content.as_bytes()).unwrap();

    Ok(())
  }
}
