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

pub use titokens::{Model, Tokenizer, Version};

#[macro_export]
macro_rules! test_version {
    () => {
        $crate::Version {
            model: $crate::Model::TI84PCE,
            os_version: "5.3.0".to_string(),
        }
    };
}

#[macro_export]
macro_rules! test_tokenizer {
    () => {
        $crate::Tokenizer::new($crate::test_version!(), "en")
    };
}

pub fn load_test_data(file: &str) -> titokens::Tokens {
    let tokenizer = test_tokenizer!();

    let (tokens, _boundaries) = tokenizer
        .tokenize(&std::fs::read_to_string(env!("TESTS_PATH").to_owned() + file).unwrap())
        .unwrap();

    tokens
}
