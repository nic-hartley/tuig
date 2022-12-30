use std::collections::HashMap;

use crate::agents::tools::AutocompleteType;

/// Allows for easy completion of BSD-style commands, e.g.:
/// 
/// - `tar |` -> tries to complete x, f, etc. (the various option characters)
/// - `tar x|` -> the same thing
/// - `tar x |` -> doesn't try to complete anything; as far as it cares the optoins are done
/// - `tar xf |` -> tries to autocomplete a filename
/// - `tar xf abc|` -> tries to autocomplete a filename starting with `abc`
pub struct BsdCompleter {
    /// The options this completer can complete
    options: HashMap<char, Option<AutocompleteType>>,
}


