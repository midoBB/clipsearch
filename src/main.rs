use anyhow::Ok;
use clap::{Parser, Subcommand};
use redb::{Database, ReadableTable, TableDefinition, WriteTransaction};
use std::{
    env,
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
};
use uuid::Uuid;

const MAX_ITEMS: u64 = 750; // TODO: make configurable
const MAX_DEDUPE_SEARCH: u64 = 100; // TODO: make configurable
const CLIPBOARD_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("clipboard");

#[derive(Parser)]
#[command(author = "midoBB")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A Wayland clipboard manager / search")]
#[command(name = "ClipSearch")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Store,
    Wipe,
    Version,
    List,
}

#[derive(Debug)]
enum SocketMessage {
    Added,
    Removed,
    Wiped,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let db_path = dirs::state_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get state dir"))?
        .join("ClipSearch")
        .join("clipboard.db");
    let socket_path = dirs::runtime_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get runtime dir"))?
        .join("ClipSearch.sock");
    match cli.command {
        Some(Commands::Store) => handle_store(db_path, socket_path),
        Some(Commands::List) => handle_list(db_path),
        Some(Commands::Wipe) => handle_wipe(db_path, socket_path),
        Some(Commands::Version) => handle_version(),
        None => print_usage(),
    }?;
    Ok(())
}

fn handle_list(db_path: PathBuf) -> Result<(), anyhow::Error> {
    let db = init_db(&db_path)?;
    let read_txn = db.begin_read()?;
    let read_table = read_txn.open_table(CLIPBOARD_TABLE)?;

    for result in read_table.iter()?.rev() {
        let (k, v) = result?;
        println!(
            "{}: {}",
            String::from_utf8_lossy(k.value()),
            String::from_utf8_lossy(v.value())
        );
    }
    Ok(())
}

fn print_usage() -> Result<(), anyhow::Error> {
    println!("Usage:");
    println!("  command [SUBCOMMAND]");
    println!();
    println!("Subcommands:");
    println!("  store           Store clipboard content");
    println!("  list            List clipboard history");
    println!("  wipe            Wipe clipboard history");
    println!("  version         Print version information");
    println!();
    println!("For more information, use --help with any subcommand.");
    Ok(())
}

fn handle_store(db_path: PathBuf, socket_path: PathBuf) -> Result<(), anyhow::Error> {
    let db = init_db(&db_path)?;
    let clipboard_state = env::var("CLIPBOARD_STATE").unwrap_or_default();
    match clipboard_state.as_str() {
        "sensitive" | "clear" => delete_last(&db, socket_path)?,
        _ => store(&db, socket_path)?,
    }
    Ok(())
}

fn store(db: &Database, socket_path: PathBuf) -> Result<(), anyhow::Error> {
    let mut input = io::stdin();
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    if buffer.len() > 25_000_000 {
        // NOTE: We don't want to store more than 25MB
        return Err(anyhow::anyhow!("Input too large"));
    }
    let trimmed = trim_space(&buffer);
    if trimmed.len() == 0 {
        return Ok(());
    }

    let id = Uuid::now_v7().to_string().as_bytes().to_owned();

    let write_txn = db.begin_write()?;
    {
        deduplicate(&write_txn, trimmed)?;
        let mut write_table = write_txn.open_table(CLIPBOARD_TABLE)?;
        write_table.insert(id.as_slice(), trimmed)?;
    }
    write_txn.commit()?;

    send_update(socket_path.clone(), SocketMessage::Added)?;

    trim_length(&db, socket_path)?;
    Ok(())
}

fn deduplicate(write_txn: &WriteTransaction, input: &[u8]) -> Result<(), anyhow::Error> {
    let mut write_table = write_txn.open_table(CLIPBOARD_TABLE)?;
    let mut to_delete = Vec::new();
    let mut seen = 0;
    for result in write_table.iter()? {
        let (k, v) = result?;
        if v.value() == input {
            to_delete.push(k.value().to_owned());
        }
        seen += 1;
        if seen >= MAX_DEDUPE_SEARCH {
            break;
        }
    }

    for key in to_delete {
        write_table.remove(key.as_slice())?;
    }

    Ok(())
}

fn trim_length(db: &Database, socket_path: PathBuf) -> Result<(), anyhow::Error> {
    let read_txn = db.begin_read()?;
    let read_table = read_txn.open_table(CLIPBOARD_TABLE)?;

    let mut to_delete = Vec::new();
    let mut seen = 0;

    for result in read_table.iter()?.rev() {
        let (k, _) = result?;
        if seen < MAX_ITEMS {
            seen += 1;
            continue;
        }
        to_delete.push(k.value().to_owned());
        seen += 1;
    }
    drop(read_table);
    read_txn.close()?;

    let write_txn = db.begin_write()?;
    {
        let mut write_table = write_txn.open_table(CLIPBOARD_TABLE)?;
        for key in to_delete {
            write_table.remove(key.as_slice())?;
        }
        send_update(socket_path, SocketMessage::Removed)?;
    }
    write_txn.commit()?;

    Ok(())
}

fn init_db(db_path: &PathBuf) -> Result<Database, anyhow::Error> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Database::open(db_path).or_else(|_| Database::create(db_path))?;
    Ok(db)
}

fn delete_last(db: &Database, socket_path: PathBuf) -> Result<(), anyhow::Error> {
    let write_txn = db.begin_write()?;
    {
        let mut write_table = write_txn.open_table(CLIPBOARD_TABLE)?;
        write_table.pop_last()?;
        send_update(socket_path, SocketMessage::Removed)?;
    }
    write_txn.commit()?;
    Ok(())
}

fn handle_wipe(db_path: PathBuf, socket_path: PathBuf) -> Result<(), anyhow::Error> {
    let db = init_db(&db_path)?;
    let write_txn = db.begin_write()?;
    {
        let mut write_table = write_txn.open_table(CLIPBOARD_TABLE)?;
        write_table.retain(|_, _| false)?;
        send_update(socket_path, SocketMessage::Wiped)?;
    }
    write_txn.commit()?;
    Ok(())
}

fn handle_version() -> Result<(), anyhow::Error> {
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

// is_space is taken from Golang's Byte isSpace function for latin1 characters
fn is_space(b: u8) -> bool {
    matches!(
        b,
        b'\t' | b'\n' | 0x0B | b'\x0C' | b'\r' | b' ' | 0x85 | 0xA0
    )
}

// trim_space is taken from Golang's bytes.TrimSpace function
fn trim_space(input: &[u8]) -> &[u8] {
    if input.is_empty() {
        return input;
    }

    let start = input
        .iter()
        .position(|&x| !is_space(x))
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|&x| !is_space(x))
        .map(|i| i + 1)
        .unwrap_or(0);

    &input[start..end]
}

fn send_update(socket_path: PathBuf, message: SocketMessage) -> Result<(), anyhow::Error> {
    if let Err(e) = UnixStream::connect(socket_path.as_os_str()).and_then(|mut socket| {
        // Try to write the message to the socket
        socket.write_all(format!("{:?}", message).as_bytes())?;
        // Flush the socket to ensure the data is sent
        socket.flush()
    }) {
        eprint!("Failed to send update to socket: {}", e);
    }
    Ok(())
}
