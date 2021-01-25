use async_trait::async_trait;
use block_tools::{Error, blocks::{BlockType, Context}, display_api::{CreationObject, DisplayMeta, DisplayObject, MethodObject, PageMeta, component::{DisplayComponent, card::{CardComponent, CardHeader, CardIcon}, input::InputComponent, text::{TextComponent, TextPreset}}}, dsl, dsl::prelude::*, models::{BlockD, NewBlock, PropertyD}, schema::{blocks, properties}};

pub struct TextBlock {}

#[async_trait]
impl BlockType for TextBlock {
	fn name(&self) -> &str {
		"text"
	}

	async fn page_display(block: &BlockD, context: &Context) -> Result<DisplayObject, Error> {
		let conn = &context.pool.get()?;

		let block_properties: Vec<PropertyD> = properties::dsl::properties
			.filter(properties::dsl::parent_id.eq(block.id))
			.load::<PropertyD>(conn)?;

		let mut name: Option<BlockD> = None;
		let mut content: Option<BlockD> = None;

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

		let data = || match &block.block_data {
			Some(str) => Some(String::from(str)),
			None => None,
		};
		let component = TextComponent {
			text: data().unwrap_or("".into()),
			color: None,
			preset: None,
		};

		let meta = DisplayMeta {
			page: Some(PageMeta {
				title: Some("Data".into()),
				header: data(),
			}),
		};
		Ok(DisplayObject {
			display: Box::new(component),
			meta: Some(meta),
		})
	}

	async fn embed_display(
		block: &BlockD,
		_context: &Context,
	) -> Result<Box<dyn DisplayComponent>, Error> {
		let data: Option<String> = block.clone().block_data.clone();
		let card_content = TextComponent {
			text: data.unwrap_or("".into()),
			color: None,
			preset: None,
		};
		let component = CardComponent {
			content: Box::new(card_content),
			color: None,
			header: CardHeader {
				title: "Data".into(),
				icon: Some(CardIcon::Box),
				block_id: Some(block.id.to_string()),
			},
		};
		Ok(Box::new(component))
	}

	async fn create_display(_context: &Context, _user_id: i32) -> Result<CreationObject, Error> {
		let header = TextComponent {
			preset: Some(TextPreset::Heading),
			text: "New Data Block".into(),
			color: None,
		};
		let main = InputComponent {
			label: Some("Data".into()),
			name: Some("DATA".into()),
			confirm_cancel: None,
			input_type: None,
			initial_value: None,
		};
		let object = CreationObject {
			header_component: Box::new(header),
			main_component: Box::new(main),
			input_template: "$[DATA]$".into(),
		};
		Ok(object)
	}

	async fn create(input: String, context: &Context, user_id: i32) -> Result<BlockD, Error> {
		let conn = &context.pool.get()?;

		let block = NewBlock {
			block_type: "data",
			created_at: std::time::SystemTime::now(),
			updated_at: std::time::SystemTime::now(),
			block_data: Some(input.as_str()),
			owner_id: user_id,
		};

		Ok(dsl::insert_into(blocks::table)
			.values(&block)
			.get_result(conn)?)
	}
}
