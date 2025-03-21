use crate::core::Core;
use anyhow::Result;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, EditView, LinearLayout, Panel, ResizedView, TextContent, TextView,
};
use cursive::Cursive;
use std::sync::{Arc, Mutex};
use tracing::*;

#[derive(Clone, Copy)]
enum Unit {
    Btc,
    Sats,
}

fn convert_amount(amount: f64, from: Unit, to: Unit) -> f64 {
    match (from, to) {
        (Unit::Btc, Unit::Sats) => amount * 100_000_000.0,
        (Unit::Sats, Unit::Btc) => amount / 100_000_000.0,
        _ => amount,
    }
}

pub fn run_ui(core: Arc<Core>, balance_content: TextContent) -> Result<()> {
    let mut siv = cursive::default();
    setup_siv(&mut siv, core.clone(), balance_content);
    info!("Starting UI event loop");
    siv.run();
    info!("Ui event loop ended");
    Ok(())
}

fn setup_siv(siv: &mut Cursive, core: Arc<Core>, balance_content: TextContent) {
    siv.set_autorefresh(true);
    siv.set_window_title("BTC Wallet".to_string());
    siv.add_global_callback('q', |s| {
        info!("Quit command received");
        s.quit()
    });
    setup_menubar(siv, core.clone());
    setup_layout(siv, core, balance_content);
    siv.add_global_callback(Event::Key(Key::Esc), |siv| siv.select_menubar());
    siv.select_menubar()
}

fn setup_menubar(siv: &mut Cursive, core: Arc<Core>) {
    siv.menubar()
        .add_leaf("Send", move |s| {
            show_send_transaction(s, core.clone());
        })
        .add_leaf("Quit", |s| s.quit());
    siv.set_autohide_menu(false)
}

fn setup_layout(siv: &mut Cursive, core: Arc<Core>, balance_content: TextContent) {
    let instruction = TextView::new("Press escape to select the top menu");
    let balance_panel = Panel::new(TextView::new_with_content(balance_content)).title("Balance");
    let info_layout = create_info_layout(&core);
    let layout = LinearLayout::vertical()
        .child(instruction)
        .child(balance_panel)
        .child(info_layout);
    siv.add_layer(layout);
}

fn create_info_layout(core: &Arc<Core>) -> LinearLayout {
    let mut info_layout = LinearLayout::horizontal();
    let keys_content = core
        .config
        .my_keys
        .iter()
        .map(|key| format!("{}", key.private.display()))
        .collect::<Vec<String>>()
        .join("\n");
    info_layout.add_child(ResizedView::with_full_width(
        Panel::new(TextView::new(keys_content)).title("Your keys"),
    ));
    let contacts_content = core
        .config
        .contacts
        .iter()
        .map(|contact| contact.name.clone())
        .collect::<Vec<String>>()
        .join("\n");
    info_layout.add_child(ResizedView::with_full_width(
        Panel::new(TextView::new(contacts_content)).title("Contacts"),
    ));
    info_layout
}

fn show_send_transaction(s: &mut Cursive, core: Arc<Core>) {
    info!("Showing send transaction dialog");
    let unit = Arc::new(Mutex::new(Unit::Btc));
    s.add_layer(
        Dialog::around(create_transaction_layout(unit.clone()))
            .title("Send Transactiomn")
            .button("Send", move |siv| {
                send_transaction(siv, core.clone(), *unit.lock().unwrap())
            })
            .button("Cancel", |siv| {
                debug!("Transaction cancelled");
                siv.pop_layer();
            }),
    );
}

fn create_transaction_layout(unit: Arc<Mutex<Unit>>) -> LinearLayout {
    LinearLayout::vertical()
        .child(TextView::new("Recipient:"))
        .child(EditView::new().with_name("recipient"))
        .child(TextView::new("Amount:"))
        .child(EditView::new().with_name("amount"))
        .child(create_unit_layout(unit))
}

fn create_unit_layout(unit: Arc<Mutex<Unit>>) -> LinearLayout {
    LinearLayout::horizontal()
        .child(TextView::new("Unit: "))
        .child(TextView::new_with_content(TextContent::new("BTC")).with_name("unit_display"))
        .child(Button::new("Switch", move |s| {
            switch_unit(s, unit.clone());
        }))
}

fn switch_unit(s: &mut Cursive, unit: Arc<Mutex<Unit>>) {
    let mut unit = unit.lock().unwrap();
    *unit = match *unit {
        Unit::Btc => Unit::Sats,
        Unit::Sats => Unit::Btc,
    };
    s.call_on_name("unit_display", |view: &mut TextView| {
        view.set_content(match *unit {
            Unit::Btc => "BTC",
            Unit::Sats => "Sats",
        })
    });
}

fn send_transaction(s: &mut Cursive, core: Arc<Core>, unit: Unit) {
    debug!("Send button pressed");
    let recipient = s
        .call_on_name("recipient", |view: &mut EditView| view.get_content())
        .unwrap();

    let amount: f64 = s
        .call_on_name("amount", |view: &mut EditView| view.get_content())
        .unwrap()
        .parse()
        .unwrap_or(0.0);
    let amount_sats = convert_amount(amount, unit, Unit::Sats) as u64;
    info!(
        "Attempting to send transaction to {} for {} satoshis",
        recipient, amount_sats
    );
    match core.send_transaction_async(recipient.as_str(), amount_sats) {
        Ok(_) => show_success_dialog(s),
        Err(e) => show_error_dialog(s, e),
    }
}

fn show_success_dialog(s: &mut Cursive) {
    info!("Transaction sent successfully");
    s.add_layer(
        Dialog::text("Transaction sent successfull")
            .title("Success")
            .button("Ok", |s| {
                debug!("Closing success dialog");
                s.pop_layer();
                s.pop_layer();
            }),
    );
}

fn show_error_dialog(s: &mut Cursive, error: impl std::fmt::Display) {
    error!("Failed to send transaction {}", error);

    s.add_layer(
        Dialog::text(format!("Failed to send transaction: {}", error))
            .title("Error")
            .button("Ok", |s| {
                debug!("Closing error dialog");
                s.pop_layer();
            }),
    );
}
