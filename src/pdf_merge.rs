//! Prepare private PDF snapshots, then publish one PDF in the selected page order.
use crate::{
    engine::{self, Progress, cancelled},
    model::{ConversionSettings, MediaKind, OutputFormat, PdfAction},
    pdf_ops,
    platform::set_hidden,
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use tempfile::TempDir;

pub(crate) const MAX_PAGES: usize = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct MergePageRef {
    pub source: usize,
    /// One-based page number in this source's prepared PDF.
    pub page: u32,
}

#[derive(Clone, Debug)]
pub struct MergeSource {
    pub input: PathBuf,
    pub page_count: u32,
}

#[derive(Debug)]
pub struct PreparedPdfMerge {
    _workspace: TempDir,
    sources: Vec<MergeSource>,
    snapshots: Vec<PathBuf>,
}

impl PreparedPdfMerge {
    pub fn sources(&self) -> &[MergeSource] {
        &self.sources
    }

    pub fn pages(&self) -> Vec<MergePageRef> {
        self.sources
            .iter()
            .enumerate()
            .flat_map(|(source, info)| {
                (1..=info.page_count).map(move |page| MergePageRef { source, page })
            })
            .collect()
    }
}

fn validate_settings(settings: &ConversionSettings) -> Result<()> {
    if settings.format != OutputFormat::Pdf || settings.pdf.action != PdfAction::Merge {
        bail!("L'unione richiede destinazione PDF e operazione Merge");
    }
    engine::validate(settings)
}

/// Conversion settings are applied once here. Nothing is written to output_dir.
pub fn prepare_pdf_merge(
    inputs: &[PathBuf],
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<PreparedPdfMerge> {
    cancelled(cancel)?;
    validate_settings(settings)?;
    if inputs.is_empty() || inputs.len() > MAX_PAGES {
        bail!("Per creare un PDF unico seleziona da 1 a {MAX_PAGES} file compatibili");
    }
    let mut local_settings = settings.clone();
    local_settings.format = OutputFormat::Pdf;
    local_settings.pdf.action = PdfAction::Compress;
    let mut originals = Vec::with_capacity(inputs.len());
    for input in inputs {
        cancelled(cancel)?;
        let input = fs::canonicalize(input)
            .with_context(|| format!("File non accessibile: {}", input.display()))?;
        if !input.is_file()
            || !matches!(
                MediaKind::from_path(&input),
                MediaKind::Image
                    | MediaKind::Pdf
                    | MediaKind::Document
                    | MediaKind::Spreadsheet
                    | MediaKind::Html
            )
        {
            bail!(
                "Il file non può essere convertito in PDF: {}",
                input.display()
            );
        }
        if let Some(reason) = tools.missing_for_input(&input, &local_settings) {
            bail!("{}: {reason}", input.display());
        }
        originals.push(input);
    }
    let workspace = tempfile::Builder::new()
        .prefix(".the-converter-merge-")
        .tempdir()?;
    set_hidden(workspace.path())?;
    let mut prepared = PreparedPdfMerge {
        _workspace: workspace,
        sources: Vec::with_capacity(inputs.len()),
        snapshots: Vec::with_capacity(inputs.len()),
    };
    let mut total_pages = 0usize;
    for (index, input) in originals.into_iter().enumerate() {
        cancelled(cancel)?;
        let source_dir = prepared
            ._workspace
            .path()
            .join(format!("source-{index:05}"));
        fs::create_dir(&source_dir)?;
        let label = input.file_name().unwrap_or_default().to_string_lossy();
        progress(Progress {
            fraction: index as f32 / inputs.len() as f32,
            message: format!("Preparazione di {label}…"),
        });
        cancelled(cancel)?;
        let snapshot = if MediaKind::from_path(&input) == MediaKind::Pdf {
            let snapshot = source_dir.join("source.pdf");
            copy_snapshot(&input, &snapshot, cancel)?;
            snapshot
        } else {
            local_settings.output_dir = Some(source_dir);
            let output = engine::convert(&input, &local_settings, tools, cancel, |update| {
                progress(Progress {
                    fraction: (index as f32 + update.fraction * 0.9) / inputs.len() as f32,
                    message: update.message,
                });
            })?;
            if output.len() != 1 {
                bail!("La sorgente deve produrre un solo PDF temporaneo");
            }
            output.into_iter().next().unwrap()
        };
        let page_count = pdf_ops::merge_page_count(&snapshot, cancel)
            .with_context(|| format!("Impossibile preparare {label}"))?;
        total_pages += page_count as usize;
        if total_pages > MAX_PAGES {
            bail!("Il PDF unito non può superare {MAX_PAGES} pagine");
        }
        prepared.snapshots.push(snapshot);
        prepared.sources.push(MergeSource { input, page_count });
        progress(Progress {
            fraction: (index + 1) as f32 / inputs.len() as f32,
            message: format!("Preparata sorgente {} di {}", index + 1, inputs.len()),
        });
    }
    cancelled(cancel)?;
    Ok(prepared)
}

fn copy_snapshot(input: &Path, output: &Path, cancel: &AtomicBool) -> Result<()> {
    let mut input = File::open(input)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        cancelled(cancel)?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            return cancelled(cancel);
        }
        output.write_all(&buffer[..count])?;
    }
}

pub(crate) fn validate_order(order: &[MergePageRef], counts: &[u32]) -> Result<()> {
    let total: usize = counts.iter().map(|&count| count as usize).sum();
    if total == 0 || total > MAX_PAGES || order.len() != total {
        bail!("L'ordine deve contenere tutte le pagine, da 1 a {MAX_PAGES}");
    }
    let mut seen = HashSet::with_capacity(total);
    for page in order {
        if page.page == 0
            || !counts
                .get(page.source)
                .is_some_and(|&count| page.page <= count)
            || !seen.insert(*page)
        {
            bail!("L'ordine contiene pagine duplicate o riferimenti non validi");
        }
    }
    Ok(())
}

/// Save only the prepared snapshots. Import settings are frozen; output_dir can change.
pub fn merge_prepared(
    prepared: &PreparedPdfMerge,
    order: &[MergePageRef],
    settings: &ConversionSettings,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    cancelled(cancel)?;
    validate_settings(settings)?;
    let counts: Vec<_> = prepared
        .sources
        .iter()
        .map(|source| source.page_count)
        .collect();
    validate_order(order, &counts)?;
    pdf_ops::merge_ordered(
        &prepared.snapshots,
        Some(order),
        &prepared.sources[0].input,
        settings.output_dir.as_deref(),
        cancel,
        &mut progress,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_workspace_is_hidden_owned_and_removed_after_cancelled_save() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("source.png");
        image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0]))
            .save(&input)
            .unwrap();
        let mut settings = ConversionSettings {
            format: OutputFormat::Pdf,
            output_dir: Some(directory.path().join("output")),
            ..Default::default()
        };
        settings.pdf.action = PdfAction::Merge;
        let prepared = prepare_pdf_merge(
            &[input],
            &settings,
            &ToolPaths::default(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let workspace = prepared._workspace.path().canonicalize().unwrap();
        assert!(
            prepared
                .snapshots
                .iter()
                .all(|path| path.canonicalize().unwrap().starts_with(&workspace))
        );
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            assert_ne!(fs::metadata(&workspace).unwrap().file_attributes() & 2, 0);
        }
        assert!(
            merge_prepared(
                &prepared,
                &prepared.pages(),
                &settings,
                &AtomicBool::new(true),
                |_| {},
            )
            .is_err()
        );
        drop(prepared);
        assert!(!workspace.exists());
        assert!(!settings.output_dir.unwrap().exists());
    }

    #[test]
    fn page_order_requires_a_complete_unique_bounded_permutation() {
        assert!(validate_order(&[], &[]).is_err());
        assert!(validate_order(&[], &[MAX_PAGES as u32 + 1]).is_err());
        assert!(
            validate_order(
                &[
                    MergePageRef { source: 0, page: 2 },
                    MergePageRef { source: 0, page: 1 }
                ],
                &[2],
            )
            .is_ok()
        );
    }
}
