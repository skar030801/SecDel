use crate::commands::report::ReportData;
use crate::error::{Result, SecDelError};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn generate_pdf_report(data: &ReportData, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .map_err(|e| SecDelError::ReportFailed(format!("Cannot create PDF file: {}", e)))?;
    let mut writer = BufWriter::new(file);

    let (doc, page1, layer1) =
        PdfDocument::new("SecDeL Security Report", Mm(210.0), Mm(297.0), "Title");

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| SecDelError::ReportFailed(format!("Font error: {}", e)))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| SecDelError::ReportFailed(format!("Font error: {}", e)))?;

    let current_layer = doc.get_page(page1).get_layer(layer1);

    current_layer.use_text(
        "SecDeL Security Report",
        24.0,
        Mm(20.0),
        Mm(270.0),
        &font_bold,
    );
    current_layer.use_text(
        &format!("Date: {}", data.timestamp),
        12.0,
        Mm(20.0),
        Mm(255.0),
        &font,
    );
    current_layer.use_text(
        &format!("Version: {}", data.application_version),
        12.0,
        Mm(20.0),
        Mm(248.0),
        &font,
    );
    current_layer.use_text(
        &format!("Operation: {}", data.operation_type),
        12.0,
        Mm(20.0),
        Mm(241.0),
        &font,
    );

    let mut y_pos = 220.0;
    current_layer.use_text("Files Processed", 16.0, Mm(20.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;
    current_layer.use_text(
        &format!("Total Files: {}", data.total_files),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 7.0;
    current_layer.use_text(
        &format!("Total Bytes: {}", data.total_bytes_processed),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 7.0;
    current_layer.use_text(
        &format!("Confidence: {:.1}%", data.confidence_score),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );

    y_pos -= 15.0;
    current_layer.use_text("Detailed File List", 14.0, Mm(20.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;

    for file in &data.files {
        if y_pos < 30.0 {
            break;
        }
        let path_str = if file.path.len() > 60 {
            format!("...{}", &file.path[file.path.len() - 57..])
        } else {
            file.path.clone()
        };
        current_layer.use_text(&path_str, 8.0, Mm(20.0), Mm(y_pos), &font);
        y_pos -= 4.0;
    }

    y_pos -= 10.0;
    current_layer.use_text("Devices Processed", 14.0, Mm(20.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;

    for device in &data.devices {
        if y_pos < 20.0 {
            break;
        }
        current_layer.use_text(
            &format!(
                "{} - {} ({})",
                device.model, device.erase_method, device.path
            ),
            8.0,
            Mm(20.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 4.0;
    }

    let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Details");
    let page2_layer = doc.get_page(page2).get_layer(layer2);
    let mut y2 = 280.0;

    page2_layer.use_text(
        "Security Report Details",
        18.0,
        Mm(20.0),
        Mm(y2),
        &font_bold,
    );
    y2 -= 12.0;
    page2_layer.use_text(
        &format!("Operation Level: {}", data.level),
        10.0,
        Mm(20.0),
        Mm(y2),
        &font,
    );
    y2 -= 8.0;
    page2_layer.use_text(
        &format!("Duration: {:.1} seconds", data.duration_seconds),
        10.0,
        Mm(20.0),
        Mm(y2),
        &font,
    );

    y2 -= 15.0;
    page2_layer.use_text("Notes", 14.0, Mm(20.0), Mm(y2), &font_bold);
    y2 -= 10.0;

    let notes = if data.notes.is_empty() {
        "No additional notes."
    } else {
        &data.notes
    };
    page2_layer.use_text(notes, 10.0, Mm(20.0), Mm(y2), &font);

    doc.save(&mut writer)
        .map_err(|e| SecDelError::ReportFailed(format!("Failed to save PDF: {}", e)))?;

    Ok(())
}
