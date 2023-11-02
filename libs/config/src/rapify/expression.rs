use hemtt_common::io::WriteExt;

use crate::Expression;

use super::Rapify;

impl Rapify for Expression {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_cstring(&self.value)?;
        Ok(self.value.len() + 1)
    }

    fn rapified_length(&self) -> usize {
        self.value.len() + 1
    }

    fn rapified_code(&self) -> u8 {
        4
    }
}

#[cfg(test)]
mod tests {
    use crate::Expression;

    use super::Rapify;

    #[test]
    fn str() {
        let mut buffer = Vec::new();
        let written = Expression {
            value: "getScreenSize".to_string(),
            span: 0..14,
        }
        .rapify(&mut buffer, 0)
        .unwrap();
        assert_eq!(written, 14);
        assert_eq!(
            buffer,
            vec![103, 101, 116, 83, 99, 114, 101, 101, 110, 83, 105, 122, 101, 0]
        );
    }
}
