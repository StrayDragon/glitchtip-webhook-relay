use crate::types::*;

pub struct Converter;

impl Converter {
    pub fn glitchtip_to_feishu_text(glitchtip: &GlitchTipSlackWebhook) -> FeishuWebhook {
        let mut text_content = format!("🚨 **{}**\n\n", glitchtip.alias);

        // Add title from the first attachment
        if let Some(attachment) = glitchtip.attachments.first() {
            text_content.push_str(&format!("**错误**: {}\n\n", attachment.title));
        }

        // Add fields as key-value pairs
        for attachment in &glitchtip.attachments {
            for field in &attachment.fields {
                text_content.push_str(&format!("**{}**: {}\n", field.title, field.value));
            }
        }

        // Add link if available
        if let Some(attachment) = glitchtip.attachments.first() {
            if !attachment.title_link.is_empty() {
                text_content.push_str(&format!("\n🔗 [查看详情]({})", attachment.title_link));
            }
        }

        FeishuWebhook {
            msg_type: "text".to_string(),
            content: Some(FeishuContent {
                text: Some(text_content),
                post: None,
            }),
            card: None,
        }
    }

    pub fn glitchtip_to_feishu_rich_text(glitchtip: &GlitchTipSlackWebhook) -> FeishuWebhook {
        let mut content = Vec::new();

        // Add title
        if let Some(attachment) = glitchtip.attachments.first() {
            content.push(vec![
                FeishuPostElement::Text {
                    text: format!("🚨 **{}**\n\n", glitchtip.alias),
                },
                FeishuPostElement::Text {
                    text: format!("**错误**: {}\n\n", attachment.title),
                },
            ]);

            // Add fields
            let mut field_elements = Vec::new();
            for field in &attachment.fields {
                field_elements.push(FeishuPostElement::Text {
                    text: format!("**{}**: {}\n", field.title, field.value),
                });
            }

            // Add link
            if !attachment.title_link.is_empty() {
                field_elements.push(FeishuPostElement::Link {
                    text: "查看详情".to_string(),
                    href: attachment.title_link.clone(),
                });
            }

            if !field_elements.is_empty() {
                content.push(field_elements);
            }
        }

        FeishuWebhook {
            msg_type: "post".to_string(),
            content: Some(FeishuContent {
                text: None,
                post: Some(FeishuPost {
                    zh_cn: FeishuPostContent {
                        title: Some(format!("{} - 错误通知", glitchtip.alias)),
                        content,
                    },
                }),
            }),
            card: None,
        }
    }

    pub fn glitchtip_to_feishu_card(glitchtip: &GlitchTipSlackWebhook) -> FeishuWebhook {
        // Create a simple card format
        let card = serde_json::json!({
            "schema": "2.0",
            "body": {
                "type": "card",
                "header": {
                    "template": "red",
                    "title": {
                        "content": format!("{} - 错误通知", glitchtip.alias),
                        "tag": "plain_text"
                    }
                },
                "elements": []
            }
        });

        let mut elements = Vec::new();

        // Add error details
        if let Some(attachment) = glitchtip.attachments.first() {
            elements.push(serde_json::json!({
                "tag": "div",
                "text": {
                    "content": format!("**错误**: {}", attachment.title),
                    "tag": "lark_md"
                }
            }));

            // Add fields
            for field in &attachment.fields {
                elements.push(serde_json::json!({
                    "tag": "div",
                    "fields": [
                        {
                            "is_short": field.short,
                            "text": {
                                "content": format!("**{}**\n{}", field.title, field.value),
                                "tag": "lark_md"
                            }
                        }
                    ]
                }));
            }

            // Add action button for link
            if !attachment.title_link.is_empty() {
                elements.push(serde_json::json!({
                    "tag": "action",
                    "actions": [
                        {
                            "tag": "button",
                            "text": {
                                "content": "查看详情",
                                "tag": "plain_text"
                            },
                            "type": "primary",
                            "url": attachment.title_link
                        }
                    ]
                }));
            }
        }

        let mut card_with_elements = card;
        card_with_elements["body"]["elements"] = serde_json::Value::Array(elements);

        FeishuWebhook {
            msg_type: "interactive".to_string(),
            content: None,
            card: Some(card_with_elements),
        }
    }
}