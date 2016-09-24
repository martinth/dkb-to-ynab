extern crate csv;
extern crate rustc_serialize;
extern crate clap;
extern crate chrono;

use clap::App;
use csv::Reader;
use csv::Writer;

static DEBIT_MARKER: &'static str = "Kontonummer:";
static CREDIT_MARKER: &'static str = "Kreditkarte:";
static OUTPUT_HEADER: &'static [ &'static str ] = &["Date", "Payee", "Category", "Memo", "Outflow", "Inflow"];


#[derive(RustcDecodable, Debug)]
struct TypeHeader {
    kind: String,
    description: String,
}

fn convert_dt_format(input: &str) -> String {
    chrono::NaiveDate::parse_from_str(input, "%d.%m.%Y")
                    .map(|dt| dt.format("%Y/%m/%d").to_string())
                    .unwrap_or(String::new())
}

fn convert_number_format(input: &str) -> Result<f64, RecordError> {
    input.chars()
        .filter(|c| *c == '-' || c.is_digit(10))
        .collect::<String>()
        .parse::<i32>()
        .map_err(RecordError::AmountFormat)
        .map(|cents| (cents as f64) / 100.0)
}

struct OutAndInflow {
    outflow: String,
    inflow: String,
}
impl From<Option<f64>> for OutAndInflow {
    /// Takes a signed number and converts it to the OutAndInflow struct. 
    ///
    /// If the option is empty, a struct with empty strings will be returned,
    fn from(signed_amount: Option<f64>) -> OutAndInflow {
        signed_amount.map_or(OutAndInflow { 
            outflow: String::new(),
            inflow: String::new()
        }, |signed_amount| {
            if signed_amount < 0.0 {
                OutAndInflow {
                    outflow: format!("{0:.2}", signed_amount * -1.0),
                    inflow: String::new()
                }
            } else {
                OutAndInflow {
                    outflow: String::new(),
                    inflow: format!("{0:.2}", signed_amount),
                }
            }
        }) 
    }
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

// See: http://classic.youneedabudget.com/support/article/csv-file-importing
// Date,Payee,Category,Memo,Outflow,Inflow
// 01/25/12,Sample Payee,,Sample Memo for an outflow,100.00,
// 01/26/12,Sample Payee 2,,Sample memo for an inflow,,500.00
#[derive(RustcEncodable, Debug)]
struct OutputRecord {
    date: String,
    payee: String,
    category: String,
    memo: String,
    outflow: String,
    inflow: String,
}
impl From<CreditRecord> for OutputRecord {
    fn from(r: CreditRecord) -> OutputRecord {
        let out_and_inflow = OutAndInflow::from(convert_number_format(&r.betrag).ok());

        OutputRecord {
            date: convert_dt_format(&r.wertstellung),
            payee: String::new(),
            category: String::new(),
            memo: r.beschreibung,
            outflow: out_and_inflow.outflow,
            inflow: out_and_inflow.inflow,
        }
    }
}
impl From<DebitRecord> for OutputRecord {
    fn from(r: DebitRecord) -> OutputRecord {
        let out_and_inflow = OutAndInflow::from(convert_number_format(&r.betrag).ok());

        OutputRecord {
            date: convert_dt_format(&r.wertstellung),
            payee: r.auftraggeber,
            category: String::new(),
            memo: r.verwendungszweck,
            outflow: out_and_inflow.outflow,
            inflow: out_and_inflow.inflow,
        }
    }
}

#[derive(Debug)]
enum InputError {
    Csv(csv::Error),
    UnknownFileType,
}

#[derive(Debug)]
enum OutputError {
    Csv(csv::Error),
}

#[derive(Debug)]
enum RecordError {
    AmountFormat(std::num::ParseIntError)
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

fn write_output_csv(data: Vec<OutputRecord>, file_name: &str) -> Result<(), OutputError> {
    let mut writer = try!(Writer::from_file(file_name).map_err(OutputError::Csv));
    writer.write(OUTPUT_HEADER.into_iter());

    for record in data {
        let result = writer.encode(record);
        assert!(result.is_ok());
    }
    Ok(())
    //println!("{:?}", wtr.as_string());
}

fn main() {

    let options = App::new("dkb_to_ynab")
                          .version("0.1")
                          .author("Martin Thurau <martin.thurau@gmail.com>")
                          .about("Converts DKB CSV files to CSV that YNAB can understand")
                          .args_from_usage(
                              "<INPUT>              'Sets the input file'
                              <OUTPUT>              'Sets the output file'")
                          .get_matches();

    match read_input_csv(options.value_of("INPUT").unwrap()) {
        Ok(lines) => {
            write_output_csv(lines, options.value_of("OUTPUT").unwrap());
        },
        Err(e) => println!("{:?}", e)
    }
}
