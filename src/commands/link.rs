use colored::Colorize;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
    print::format_two_tokens,
    skim::open::Iteration,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;

    let multi = false;

    let from = Iteration::new(
        "link from".to_string(),
        list.clone(),
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
    )
    .run()
    .await?;

    link(from, db, &external_commands, &surf_parsing, md_static).await?;

    Ok("success".cyan().to_string())
}

pub(crate) async fn link(
    from: Note,
    db: SqliteAsyncHandle,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    md_static: MarkdownStatic,
) -> Result<(), anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;
    let to = Iteration::new(
        "link_to".to_string(),
        list,
        db.clone(),
        false,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
    )
    .run()
    .await?;

    db.lock()
        .await
        .insert_link(&from.name(), &to.name())
        .await?;
    eprintln!(
        "{}",
        format_two_tokens(
            "linked: ",
            &format!("\"{}\" -> \"{}\"", from.name(), to.name())
        )
    );
    Ok(())
}
