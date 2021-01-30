use async_trait::async_trait;
use block_tools::{
	blocks::{BlockType, Context},
	display_api::{
		component::{
			card::{CardComponent, CardHeader, CardIcon},
			input::InputComponent,
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
use serde_json::json;
pub struct TextBlock {}

fn text_properties(
	block: &Block,
	context: &Context,
) -> Result<(Option<Block>, Option<Block>), Error> {
	let conn = &context.pool.get()?;

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

	Ok((name, content))
}

#[async_trait]
impl BlockType for TextBlock {
	fn name(&self) -> &str {
		"text"
	}

	async fn page_display(block: &Block, context: &Context) -> Result<DisplayObject, Error> {
		let (name, content) = text_properties(block, context)?;

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

		Ok(DisplayObject::new(Box::new(content))
			.meta(DisplayMeta::new().page(PageMeta::new().header(&name))))
	}

	async fn embed_display(
		block: &Block,
		context: &Context,
	) -> Result<Box<dyn DisplayComponent>, Error> {
		let (name, content) = text_properties(block, context)?;

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
		let component = CardComponent {
			content: Box::new(content),
			color: None,
			header: CardHeader {
				title: name,
				icon: Some(CardIcon::Type),
				block_id: Some(block.id.to_string()),
			},
		};
		Ok(Box::new(component))
	}

	async fn create_display(_context: &Context, _user_id: i32) -> Result<CreationObject, Error> {
		let header = TextComponent::new("New Text Block").preset(TextPreset::Heading);
		let name_input = InputComponent::new().label("Name").name("NAME");
		let content_input = InputComponent::new().label("Text").name("CONTENT");
		let main = StackComponent::new(StackDirection::Vertical)
			.add(Box::new(name_input))
			.add(Box::new(content_input));

		let template = json!({
			"name": "$[NAME]$",
			"content": "$[CONTENT]$",
		});
		let object = CreationObject {
			header_component: Box::new(header),
			main_component: Box::new(main),
			input_template: template.to_string(),
		};
		Ok(object)
	}

	async fn create(input: String, context: &Context, user_id: i32) -> Result<Block, Error> {
		let conn = &context.pool.get()?;
		let input: CreationArgs =
			serde_json::from_str(&input).map_err(|_| BlockError::InputParse)?;

		let text_block = MinNewBlock {
			block_type: "text",
			owner_id: user_id,
		}
		.insert(conn)?;

		let name_block = MinNewBlock {
			block_type: "data",
			owner_id: user_id,
		}
		.into()
		.data(input.name)
		.insert(conn)?;

		let content_block = MinNewBlock {
			block_type: "data",
			owner_id: user_id,
		}
		.into()
		.data(input.content)
		.insert(conn)?;

		text_block
			.make_property("name", name_block.id)
			.insert(conn)?;
		text_block
			.make_property("content", content_block.id)
			.insert(conn)?;

		Ok(text_block)
	}
}

#[derive(Serialize, Deserialize)]
struct CreationArgs<'a> {
	name: &'a str,
	content: &'a str,
}
