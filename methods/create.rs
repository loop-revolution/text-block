use crate::blocks::text_block::TextBlock;
use block_tools::blocks::BlockType;
use block_tools::{
	blocks::Context,
	models::{Block, NewBlock},
	Error,
};

impl TextBlock {
	pub fn handle_create(input: String, context: &Context, user_id: i32) -> Result<Block, Error> {
		let conn = &context.pool.get()?;

		let display = Self::data_to_display(&input);
		let data = Self::display_to_data(display);

		let block = NewBlock {
			block_data: Some(data),
			..NewBlock::new(Self::name(), user_id)
		};

		block.insert(conn)
	}
}
