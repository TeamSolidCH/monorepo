/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */
use core::fmt;
#[macro_export]
macro_rules! timezone {
    ($($name:ident => $value:expr,)*) => {
        #[derive(Debug, poise::ChoiceParameter)]
        pub enum TimezoneChoices {
            $(
                #[name = $value]
                $name,
            )*
        }
        impl fmt::Display for TimezoneChoices {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        TimezoneChoices::$name => write!(f, $value),
                    )*
                }
            }
        }
        impl TimezoneChoices {
            pub fn to_normalized_string(&self) -> String {
                let re = regex::Regex::new(r"([A-Z][a-z]+)").expect("Unable to create regex pattern");
                let mut index = 0;
                let mut result = String::new();
                let name = match self {
                    $(
                        TimezoneChoices::$name => stringify!($name),
                    )*
                };
                for field in re.find_iter(name) {
                    if index == 1 {
                        result.push_str("/")
                    } else if index > 1 {
                        result.push_str("_")
                    }
                    result.push_str(&field.as_str());
                    index += 1;
                }
                result
            }
        }
    };
}

timezone!(
    EtcUtc => "UTC (+00:00)",
    AfricaLagos => "Africa/Lagos (+01:00)",
    AfricaCairo => "Africa/Cairo (+02:00/ +03:00)",
    AfricaCeuta => "Africa/Ceuta (+01:00 / +02:00)",
    AfricaElAaiun => "Africa/Casablanca (+01:00 / +00:00)",
    AfricaKhartoum => "Africa/Khartoum (+02:00)",
    AfricaNairobi => "Africa/Nairobi (+03:00)",
    AmericaAdak => "America/Adak (-10:00, -9:00)",
    AmericaAnchorage => "America/Anchorage (-09:00, -8:00)",
    AmericaAraguaina => "America/Araguaina (-03:00)",
    AmericaAsuncion => "America/Asuncion (-04:00 /-03:00)",
    AmericaBarbados => "America/Barbados (-04:00)",
    AmericaBahiaBanderas => "America/Bahia_Banderas (-06:00)",
    AmericaBogota => "America/Bogota (-05:00)",
    AmericaBoise => "America/Boise (-07:00 / -06:00)",
    EuropeZurich => "Europe/Zurich (+01:00 / +02:00)",
    EuropeBucharest => "Europe/Bucharest (+02:00 / +03:00)",
);
