use block_tools::{
	auth::{
		optional_token, optional_validate_token,
		permissions::{has_perm_level, PermLevel},
	},
	blocks::Context,
	display_api::{component::menus::menu::MenuComponent, DisplayMeta, DisplayObject, PageMeta},
	models::Block,
	LoopError,
};

use crate::blocks::text_block::TextBlock;

impl TextBlock {
	pub fn handle_page_display(
		block: &Block,
		context: &Context,
	) -> Result<DisplayObject, LoopError> {
		let conn = &context.conn()?;
		let user_id = optional_validate_token(optional_token(context))?;

		// Make access to data details easier
		let data = block.block_data.clone().unwrap_or_default();

		// Display API to render
		let value = Self::data_to_display(&data);
		let mut component = Self::editable_component(block.id.to_string(), Some(value));
		component.editable = Some(false);

		let mut page = PageMeta {
			title: Some("Text".to_string()),
			header: Some(format!("Text Block #{}", block.id)),
			..Default::default()
		};

		if let Some(user_id) = user_id {
			let mut menu = MenuComponent::from_block(block, user_id);
			menu.load_comments(conn)?;
			// Add a menu to the page
			page.menu = Some(menu);
			// If the user can edit it the data, make it possible to edit
			if has_perm_level(user_id, block, PermLevel::Edit) {
				component.editable = Some(true);
			}
		}

		let meta = DisplayMeta {
			page: Some(page),
			..Default::default()
		};
		Ok(DisplayObject {
			meta: Some(meta),
			..DisplayObject::new(component.into())
		})
	}
}
