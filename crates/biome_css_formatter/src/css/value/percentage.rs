use crate::prelude::*;
use biome_css_syntax::{CssPercentage, CssPercentageFields};
use biome_formatter::write;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatCssPercentage;
impl FormatNodeRule<CssPercentage> for FormatCssPercentage {
    fn fmt_fields(&self, node: &CssPercentage, f: &mut CssFormatter) -> FormatResult<()> {
        let CssPercentageFields {
            value,
            reminder_token,
        } = node.as_fields();

        write!(f, [value.format(), reminder_token.format()])
    }
}
