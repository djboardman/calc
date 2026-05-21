#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Currency(String),
    Money { currency: String, minor_units: i64 },
    Text(String),
    Boolean(bool),
    List(Vec<Value>),
}

impl Value {
    pub(crate) fn number(number: f64) -> Self {
        Self::Number(number)
    }

    pub(crate) fn currency(currency: impl Into<String>) -> Self {
        Self::Currency(currency.into())
    }

    pub(crate) fn money(currency: impl Into<String>, minor_units: i64) -> Self {
        Self::Money {
            currency: currency.into(),
            minor_units,
        }
    }

    pub(crate) fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub(crate) fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub(crate) fn list(values: Vec<Value>) -> Self {
        Self::List(values)
    }

    pub(crate) fn type_key(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::Currency(currency) => ValueType::Currency(currency.clone()),
            Self::Money { currency, .. } => ValueType::Money(currency.clone()),
            Self::Text(_) => ValueType::Text,
            Self::Boolean(_) => ValueType::Boolean,
            Self::List(values) => ValueType::List(
                values
                    .first()
                    .map(|value| Box::new(value.type_key()))
                    .unwrap_or_else(|| Box::new(ValueType::EmptyList)),
            ),
        }
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Number(number) => number.to_string(),
            Self::Currency(currency) => currency.clone(),
            Self::Money {
                currency,
                minor_units,
            } => {
                let sign = if *minor_units < 0 { "-" } else { "" };
                let absolute = minor_units.abs();
                let currency = money_display_currency(currency);
                format!("{sign}{currency}{}.{:02}", absolute / 100, absolute % 100)
            }
            Self::Text(text) => format!("\"{}\"", text),
            Self::Boolean(value) => value.to_string(),
            Self::List(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::display_text)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ValueType {
    Number,
    Currency(String),
    Money(String),
    Text,
    Boolean,
    List(Box<ValueType>),
    EmptyList,
}

fn money_display_currency(currency: &str) -> &str {
    match currency {
        "GBP" => "£",
        "USD" => "$",
        "EUR" => "€",
        _ => currency,
    }
}
