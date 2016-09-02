extern crate csv;
extern crate rustc_serialize;

use std::env;
use csv::Reader;

static DEBIT_MARKER: &'static str = "Kontonummer:";
static CREDIT_MARKER: &'static str = "Kreditkarte:";

#[derive(RustcDecodable, Debug)]
struct TypeHeader {
    kind: String,
    description: String,
}

// "Buchungstag";"Wertstellung";"Buchungstext";"Auftraggeber / Beg�nstigter";"Verwendungszweck";"Kontonummer";"BLZ";"Betrag (EUR)";"Gl�ubiger-ID";"Mandatsreferenz";"Kundenreferenz";
// "02.09.2016";"02.09.2016";"Lastschrift";"Stromio GmbH";"ABSCHLAG Strom 09/16 VK: 123456789 gruenstrom easy12";"DE75xxxx";"AARBDE5WDOM";"-95,00";"DE95xxxx    ";"xxxxx-01-1        ";"";
#[derive(RustcDecodable, Debug)]
struct DebitRecord {
    buchungstag: String,
    wertstellung: String,
    buchungstext: String,
    auftraggeber: String,
    verwendungszweck: String,
    kontonummer: String,
    blz: String,
    betrag: String,
    glaeubiger_id: String,
    mandatsref: String,
    kundenreg: String,
}

// "Umsatz abgerechnet";"Wertstellung";"Belegdatum";"Beschreibung";"Betrag (EUR)";"Urspr�nglicher Betrag";
// "Nein";"30.08.2016";"29.08.2016";"NETFLIX.COM866-579-7172";"-9,99";"";
#[derive(RustcDecodable, Debug)]
struct CreditRecord {
    abgerechnet: String,
    wertstellung: String,
    belegdatum: String,
    beschreibung: String,
    betrag: String,
    original_betrag: String,
}

#[derive(Debug)]
struct OutputRecord {
    betrag: String,
}
impl From<CreditRecord> for OutputRecord {
    fn from(r: CreditRecord) -> OutputRecord {
        OutputRecord {
            betrag: r.betrag
        }
    }
}
impl From<DebitRecord> for OutputRecord {
    fn from(r: DebitRecord) -> OutputRecord {
        OutputRecord {
            betrag: r.betrag
        }
    }
}

#[derive(Debug)]
enum InputError {
    Csv(csv::Error),
    UnknownFileType,
}


fn read_input_csv(file_name: &str) -> Result<Vec<OutputRecord>, InputError> {

    let mut header_reader = try!(Reader::from_file(file_name).map_err(InputError::Csv)).has_headers(false).delimiter(';' as u8);
    let header = header_reader.decode::<TypeHeader>().next().unwrap().unwrap();  // TODO: catch errors

    let mut data_reader = Reader::from_file(file_name).unwrap().delimiter(';' as u8).flexible(true);

    // TODO: can I factor out common code?
    if header.kind == DEBIT_MARKER {
        Ok(data_reader.decode::<DebitRecord>().skip_while(Result::is_err).skip(1).map(|r| {
            OutputRecord::from(r.unwrap())
        }).collect())
    } else if header.kind == CREDIT_MARKER {
        Ok(data_reader.decode::<CreditRecord>().skip_while(Result::is_err).skip(1).map(|r| {
            OutputRecord::from(r.unwrap())
        }).collect())
    } else {
        Err(InputError::UnknownFileType)
    }
    
}

fn main() {

    if let Some(input_file) = env::args().nth(1) {
        match read_input_csv(&input_file) {
            Ok(lines) => {
                for line in lines {
                    println!("{:?}", line);
                }
            },
            Err(e) => println!("{:?}", e)
        }
       
    }
}
