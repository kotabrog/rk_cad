use crate::old::step_file::StepFile;
use std::io::Write;

pub fn write_step_file(step: &StepFile, mut w: impl Write) -> std::io::Result<()> {
    for l in &step.header {
        writeln!(w, "{l}")?;
    }
    writeln!(w, "DATA;")?;
    for e in &step.entities {
        writeln!(w, "#{} = {}({});", e.id, e.keyword, e.params)?;
    }
    writeln!(w, "ENDSEC;")?;
    for l in &step.trailer {
        writeln!(w, "{l}")?;
    }
    Ok(())
}
