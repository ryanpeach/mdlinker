use regex::Regex;

/// Gives you ngrams of size 1..=n
/// Stops on boundary pattern
pub fn up_to_n(text: &str, n: usize, boundary_regex: &Regex, spacing_regex: &Regex) -> Vec<String> {
    let mut ngrams = Vec::new();

    // Replace the spacing pattern with a single space
    let text = spacing_regex.replace_all(text, " ");

    // Split the text into segments based on the boundaries (i.e., sentences/phrases)
    let segments: Vec<&str> = boundary_regex.split(&text).collect();

    // Generate n-grams for each segment
    for segment in segments {
        let words: Vec<&str> = segment
            .split_whitespace()
            .filter(|&word| !word.is_empty())
            .collect();

        // Only attempt to create n-grams if there are enough words
        for n in 1..=n {
            if words.len() >= n {
                for i in 0..=words.len().saturating_sub(n) {
                    log::debug!("words: {:?}, i: {:?}, size: {:?}", words, i, n);
                    let ngram = words[i..i + n].join(" ");
                    ngrams.push(ngram);
                }
            }
        }
    }

    ngrams
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::up_to_n;

    const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.";

    /// Mostly used for testing the more efficient `up_to_n`
    fn ngrams(text: &str, n: usize, boundary_pattern: &str) -> Vec<String> {
        let mut ngrams = Vec::new();

        // Replace the spacing pattern with a single space
        let text = Regex::new(r" ")
            .expect("Just a test")
            .replace_all(text, " ");

        // Split the text into segments based on the boundaries (i.e., sentences/phrases)
        let segments: Vec<&str> = Regex::new(boundary_pattern)
            .expect("Just a test")
            .split(&text)
            .collect();

        // Generate n-grams for each segment
        for segment in segments {
            let words: Vec<&str> = segment
                .split_whitespace()
                .filter(|&word| !word.is_empty())
                .collect();

            // Only attempt to create n-grams if there are enough words
            if words.len() >= n {
                for i in 0..=words.len().saturating_sub(n) {
                    let ngram = words[i..i + n].join(" ");
                    ngrams.push(ngram);
                }
            }
        }

        ngrams
    }

    #[cfg(test)]
    mod test_ngrams {
        use std::collections::HashSet;

        use regex::Regex;

        use super::{ngrams, up_to_n, LOREM_IPSUM};
        const TRIGRAMS: &[&str] = &[
            // Sentence 1
            "Lorem ipsum dolor",
            "ipsum dolor sit",
            "dolor sit amet",
            "consectetur adipiscing elit",
            "sed do eiusmod",
            "do eiusmod tempor",
            "eiusmod tempor incididunt",
            "tempor incididunt ut",
            "incididunt ut labore",
            "ut labore et",
            "labore et dolore",
            "et dolore magna",
            "dolore magna aliqua",
            // Sentence 2
            "Ut enim ad",
            "enim ad minim",
            "ad minim veniam",
            "quis nostrud exercitation",
            "nostrud exercitation ullamco",
            "exercitation ullamco laboris",
            "ullamco laboris nisi",
            "laboris nisi ut",
            "nisi ut aliquip",
            "ut aliquip ex",
            "aliquip ex ea",
            "ex ea commodo",
            "ea commodo consequat",
        ];
        const BIGRAMS: &[&str] = &[
            // Sentence 1
            "Lorem ipsum",
            "ipsum dolor",
            "dolor sit",
            "sit amet",
            "consectetur adipiscing",
            "adipiscing elit",
            "sed do",
            "do eiusmod",
            "eiusmod tempor",
            "tempor incididunt",
            "incididunt ut",
            "ut labore",
            "labore et",
            "et dolore",
            "dolore magna",
            "magna aliqua",
            // Sentence 2
            "Ut enim",
            "enim ad",
            "ad minim",
            "minim veniam",
            "quis nostrud",
            "nostrud exercitation",
            "exercitation ullamco",
            "ullamco laboris",
            "laboris nisi",
            "nisi ut",
            "ut aliquip",
            "aliquip ex",
            "ex ea",
            "ea commodo",
            "commodo consequat",
        ];
        const MONOGRAMS: &[&str] = &[
            "Lorem",
            "ipsum",
            "dolor",
            "sit",
            "amet",
            "consectetur",
            "adipiscing",
            "elit",
            "sed",
            "do",
            "eiusmod",
            "tempor",
            "incididunt",
            "ut",
            "labore",
            "et",
            "dolore",
            "magna",
            "aliqua",
            "Ut",
            "enim",
            "ad",
            "minim",
            "veniam",
            "quis",
            "nostrud",
            "exercitation",
            "ullamco",
            "laboris",
            "nisi",
            "ut",
            "aliquip",
            "ex",
            "ea",
            "commodo",
            "consequat",
        ];

        #[test]
        fn test_trigrams() {
            let out = ngrams(LOREM_IPSUM, 3, r"[,.]");
            assert_eq!(out, TRIGRAMS);
        }

        #[test]
        fn test_bigrams() {
            let out = ngrams(LOREM_IPSUM, 2, r"[,.]");
            assert_eq!(out, BIGRAMS);
        }

        #[test]
        fn test_monograms() {
            let out = ngrams(LOREM_IPSUM, 1, r"[,.]");
            assert_eq!(out, MONOGRAMS);
        }

        #[test]
        fn test_up_to() {
            for n in (1..=3).rev() {
                let up_to_out = HashSet::from_iter(up_to_n(
                    LOREM_IPSUM,
                    n,
                    &Regex::new(r"[,.]").expect("Just a test"),
                    &Regex::new(r" ").expect("Just a test"),
                ));
                let mut out = HashSet::new();
                for m in 1..=n {
                    let to = ngrams(LOREM_IPSUM, m, r"[,.]");
                    out.extend(to);
                }
                assert_eq!(up_to_out, out, "ngrams_up_to {n:?} are not the same");
            }
        }
    }
}
