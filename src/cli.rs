pub struct Arguments {
    pub recursive: bool,
    pub pattern: String,
    pub files: Vec<String>,
}

impl Arguments {
    pub fn parse(args: &[String]) -> Result<Arguments, String> {
        let mut recursive = false;
        let mut use_extended = false;
        let mut pattern = None;
        let mut files = Vec::new();
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-r" => recursive = true,
                "-E" => use_extended = true,
                _ => {
                    if pattern.is_none() {
                        pattern = Some(args[i].clone());
                    } else {
                        files.push(args[i].clone());
                    }
                }
            }
            i += 1;
        }
        if !use_extended {
            return Err("Expected '-E' flag".to_string());
        }
        let pattern = pattern.ok_or("Expected a pattern argument".to_string())?;
        Ok(Arguments {
            recursive,
            pattern,
            files,
        })
    }
}