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
		misc::richtext::RichTextComponent,
		DisplayComponent,
	},
	models::Block,
};

use super::super::TextBlock;

impl TextBlock {
	pub fn handle_embed_display(block: &Block, context: &Context) -> DisplayComponent {
		let user_id = optional_validate_token(optional_token(context)).unwrap();

		let card_content = Self::rightfully_editable_richtext(user_id, block);

		let mut header = CardHeader {
			icon: Some(Icon::Type),
			block_id: Some(block.id.to_string()),
			..CardHeader::new("Text")
		};
		if let Some(user_id) = user_id {
			header.menu = Some(MenuComponent::from_block(block, user_id));
		}

		CardComponent {
			content: box card_content.into(),
			color: block.color.clone(),
			header: box header,
		}
		.into()
	}
}

impl TextBlock {
	pub fn rightfully_editable_richtext(user_id: Option<i32>, block: &Block) -> RichTextComponent {
		let data = block.block_data.clone().unwrap_or_default();

		let value = Self::data_to_display(&data);
		let mut component = Self::editable_component(block.id.to_string(), Some(value));
		component.editable = Some(false);

		if let Some(user_id) = user_id {
			// If the user can edit the data, make it possible to edit
			if has_perm_level(user_id, block, PermLevel::Edit) {
				component.editable = Some(true);
			}
		}

		component
	}
}
