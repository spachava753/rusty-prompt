mod key;
mod completion;

#[cfg(test)]
mod tests {
    use std::io::{stdout, Write};
    use crossterm::{
        ExecutableCommand, QueueableCommand,
        terminal, cursor, style::{self, Stylize}, Result,
    };

    #[test]
    fn it_works() -> Result<()> {
        let mut stdout = stdout();

        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        for y in 0..40 {
            for x in 0..150 {
                if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                    // in this loop we are more efficient by not flushing the buffer.
                    stdout
                        .queue(cursor::MoveTo(x, y))?
                        .queue(style::PrintStyledContent("█".magenta()))?;
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }
}
