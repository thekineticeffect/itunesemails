extern crate mailparse;
extern crate scraper;
extern crate currency;
extern crate csv;

use std::io;
use std::io::{BufReader, Read, Error};
use std::fs;
use std::fs::{DirEntry, File};
use std::path::{Path};
use scraper::{Html, Selector};
use currency::Currency;

fn main() {
    process_folder("./mails");
}

fn process_folder(dir: &str) {
    let xxx = files_in_folder(dir).map_err(|error|
        println!("Had error: {}", error)
    ).unwrap();

    let mut purchases = Vec::new();
    for path in xxx {
        let file_purchases = process_file(path.path().as_path());
        if let Some(mut file_purchases) = file_purchases {
            purchases.append(&mut file_purchases);
        }
    }
    write_purchases("out.csv", &purchases);
    println!("Found {} purchases.", purchases.len());
}

struct Purchase {
    name: String,
    purchaser: String,
    price: Currency,
}

fn write_purchases(destination: &str, purchases: &Vec<Purchase>) {
    let file = File::create(destination).unwrap();
    let mut wtr = csv::Writer::from_writer(file);

    // When writing records without Serde, the header record is written just
    // like any other record.
    wtr.write_record(&["Item", "Purchaser", "Price"]).unwrap();
    for purchase in purchases {
        wtr.write_record(&[String::clone(&purchase.name), String::clone(&purchase.purchaser), purchase.price.to_string()]).unwrap();
    }
    wtr.flush().unwrap();
}

fn process_file(filename: &Path) -> Option<Vec<Purchase>> {
    let contents = read_file(&filename);
    if contents.is_err() {
        return None;
    }
    let contents = contents.unwrap();

    let parsed = mailparse::parse_mail(contents.as_slice());
    if parsed.is_err() {
        return None;
    }
    let parsed = parsed.unwrap();
//    println!("Parsed email: {:?}", parsed.headers.get_first_value("Subject").unwrap());

    let body = parsed.subparts[1].get_body().unwrap();
    extract_purchases_from_html(&body[..])
}

fn extract_purchases_from_html(html: &str) -> Option<Vec<Purchase>> {
    let mut purchases = Vec::new();
    let document = Html::parse_document(html);

    let td_sel = Selector::parse("td").unwrap();
    let mut id_rows = document.select(&td_sel)
        .filter(|elem| elem.text().next().unwrap_or("") == "APPLE ID");
    let purchaser = id_rows.next().unwrap().last_child().unwrap().value().as_text().unwrap().to_string();

    let tbl_sel = Selector::parse(".aapl-mobile-tbl").unwrap();
    let row_sel = Selector::parse("tr").unwrap();
    let name_sel = Selector::parse(".title").unwrap();
    let price_sel = Selector::parse(".price-cell").unwrap();

    let table = document.select(&tbl_sel).next().unwrap();
    let rows = table.select(&row_sel)
        .filter(|element| element.value().attr("style").unwrap_or("") == "max-height:114px;");
    for element in rows {
        let name = element.select(&name_sel).next().unwrap().text().next().unwrap();
        let price = element.select(&price_sel).next().unwrap().text().filter(|text| text.contains("$")).next().unwrap();
//        println!("Element: {} - {}", name, price);
        let price = Currency::from_str(price).unwrap_or(Currency::new());
        let purchase = Purchase{name: String::from(name), purchaser: String::clone(&purchaser), price};
        purchases.push(purchase);
    }
    Some(purchases)
}

fn read_file(filename: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);

    let mut contents = Vec::new();
    let _read_bytes = buf_reader.read_to_end(&mut contents);
    Ok(contents)
}

fn files_in_folder(dir: &str) -> io::Result<impl Iterator<Item = DirEntry>> {
    let dir = fs::read_dir(dir);
    match dir {
        Ok(dir) => Ok(dir
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.metadata().is_ok())
            .filter_map(|entry|
                if entry.metadata().unwrap().is_file() {
                    Some(entry)
                } else {
                    None
                })),
        Err(err) => Err(err),
    }
}
