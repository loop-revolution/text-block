use block_tools::{
	auth::{
		optional_token, optional_validate_token,
		permissions::{has_perm_level, PermLevel},
	},
	blocks::Context,
	display_api::component::{
		atomic::icon::Icon,
		layout::card::{CardComponent, CardHeader},
		menus::menu::MenuComponent,
		DisplayComponent,
	},
	models::Block,
	LoopError,
};

use super::super::TextBlock;

impl TextBlock {
	pub fn handle_embed_display(
		block: &Block,
		context: &Context,
	) -> Result<DisplayComponent, LoopError> {
		let conn = &context.conn()?;
		let user_id = optional_validate_token(optional_token(context)).unwrap();
		let data = block.block_data.clone().unwrap_or_default();

		let value = Self::data_to_display(&data);
		let mut card_content = Self::editable_component(block.id.to_string(), Some(value));
		card_content.editable = Some(false);

		if let Some(user_id) = user_id {
			// If the user can edit the data, make it possible to edit
			if has_perm_level(user_id, block, PermLevel::Edit) {
				card_content.editable = Some(true);
			}
		}

		let mut header = CardHeader {
			icon: Some(Icon::Type),
			block_id: Some(block.id.to_string()),
			..CardHeader::new("Text")
		};
		if let Some(user_id) = user_id {
			let mut menu = MenuComponent::from_block(block, user_id);
			menu.load_comments(conn)?;
			header.menu = Some(menu);
		}

		Ok(CardComponent {
			content: box card_content.into(),
			color: block.color.clone(),
			header: box header,
		}
		.into())
	}
}
