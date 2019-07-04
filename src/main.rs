/*
NAME

    remind -- print reminders of upcoming events

USAGE

    remind -- show reminders for next seven days
    remind [year] month day message -- add reminder to database

DESCRIPTION

    Remind maintains a database of reminders in the .reminders file,
    in the user's home directory, each a single line of the form

        [year] month day message

    Year is optional, and must be an integer greater than 99; if no
    year is given, the reminder applies to all years (for instance,
    birthdays).

    If remind is called with no arguments, it writes to standard
    output all reminders that occur within the next seven days. If
    remind is called with arguments giving a date and message, a
    reminder is added to the database. Any time remind is called,
    all past reminders are deleted from the database.

EXAMPLE

    $ date
    Sun Jun 30 19:45:38 CDT 2019
    $ remind 4 2 Anne birthday
    $ remind 10 13 Kate birthday
    $ remind 7 4 Independence Day
    $ remind 2019 7 2 lunch with Pat
    $ remind 2019 5 13 dentist 2:00pm
    $ remind
    7 4 Independence Day
    2019 7 2 lunch with Pat
    $ cat ./reminders
    4 2 Anne birthday
    10 13 Kate birthday
    7 4 Independence Day
    2019 7 2 lunch with Pat
*/
use itertools::Itertools;
use chrono::prelude::*;

fn main() -> Result<(), String> {
    let mut r = Reminders::new(".reminders")?;
    let args = std::env::args().skip(1);
    if args.len() == 0 {
        print!("{}", r.stringify(7));
    } else {
        r.add(r.parse_item(args)?);
    }
    r.close()
}

#[derive(Debug)]
struct Reminders {
    path: std::path::PathBuf,
    today: NaiveDate,
    reminder_items: Vec<ReminderItem>,
}

#[derive(Debug)]
struct ReminderItem {
    date: NaiveDate,
    has_year: bool,
    message: String,
}

impl Reminders {
    fn new(path_str: &str) -> Result<Self, String> {
        let mut path = match dirs::home_dir() {
            Some(dir) => dir,
            None => return Err("could not find home directory!".to_string())
        };
        path.push(path_str);
        let today = Local::today().naive_local();
        let mut reminder = Reminders { path, today, reminder_items: vec!() };
        if let Ok(data) = std::fs::read_to_string(&reminder.path) {
            for line in data.split("\n").filter(|&l| l != "") {
                reminder.add(reminder.parse_item(line.split(" ").collect::<Vec<_>>().into_iter())?);
            }
        }
        Ok(reminder)
    }
    fn add(&mut self, item: ReminderItem) {
        self.reminder_items.push(item);
    }
    fn stringify(&self, ndays: i32) -> String {
        let days_ce = self.today.num_days_from_ce();
        self.reminder_items.iter()
            .filter(|item|
                item.date.num_days_from_ce() >= days_ce &&
                (ndays == 0 || item.date.num_days_from_ce() < (days_ce + ndays)))
            .map(|i| i.to_string() + "\n")
            .join("")
    }
    fn close(self) -> Result<(), String> {
        match std::fs::write(&self.path, self.stringify(0)) {
            Err(m) => Err(format!("could not write reminders to {}: {}", self.path.display(), m)),
            _ => Ok(())
        }
    }
    fn parse_item<I, T>(&self, mut args: I) -> Result<ReminderItem, String>
    where I: Iterator<Item=T> + ExactSizeIterator,
        T: std::fmt::Display,
    {
        let usage = Err("usage: remind [year] month day message".to_string());
        let mut arg = args.next();
        let year = match &arg {
            Some(year) => {
                match year.to_string().parse::<i32>() {
                    Ok(year) if year > 99 => {
                        arg = args.next();
                        Some(year)
                    }
                    Ok(_) => None,
                    _ => return usage
                }
            }
            None => return usage
        };
        if args.len() < 2 {
            return usage;
        }
        let month = match arg.unwrap().to_string().parse::<u32>() {
            Ok(month) => month,
            _ => return usage
        };
        let day = match args.next().unwrap().to_string().parse::<u32>() {
            Ok(day) => day,
            _ => return usage
        };

        let date = if let Some(year) = year {
            NaiveDate::from_ymd_opt(year, month, day)
        } else {
            self.next_recurring_date(month, day)
        };
        if let Some(date) = date {
            Ok(ReminderItem{ date, has_year: year.is_some(), message: args.join(" ") })
        } else {
            usage
        }
    }
    fn next_recurring_date(&self, month: u32, day: u32) -> Option<NaiveDate> {
        let mut year = self.today.year();
        if month == 2 && day == 29 {
            loop {
                if let Some(date) = NaiveDate::from_ymd_opt(year, 2, 29) {
                    if date.num_days_from_ce() >= self.today.num_days_from_ce() {
                        break Some(date);
                    }
                }
                year += 1;
            }
        } else if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date.num_days_from_ce() >= self.today.num_days_from_ce() {
                Some(date)
            } else {
                NaiveDate::from_ymd_opt(year + 1, month, day)
            }
        } else {
            None
        }
    }
}

impl std::fmt::Display for ReminderItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.has_year {
            write!(f, "{} ", self.date.year())?;
        }
        write!(f, "{} {} {}", self.date.month(), self.date.day(), self.message)
    }
}