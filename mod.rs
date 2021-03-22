use super::data_block::{self, masked_data_edit};
use block_tools::{
	auth::{
		optional_token, optional_validate_token,
		permissions::{can_view, has_perm_level, PermLevel},
	},
	blocks::{BlockType, Context, TypeInfo},
	display_api::{
		component::{
			card::{error_card, CardComponent, CardHeader},
			icon::Icon,
			input::{InputComponent, InputSize},
			menu::MenuComponent,
			stack::{StackComponent, StackDirection},
			text::{TextComponent, TextPreset},
			DisplayComponent,
		},
		CreationObject, DisplayMeta, DisplayObject, PageMeta,
	},
	dsl::prelude::*,
	models::{Block, MinNewBlock, Property},
	schema::{blocks, properties},
	BlockError, Error,
};
use serde::{Deserialize, Serialize};
pub struct TextBlock {}

pub const BLOCK_NAME: &str = "text";

fn text_properties(
	block: &Block,
	conn: &PgConnection,
	user_id: Option<i32>,
) -> Result<(Option<Block>, Option<Block>), Error> {
	let block_properties: Vec<Property> = properties::dsl::properties
		.filter(properties::dsl::parent_id.eq(block.id))
		.load::<Property>(conn)?;

	let mut name: Option<Block> = None;
	let mut content: Option<Block> = None;

	for property in block_properties {
		if property.property_name == "name" {
			name = blocks::dsl::blocks
				.filter(blocks::id.eq(property.value_id))
				.limit(1)
				.get_result(conn)
				.optional()?;
		} else if property.property_name == "content" {
			content = blocks::dsl::blocks
				.filter(blocks::id.eq(property.value_id))
				.limit(1)
				.get_result(conn)
				.optional()?;
		}
	}

	if let Some(block) = name {
		if !can_view(user_id, &block) {
			name = None;
		} else {
			name = Some(block)
		}
	}

	if let Some(block) = content {
		if !can_view(user_id, &block) {
			content = None;
		} else {
			content = Some(block)
		}
	}

	Ok((name, content))
}

impl BlockType for TextBlock {
	fn name() -> String {
		BLOCK_NAME.to_string()
	}

	fn info() -> TypeInfo {
		TypeInfo {
			name: Self::name(),
			icon: Icon::Type,
			desc: "A piece of text with a name.".to_string(),
		}
	}

	fn page_display(block: &Block, context: &Context) -> Result<DisplayObject, Error> {
		let conn = &context.pool.get()?;
		let user_id = optional_validate_token(optional_token(context))?;
		let (name, content) = text_properties(block, conn, user_id)?;

		let name_string = match name.clone().and_then(|block| block.block_data) {
			Some(string) => string,
			None => "Untitled Block".into(),
		};

		let content: Box<dyn DisplayComponent> = match content {
			Some(block) => match user_id {
				Some(id) if has_perm_level(id, &block, PermLevel::Edit) => {
					box masked_data_edit(block.id.to_string(), block.block_data, false)
						.label("Text...")
				}
				_ => {
					box TextComponent::new(&block.block_data.unwrap_or_else(|| "No content".into()))
				}
			},
			None => box TextComponent::new("No content"),
		};

		let mut page = PageMeta::new();

		if let Some(user_id) = user_id {
			page.menu = Some(MenuComponent::load_from_block(block, user_id));
			if let Some(name) = name {
				if has_perm_level(user_id, &name, PermLevel::Edit) {
					page = page.header_component(
						box masked_data_edit(name.id.to_string(), name.block_data, true)
							.label("Group Name")
							.size(InputSize::Medium),
					)
				} else {
					page = page.header(&name_string)
				}
			}
		} else {
			page = page.header(&name_string)
		}

		Ok(DisplayObject::new(content).meta(DisplayMeta::default().page(page)))
	}

	fn embed_display(block: &Block, context: &Context) -> Box<dyn DisplayComponent> {
		embed_display(block, context).unwrap_or_else(|e| Box::new(error_card(&e.to_string())))
	}

	fn create_display(_context: &Context, _user_id: i32) -> Result<CreationObject, Error> {
		let header = TextComponent::new("New Text Block").preset(TextPreset::Heading);
		let name_input = InputComponent::new().label("Name").name("NAME");
		let content_input = InputComponent::new().label("Text").name("CONTENT");
		let main = StackComponent::new(StackDirection::Vertical)
			.append(Box::new(name_input))
			.append(Box::new(content_input));

		let template: String = r#"{
			"name": $[NAME]$,
			"content": $[CONTENT]$
		}"#
		.split_whitespace()
		.collect();
		let object = CreationObject {
			header_component: Box::new(header),
			main_component: Box::new(main),
			input_template: template,
		};
		Ok(object)
	}

	fn create(input: String, context: &Context, user_id: i32) -> Result<Block, Error> {
		let conn = &context.pool.get()?;
		let input = serde_json::from_str::<CreationArgs>(&input);

		let input: CreationArgs = input.map_err(|_| BlockError::InputParse)?;

		let text_block = MinNewBlock {
			block_type: &TextBlock::name(),
			owner_id: user_id,
		}
		.insert(conn)?;

		let name_block = MinNewBlock {
			block_type: data_block::BLOCK_NAME,
			owner_id: user_id,
		}
		.into()
		.data(&input.name)
		.insert(conn)?;

		let content_block = MinNewBlock {
			block_type: data_block::BLOCK_NAME,
			owner_id: user_id,
		}
		.into()
		.data(&input.content)
		.insert(conn)?;

		text_block
			.make_property("name", name_block.id)
			.insert(conn)?;
		text_block
			.make_property("content", content_block.id)
			.insert(conn)?;

		Ok(text_block)
	}

	fn method_delegate(
		_context: &Context,
		name: String,
		_block_id: i64,
		_args: String,
	) -> Result<Block, Error> {
		Err(BlockError::MethodExist(name, TextBlock::name()).into())
	}

	fn block_name(block: &Block, context: &Context) -> Result<String, Error> {
		let conn = &context.pool.get()?;
		let user_id = optional_validate_token(optional_token(context))?;
		let (name, _) = text_properties(block, conn, user_id)?;
		Ok(match name.and_then(|block| block.block_data) {
			Some(data) => data,
			None => "Text Block".to_string(),
		})
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct CreationArgs {
	name: String,
	content: String,
}

fn embed_display(block: &Block, context: &Context) -> Result<Box<dyn DisplayComponent>, Error> {
	let conn = &context.pool.get()?;
	let user_id = optional_validate_token(optional_token(context))?;
	let (name, content) = text_properties(block, conn, user_id)?;

	let name = name.and_then(|block| block.block_data);
	let content = content.and_then(|block| block.block_data);

	let name = match name {
		Some(string) => string,
		None => "Untitled Block".into(),
	};

	let content = match content {
		Some(string) => TextComponent::new(&string),
		None => TextComponent::new("Empty Block"),
	};

	let menu = match user_id {
		Some(user_id) => Some(MenuComponent::load_from_block(block, user_id)),
		None => None,
	};

	let mut header = CardHeader::new(&name).icon(Icon::Type).id(block.id);
	header.menu = menu;

	let component = CardComponent {
		content: Box::new(content),
		color: None,
		header,
	};
	Ok(Box::new(component))
}
