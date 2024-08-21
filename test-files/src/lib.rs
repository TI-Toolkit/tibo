#[cfg(test)]
mod tests {
    #[test]
    fn everything_tokenizes() {
        for file in walkdir::WalkDir::new(env!("TESTS_PATH"))
            .into_iter()
            .map(|file| file.unwrap())
            .filter(|file| {
                let name = file.file_name().to_string_lossy();

                name.ends_with(".txt") && !name.to_lowercase().contains("readme")
            })
        {
            let tokenizer = titokens::Tokenizer::new(
                titokens::Version {
                    model: titokens::Model::TI84PCE,
                    os_version: "5.3.0".to_string(),
                },
                "en",
            );

            let (_tokens, _boundaries) = tokenizer
                .tokenize(&std::fs::read_to_string(file.path()).unwrap())
                .unwrap();
        }
    }
}

// Ideally this would be a constant, but I don't want to have to make os_version dyn
pub fn test_version() -> titokens::Version {
    titokens::Version {
        model: titokens::Model::TI84PCE,
        os_version: "5.3.0".to_string(),
    }
}
pub fn load_test_data(file: &str) -> titokens::Tokens {
    let tokenizer = titokens::Tokenizer::new(test_version(), "en");

    let (tokens, _boundaries) = tokenizer
        .tokenize(&std::fs::read_to_string(env!("TESTS_PATH").to_owned() + file).unwrap())
        .unwrap();

    tokens
}
