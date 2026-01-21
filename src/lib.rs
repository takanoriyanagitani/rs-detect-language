use std::io;

use io::Read;

use io::BufWriter;
use io::Write;

use lingua::IsoCode639_1;
use lingua::IsoCode639_3;

use lingua::Language;

use lingua::LanguageDetector;
use lingua::LanguageDetectorBuilder;

#[derive(Debug)]
pub enum LangDetectErr {
    TooSmallDistance(f64),
    TooGreatDistance(f64),
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    pub minimum_relative_distance: Option<f64>,
    pub enable_low_accuracy_mode: bool,
    pub enable_preload: bool,
}

impl Config {
    pub fn with_min_rel_dist(mut self, raw_dist: f64) -> Result<Self, LangDetectErr> {
        if raw_dist < 0.0 {
            return Err(LangDetectErr::TooSmallDistance(raw_dist));
        }

        if 0.99 <= raw_dist {
            return Err(LangDetectErr::TooGreatDistance(raw_dist));
        }

        self.minimum_relative_distance = Some(raw_dist);

        Ok(self)
    }

    pub fn disable_high_accuracy_mode(mut self) -> Self {
        self.enable_low_accuracy_mode = true;
        self
    }

    pub fn disable_lazy_load(mut self) -> Self {
        self.enable_preload = true;
        self
    }
}

impl Config {
    pub fn build_from_builder(self, mut bldr: LanguageDetectorBuilder) -> LanguageDetector {
        if self.enable_low_accuracy_mode {
            bldr.with_low_accuracy_mode();
        }

        if let Some(dist) = self.minimum_relative_distance {
            bldr.with_minimum_relative_distance(dist);
        }

        if self.enable_preload {
            bldr.with_preloaded_language_models();
        }

        bldr.build()
    }

    pub fn build_from_all_languages(self) -> LanguageDetector {
        let bldr = LanguageDetectorBuilder::from_all_languages();
        self.build_from_builder(bldr)
    }

    pub fn build_from_all_spoken_languages(self) -> LanguageDetector {
        let bldr = LanguageDetectorBuilder::from_all_spoken_languages();
        self.build_from_builder(bldr)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DetectedLanguage {
    pub lang: Language,
    pub iso_639_1: IsoCode639_1,
    pub iso_639_3: IsoCode639_3,
    pub confidence: f64,
}

impl DetectedLanguage {
    pub fn text2languages<S>(d: &LanguageDetector, txt: S) -> impl Iterator<Item = Self>
    where
        S: Into<String>,
    {
        d.compute_language_confidence_values(txt)
            .into_iter()
            .map(|t| Self {
                lang: t.0,
                iso_639_1: t.0.iso_code_639_1(),
                iso_639_3: t.0.iso_code_639_3(),
                confidence: t.1,
            })
    }

    pub fn reader2languages<R>(
        d: &LanguageDetector,
        rdr: R,
        limit: u64,
    ) -> Result<impl Iterator<Item = Self>, io::Error>
    where
        R: Read,
    {
        let mut taken = rdr.take(limit);
        let mut buf: String = String::default();
        taken.read_to_string(&mut buf)?;
        Ok(Self::text2languages(d, buf))
    }

    pub fn stdin2languages(
        d: &LanguageDetector,
        limit: u64,
    ) -> Result<impl Iterator<Item = Self>, io::Error> {
        Self::reader2languages(d, io::stdin().lock(), limit)
    }
}

impl DetectedLanguage {
    pub fn to_writer<W>(&self, wtr: &mut W) -> Result<(), io::Error>
    where
        W: Write,
    {
        serde_json::to_writer(wtr, self)?;
        Ok(())
    }

    pub fn to_writer_all<I, W>(results: I, mut wtr: W) -> Result<(), io::Error>
    where
        I: Iterator<Item = Self>,
        W: Write,
    {
        for rslt in results {
            rslt.to_writer(&mut wtr)?;
            writeln!(wtr)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn to_stdout_all<I>(results: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Self>,
    {
        let o = io::stdout();
        let mut ol = o.lock();
        Self::to_writer_all(results, BufWriter::new(&mut ol))?;
        ol.flush()
    }
}

impl DetectedLanguage {
    pub fn stdin2langs2stdout(
        d: &LanguageDetector,
        limit: u64,
        result_limit: Option<usize>,
    ) -> Result<(), io::Error> {
        let langs = Self::stdin2languages(d, limit)?;
        if let Some(n) = result_limit {
            return Self::to_stdout_all(langs.take(n));
        }
        Self::to_stdout_all(langs)
    }
}

pub fn print_detected_from_stdin(
    d: &LanguageDetector,
    limit: u64,
    result_limit: Option<usize>,
) -> Result<(), io::Error> {
    DetectedLanguage::stdin2langs2stdout(d, limit, result_limit)
}
