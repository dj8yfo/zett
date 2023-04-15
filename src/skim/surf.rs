use std::{fmt::Display, sync::Arc};

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{color::ColorScheme, ExternalCommands},
    highlight::MarkdownStatic,
    link::Link,
    note::Note,
};

#[derive(Debug)]
pub enum Action {
    Jump(Link),
    Open(Link),
    Return(Note),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Jump(link) => write!(f, "{} : {}", "jump", link),
            Self::Open(link) => write!(f, "{} : {}", "open", link),
            Self::Return(note) => write!(f, "{} : {}", "return to explore", note),
        }
    }
}
pub(crate) struct Iteration {
    items: Option<Vec<Link>>,
    multi: bool,
    external_commands: ExternalCommands,
    return_note: Note,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
}
impl Iteration {
    pub(crate) fn new(
        items: Vec<Link>,
        multi: bool,
        external_commands: ExternalCommands,
        return_note: Note,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Self {
        Self {
            items: Some(items),
            multi,
            external_commands,
            return_note,
            md_static,
            color_scheme,
        }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Action> {
        let items = self.items.take().unwrap();

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        for mut link in items {
            let tx_double = tx.clone();
            let ext_cmds_double = self.external_commands.clone();
            tokio::task::spawn(async move {
                link.prepare_display();
                link.prepare_preview(&ext_cmds_double.preview, self.md_static, self.color_scheme);
                let _result = tx_double.send(Arc::new(link));
                // if result.is_err() {
                //     eprintln!("{}", format!("{:?}", result).red());
                // }
            });
        }

        drop(tx);

        let out = tokio::task::spawn_blocking(move || {
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview(Some(""))
                .prompt(Some("(surf) > "))
                .preview_window(Some("up:50%"))
                .multi(self.multi)
                .bind(vec![
                    "ctrl-j:accept",
                    "ctrl-c:abort",
                    "ctrl-e:abort",
                    "Enter:accept",
                    "ESC:abort",
                ])
                .build()
                .unwrap();
            Skim::run_with(&options, Some(rx))
        })
        .await
        .unwrap();

        if let Some(out) = out {
            let selected_items = out
                .selected_items
                .iter()
                .map(|selected_item| {
                    (**selected_item)
                        .as_any()
                        .downcast_ref::<Link>()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<Link>>();

            match out.final_key {
                Key::Enter => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Action::Open(item.clone()));
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                Key::Ctrl('j') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Action::Jump(item.clone()));
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('e') => {
                    return Ok(Action::Return(self.return_note));
                }
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of surf cycle"
                    ))
                }
                _ => {
                    unreachable!();
                }
            };
        } else {
            return Err(anyhow::anyhow!("skim internal errors"));
        }
    }
}
