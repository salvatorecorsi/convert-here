//! Office exports and editable, text-only reconstruction of PDFs.
//!
//! PDF reconstruction deliberately uses embedded text only: it does not infer
//! tables, retain images or reproduce the source typography.
use crate::{
    engine::{Progress, cancelled, commit, copy_commit, run_process, stem, temporary},
    model::{ConversionSettings, CsvDelimiter, OutputFormat},
    tools::ToolPaths,
};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::{Seek, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

const MAX_EXTRACTED_TEXT_BYTES: u64 = 64 * 1024 * 1024;

fn csv_filter(delimiter: CsvDelimiter, export: bool) -> String {
    let separator = match delimiter {
        CsvDelimiter::Comma => 44,
        CsvDelimiter::Semicolon => 59,
        CsvDelimiter::Tab => 9,
        CsvDelimiter::Pipe => 124,
    };
    // Explicit UTF-8 and double quotes; export only the first worksheet's
    // displayed values. Import token 13 must be present and false to prevent
    // evaluating formula expressions. Quoted fields, spaces and IDs stay text.
    if export {
        format!(
            "csv:Text - txt - csv (StarCalc):{separator},34,76,1,,1033,true,true,true,false,false,0,false"
        )
    } else {
        format!(
            "Text - txt - csv (StarCalc):{separator},34,76,1,,1033,true,false,true,false,false,0,false,false,false"
        )
    }
}

pub(crate) fn convert_office(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    let format = settings.format;
    let filter = match format {
        OutputFormat::Pdf => "pdf".to_owned(),
        OutputFormat::Csv => csv_filter(settings.csv.delimiter, true),
        OutputFormat::Xlsx => "xlsx:Calc MS Excel 2007 XML".to_owned(),
        _ => bail!("Formato Office di destinazione non supportato: {format}"),
    };
    let stage = tempfile::Builder::new()
        .prefix(".the-converter-office-")
        .tempdir_in(dir)?;
    let profile = stage.path().join("profile");
    prepare_profile(&profile)?;
    let profile_uri = profile_uri(&profile)?;
    let mut cmd = Command::new(tools.soffice.as_ref().context("LibreOffice non trovato")?);
    cmd.arg(format!("-env:UserInstallation={profile_uri}"))
        .args([
            "--headless",
            "--nologo",
            "--nodefault",
            "--nofirststartwizard",
        ]);
    if input
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
    {
        cmd.arg(format!(
            "--infilter={}",
            csv_filter(settings.csv.delimiter, false)
        ));
    }
    cmd.args(["--convert-to", &filter, "--outdir"])
        .arg(stage.path())
        .arg(input);
    progress(Progress {
        fraction: 0.1,
        message: "Conversione documento con LibreOffice…".into(),
    });
    run_process(cmd, cancel, None, progress)?;
    cancelled(cancel)?;
    let output = stage
        .path()
        .join(format!("{}.{}", stem(input), format.extension()));
    // LibreOffice can report success even when its filter rejects the input.
    if !output.is_file() {
        bail!(
            "LibreOffice non ha prodotto il file {}. Il documento potrebbe essere protetto o non supportato.",
            format.label()
        );
    }
    copy_commit(&output, dir, &stem(input), format.extension())
}

fn profile_uri(profile: &Path) -> Result<String> {
    // Windows canonical paths have a verbatim prefix not supported by file URLs.
    let text = profile.to_string_lossy();
    let path = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{unc}"))
    } else if let Some(local) = text.strip_prefix(r"\\?\") {
        PathBuf::from(local)
    } else {
        profile.to_owned()
    };
    url::Url::from_directory_path(path)
        .map(String::from)
        .map_err(|_| anyhow::anyhow!("Percorso del profilo LibreOffice non valido"))
}

fn prepare_profile(profile: &Path) -> Result<()> {
    let user = profile.join("user");
    fs::create_dir_all(&user)?;
    // Each export uses a disposable profile, so a running LibreOffice GUI cannot
    // intercept the conversion or contribute saved import/security preferences.
    // Writer and Calc use different enumerations for "never update links".
    fs::write(
        user.join("registrymodifications.xcu"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<oor:items xmlns:oor="http://openoffice.org/2001/registry">
 <item oor:path="/org.openoffice.Office.Common/Security/Scripting">
  <prop oor:name="DisableMacrosExecution" oor:op="fuse"><value>true</value></prop>
  <prop oor:name="DisableActiveContent" oor:op="fuse"><value>true</value></prop>
  <prop oor:name="DisableOLEAutomation" oor:op="fuse"><value>true</value></prop>
  <prop oor:name="MacroSecurityLevel" oor:op="fuse"><value>3</value></prop>
 </item>
 <item oor:path="/org.openoffice.Office.Writer/Content/Update">
  <prop oor:name="Link" oor:op="fuse"><value>2</value></prop>
 </item>
 <item oor:path="/org.openoffice.Office.Calc/Content/Update">
  <prop oor:name="Link" oor:op="fuse"><value>1</value></prop>
 </item>
</oor:items>"#,
    )?;
    Ok(())
}

pub(crate) fn convert_pdf_text(
    input: &Path,
    dir: &Path,
    settings: &ConversionSettings,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<PathBuf> {
    cancelled(cancel)?;
    let format = settings.format;
    if !matches!(
        format,
        OutputFormat::Txt | OutputFormat::Docx | OutputFormat::Html
    ) {
        bail!("Formato di ricostruzione PDF non supportato: {format}");
    }
    let text = if settings.ocr.enabled {
        crate::ocr::extract_text(input, dir, settings, tools, cancel, progress)?
    } else {
        embedded_pdf_text(input, dir, format, tools, cancel, progress)?
    };
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut pages: Vec<&str> = normalized.split('\u{c}').collect();
    if normalized.ends_with('\u{c}') {
        pages.pop();
    }
    let mut output = temporary(dir, format.extension())?;
    match format {
        OutputFormat::Txt => output.as_file_mut().write_all(text.as_bytes())?,
        OutputFormat::Html => write_html(output.as_file_mut(), &stem(input), &pages, cancel)?,
        OutputFormat::Docx => write_docx(output.as_file_mut(), &pages, cancel)?,
        _ => unreachable!(),
    }
    cancelled(cancel)?;
    commit(output, dir, &stem(input), format.extension())
}

fn embedded_pdf_text(
    input: &Path,
    dir: &Path,
    format: OutputFormat,
    tools: &ToolPaths,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(Progress),
) -> Result<String> {
    let text = temporary(dir, "txt")?;
    let mut cmd = Command::new(tools.pdftotext.as_ref().context("pdftotext non trovato")?);
    // The default reading-order extraction avoids baking in column spacing.
    // Keep form feeds so source page boundaries survive reconstruction.
    if format == OutputFormat::Txt {
        cmd.arg("-layout");
    }
    cmd.args(["-enc", "UTF-8"]).arg(input).arg(text.path());
    progress(Progress {
        fraction: 0.1,
        message: "Estrazione del testo incorporato nel PDF…".into(),
    });
    run_process(cmd, cancel, None, progress).map_err(crate::engine::pdf_error)?;
    cancelled(cancel)?;
    if text.as_file().metadata()?.len() > MAX_EXTRACTED_TEXT_BYTES {
        bail!("Il testo estratto dal PDF supera il limite di 64 MB");
    }
    let text = fs::read_to_string(text.path()).context("Testo PDF non valido in UTF-8")?;
    if text.trim().is_empty() {
        bail!(
            "Il PDF non contiene testo estraibile. Attiva OCR locale per riconoscere il testo delle scansioni."
        );
    }
    Ok(text)
}

fn escape_xml(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            // XML 1.0 permits these codepoint ranges. PDF extraction may contain
            // control characters that would otherwise make Word reject the XML.
            '\t'
            | '\n'
            | '\r'
            | '\u{20}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}'
            | '\u{10000}'..='\u{10ffff}' => escaped.push(character),
            _ => {}
        }
    }
    escaped
}

fn write_html(
    output: &mut impl Write,
    title: &str,
    pages: &[&str],
    cancel: &AtomicBool,
) -> Result<()> {
    writeln!(
        output,
        "<!doctype html>\n<html><head><meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>body{{font-family:system-ui,sans-serif;line-height:1.5;max-width:52rem;margin:2rem auto;padding:0 1rem}}.page{{break-after:page;margin-bottom:2rem}}.page:last-child{{break-after:auto}}p{{white-space:pre-wrap;overflow-wrap:anywhere;min-height:1em;margin:0}}@media print{{body{{max-width:none;margin:0}}}}</style></head><body>",
        escape_xml(title)
    )?;
    for (index, page) in pages.iter().enumerate() {
        cancelled(cancel)?;
        writeln!(
            output,
            "<section class=\"page\" aria-label=\"Pagina {}\">",
            index + 1
        )?;
        for line in page.trim_end_matches('\n').split('\n') {
            writeln!(output, "<p>{}</p>", escape_xml(line))?;
        }
        writeln!(output, "</section>")?;
    }
    writeln!(output, "</body></html>")?;
    Ok(())
}

fn write_docx(output: &mut (impl Write + Seek), pages: &[&str], cancel: &AtomicBool) -> Result<()> {
    let mut archive = ZipWriter::new(output);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    archive.start_file("[Content_Types].xml", options)?;
    archive.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#)?;
    archive.start_file("_rels/.rels", options)?;
    archive.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#)?;
    archive.start_file("word/document.xml", options)?;
    archive.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#)?;
    for (index, page) in pages.iter().enumerate() {
        cancelled(cancel)?;
        for (line_index, line) in page.trim_end_matches('\n').split('\n').enumerate() {
            archive.write_all(b"<w:p>")?;
            if index > 0 && line_index == 0 {
                archive.write_all(b"<w:pPr><w:pageBreakBefore/></w:pPr>")?;
            }
            archive.write_all(b"<w:r>")?;
            for (segment_index, segment) in line.split('\t').enumerate() {
                if segment_index > 0 {
                    archive.write_all(b"<w:tab/>")?;
                }
                write!(
                    archive,
                    "<w:t xml:space=\"preserve\">{}</w:t>",
                    escape_xml(segment)
                )?;
            }
            archive.write_all(b"</w:r></w:p>")?;
        }
    }
    archive.write_all(br#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1134" w:right="1134" w:bottom="1134" w:left="1134"/></w:sectPr></w:body></w:document>"#)?;
    archive.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn docx_has_resolvable_parts_escaped_text_and_source_page_breaks() {
        let mut output = Cursor::new(Vec::new());
        write_docx(
            &mut output,
            &["Città & <mare>\t001\u{0}\n", "Pagina due"],
            &AtomicBool::new(false),
        )
        .unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(output.into_inner())).unwrap();
        assert_eq!(archive.len(), 3);
        let mut relationships = String::new();
        archive
            .by_name("_rels/.rels")
            .unwrap()
            .read_to_string(&mut relationships)
            .unwrap();
        assert!(relationships.contains("Target=\"word/document.xml\""));
        let mut document = String::new();
        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut document)
            .unwrap();
        assert!(document.contains("Città &amp; &lt;mare&gt;"));
        assert!(document.contains("<w:tab/>") && document.contains("001"));
        assert!(!document.contains('\u{0}'));
        assert_eq!(document.matches("<w:pageBreakBefore/>").count(), 1);
        assert!(document.contains("Pagina due"));
    }

    #[test]
    fn html_is_self_contained_and_pdf_text_cannot_become_markup() {
        let mut output = Vec::new();
        write_html(
            &mut output,
            "<titolo>",
            &["<script src='https://example.com'> & \"", "Seconda pagina"],
            &AtomicBool::new(false),
        )
        .unwrap();
        let html = String::from_utf8(output).unwrap();
        assert!(html.contains("<title>&lt;titolo&gt;</title>"));
        assert!(!html.contains("<script"));
        assert!(html.contains("&lt;script src=&apos;https://example.com&apos;&gt; &amp; &quot;"));
        assert_eq!(html.matches("<section ").count(), 2);
        assert!(!html.contains("<link") && !html.contains("<img"));
    }
}
