//! https://www.rust-lang.org/
//! spouštět přes:
//! ```
//! cargo run --release
//! ```
//! jinak se bude pouštět debug build a to je výrazně pomalejší
extern crate csv; // na parsování CSV
extern crate chrono; // čas
extern crate serde; // serializace/deserializace
extern crate serde_derive; // makra pro ^
extern crate promptly; // cli prompt

use promptly::*;
use chrono::prelude::*;
use serde_derive::Deserialize;

use csv::ReaderBuilder;

use std::fs::File;
use std::process::exit;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
struct EntryFujTime {
	zakaznik: u32, // předpokládáme, že zákazníci mají pouze kladná celá čísla
	program: String,
	cas: String, // csv neumí parsovat do DateTime<Local>
	trvani: u32,
}

#[derive(Clone, Debug)]
struct Entry {
	zakaznik: u32, // předpokládáme, že zákazníci mají pouze kladná celá čísla
	program: String,
	cas: DateTime<Local>,
	trvani: u32,
}

// funkce pro správné zabití když se něco nepovede
fn error(msg: &str) -> ! {
	eprintln!("{}", msg);
	exit(-1)
}

impl From<EntryFujTime> for Entry {
	fn from(src: EntryFujTime) -> Entry {
		Entry {
			zakaznik: src.zakaznik,
			program: src.program,
			trvani: src.trvani,
			cas: Local
				.datetime_from_str(&src.cas, "%Y-%m-%d %H:%M:%S")
				.unwrap_or_else(|_| error("záznam má čas ve špatném formátu")),
		}
	}
}


fn main() {
	let nazev: String = prompt("zadejte název souboru");
	let log = File::open(nazev)
		.unwrap_or_else(|_| error("nepodařilo se otevřít soubor"));

	let mesic = loop {
		match prompt::<String, &'static str>("zadejte měsíc (kveten 2018 - unor 2019)")
			.to_string()
			.to_lowercase()
			.as_ref()
		{
			"kveten 2018" => break 5,
			"cerven 2018" => break 6,
			"cervenec 2018" => break 7,
			"srpen 2018" => break 8,
			"zari 2018" => break 9,
			"rijen 2018" => break 10,
			"listopad 2018" => break 11,
			"prosinec 2018" => break 12,
			"leden 2019" => break 1,
			"unor 2019" => break 2,
			_ => ()
		}
	};

	let mut log: Box<Vec<Entry>> = Box::new(
		ReaderBuilder::new()
			.delimiter(b';')
			.has_headers(false)
			.from_reader(log)
			.deserialize::<EntryFujTime>()
			//.inspect(|x| println!("{:?}", x))
			.filter(|x| x.is_ok())
			.map(|x| x.unwrap())
			.map(|x| Entry::from(x))
			.collect()
	); // -> Entry

	log.sort_unstable_by(|a: &Entry, b: &Entry| a.cas.cmp(&b.cas));

	log.retain(|x| x.cas.month() == mesic);

	let op = loop {
		match prompt::<String, &'static str>("vyberte si statistiku [\n\t'casy' (součet prosledovaných časů pro jednotlivé kanály),\n\t'divaci' (počet unikátních diváků pro jednotlivé kanály),\n\t'zatez' (maximální zátěž serveru)\n]")
			.to_string()
			.to_lowercase()
			.as_ref()
		{
			m @ "casy" |
			m @ "divaci" |
			m @ "zatez" => break m.to_string(),
			_ => (),
		}
	};

	match op.as_ref() {
		"casy" => {
			let mut log = log.iter()
				.fold(HashMap::new(), |mut acc, x| {
					*acc.entry(&x.program).or_insert(0) += x.trvani;
					acc
				})
				.iter()
				.map(|(program, trvani)| ((*program).clone(), chrono::Duration::seconds(*trvani as i64)))
				.collect::<Vec<(String, chrono::Duration)>>();

			// řazení podle trvání
			log.sort_by(|a, b| b.1.cmp(&a.1));

			log
				.iter()
				.for_each(|(program, cas)| println!("{}: {}.{}:{}:{}", program,
					cas.num_days(),
					(cas.num_hours() - (cas.num_days() * 24)),
					(cas.num_minutes() - (cas.num_hours() * 60)),
					(cas.num_seconds() - (cas.num_minutes() * 60))
				));
		},
		"divaci" => {
			log.dedup_by(|a, b| (&a.program, &a.zakaznik) == (&b.program, &b.zakaznik));

			let mut log = log.iter()
				.fold(HashMap::new(), |mut acc, x| {
					*acc.entry(&x.program).or_insert(0) += 1;
					acc
				})
				.iter()
				.map(|(n, p)| ((*n).clone(), *p))
				.collect::<Vec<(String, u32)>>();

			// řazení podle počtu
			log.sort_by(|a, b| b.1.cmp(&a.1));

			log
				.iter()
				.for_each(|(program, pocet)| println!("{}: {}", program, pocet));
		},
		"zatez" => {
			let mut log = log
				.iter()
				.fold(HashMap::new(), |mut acc, x| {
					*acc.entry((x.cas.timestamp() / 60) / 5).or_insert(0) += 1;
					acc
				})
				.iter()
				.map(|(a, b)| (*a, *b))
				.collect::<Vec<(i64, u32)>>();

			log.sort_by(|a, b| b.1.cmp(&a.1));

			log
				.iter()
				.take(20)
				.for_each(|(cas, p)| println!("{}: {}", Local.timestamp(cas * 60 * 5, 0), p));
		}
		_ => unreachable!()
	}
}
