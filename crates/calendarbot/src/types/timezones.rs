use core::fmt;

#[derive(Debug, poise::ChoiceParameter)]
pub enum TimezoneChoices {
    #[name = "UTC (+00:00)"]
    #[name = "Africa/Abidjan (+00:00)"]
    #[name = "Africa/Bissau (+00:00)"]
    #[name = "Africa/Monrovia (+00:00)"]
    #[name = "Africa/Sao_Tome (+00:00)"]
    EtcUtc,
    #[name = "Africa/Lagos (+01:00)"]
    #[name = "Africa/Tunis (+01:00)"]
    #[name = "Africa/Algiers (+01:00)"]
    #[name = "Africa/Ndjamena (+01:00)"]
    AfricaLagos,
    #[name = "Africa/Cairo (+02:00/ +03:00)"]
    AfricaCairo,
    #[name = "Africa/Ceuta (+01:00 / +02:00)"]
    AfricaCeuta,
    #[name = "Africa/Casablanca (+01:00 / +00:00)"]
    #[name = "Africa/El_Aaiun (+01:00 / +00:00)"]
    AfricaElAaiun,
    #[name = "Africa/Johannesburg (+02:00)"]
    #[name = "Africa/Windhoek (+02:00)"]
    #[name = "Africa/Juba (+02:00)"]
    #[name = "Africa/Maputo (+02:00)"]
    #[name = "Africa/Khartoum (+02:00)"]
    AfricaKhartoum,
    #[name = "Africa/Nairobi (+03:00)"]
    AfricaNairobi,
    #[name = "America/Adak (-10:00, -9:00)"]
    AmericaAdak,
    #[name = "America/Anchorage (-09:00, -8:00)"]
    AmericaAnchorage,
    #[name = "America/Araguaina (-03:00)"]
    #[name = "America/Buenos_Aires (-03:00)"]
    #[name = "America/Catamarca (-03:00)"]
    #[name = "America/Cordoba (-03:00)"]
    #[name = "America/Jujuy (-03:00)"]
    #[name = "America/La_Rioja (-03:00)"]
    #[name = "America/Mendoza (-03:00)"]
    #[name = "America/Rio_Gallegos (-03:00)"]
    #[name = "America/Salta (-03:00)"]
    #[name = "America/San_Juan (-03:00)"]
    #[name = "America/San_Luis (-03:00)"]
    #[name = "America/Tucuman (-03:00)"]
    #[name = "America/Ushuaia (-03:00)"]
    #[name = "America/Bahia (-03:00)"]
    #[name = "America/Belem (-03:00)"]
    AmericaAraguaina,
    #[name = "America/Asuncion (-04:00 /-03:00)"]
    AmericaAsuncion,
    #[name = "America/Barbados (-04:00)"]
    #[name = "America/Boa_Vista (-04:00)"]
    AmericaBarbados,
    #[name = "America/Bahia_Banderas (-06:00)"]
    #[name = "America/Belize (-06:00)"]
    AmericaBahiaBanderas,
    #[name = "America/Bogota (-05:00)"]
    AmericaBogota,
    #[name = "America/Boise (-07:00 / -06:00)"]
    AmericaBoise,
    #[name = "Europe/Belgrade (+01:00 / +2:00)"]
    #[name = "Europe/Berlin (+01:00 / +02:00)"]
    #[name = "Europe/Paris (+01:00 / +02:00)"]
    #[name = "Europe/Zurich (+01:00 / +02:00)"]
    #[name = "Europe/Budapest (+01:00 / +02:00)"]
    EuropeZurich,
    #[name = "Europe/Bucharest (+02:00 / +03:00)"]
    #[name = "Europe/Chisinau (+02:00 / +03:00)"]
    EuropeBucharest,
}

impl fmt::Display for TimezoneChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimezoneChoices::EtcUtc => write!(f, "UTC (+00:00)"),
            TimezoneChoices::AfricaLagos => write!(f, "Africa/Lagos (+01:00)"),
            TimezoneChoices::AfricaCairo => write!(f, "Africa/Cairo (+02:00/ +03:00)"),
            TimezoneChoices::AfricaCeuta => write!(f, "Africa/Ceuta (+01:00 / +02:00)"),
            TimezoneChoices::AfricaElAaiun => write!(f, "Africa/Casablanca (+01:00 / +00:00)"),
            TimezoneChoices::AfricaKhartoum => write!(f, "Africa/Khartoum (+02:00)"),
            TimezoneChoices::AfricaNairobi => write!(f, "Africa/Nairobi (+03:00)"),
            TimezoneChoices::AmericaAdak => write!(f, "America/Adak (-10:00, -9:00)"),
            TimezoneChoices::AmericaAnchorage => write!(f, "America/Anchorage (-09:00, -8:00)"),
            TimezoneChoices::AmericaAraguaina => write!(f, "America/Araguaina (-03:00)"),
            TimezoneChoices::AmericaAsuncion => write!(f, "America/Asuncion (-04:00 /-03:00)"),
            TimezoneChoices::AmericaBarbados => write!(f, "America/Barbados (-04:00)"),
            TimezoneChoices::AmericaBahiaBanderas => write!(f, "America/Bahia_Banderas (-06:00)"),
            TimezoneChoices::AmericaBogota => write!(f, "America/Bogota (-05:00)"),
            TimezoneChoices::AmericaBoise => write!(f, "America/Boise (-07:00 / -06:00)"),
            TimezoneChoices::EuropeZurich => write!(f, "Europe/Zurich (+01:00 / +02:00)"),
            TimezoneChoices::EuropeBucharest => write!(f, "Europe/Bucharest (+02:00 / +03:00)"),
        }
    }
}
