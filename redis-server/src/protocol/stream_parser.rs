use crate::datatypes::DataType;

pub enum ParseResult {
    IsMore,
    Complete,
}

trait CommandParser {
    fn next(
        &mut self,
        str: &str,
    ) -> Result<(ParseResult, Option<Box<dyn CommandParser + Send>>), String>;

    fn to_datatype(&self) -> Result<DataType, String>;
}

struct BaseCommandParser {}

impl BaseCommandParser {
    fn new() -> BaseCommandParser {
        BaseCommandParser {}
    }
}

impl CommandParser for BaseCommandParser {
    fn next(
        &mut self,
        str: &str,
    ) -> Result<(ParseResult, Option<Box<dyn CommandParser + Send>>), String> {
        if str.starts_with("*") {
            let count = &str[1..];
            println!("Parsing an array w/ count: {count}");
            let count_usize = count.parse::<usize>().map_err(|err| err.to_string())?;
            return Ok((
                ParseResult::IsMore,
                Some(Box::from(ArrayCommandParser::new(count_usize))),
            ));
        }
        if str.starts_with("$") {
            let count = &str[1..];
            println!("Parsing a string w/ count: {count}");
            let count_usize = count.parse::<usize>().map_err(|err| err.to_string())?;
            return Ok((
                ParseResult::IsMore,
                Some(Box::from(StringParser::new(count_usize))),
            ));
        }

        Ok((
            ParseResult::Complete,
            Some(Box::from(BaseCommandParser::new())),
        ))
    }

    fn to_datatype(&self) -> Result<DataType, String> {
        Ok(DataType::Nil)
    }
}

struct StringParser {
    state: String,
}

impl StringParser {
    fn new(size: usize) -> StringParser {
        StringParser {
            state: String::with_capacity(size),
        }
    }
}

impl CommandParser for StringParser {
    fn next(
        &mut self,
        str: &str,
    ) -> Result<(ParseResult, Option<Box<dyn CommandParser + Send>>), String> {
        if str.len() != self.state.capacity() {
            return Err("Expected size of string to match spec".to_string());
        }
        self.state.push_str(str);

        Ok((ParseResult::Complete, None))
    }

    fn to_datatype(&self) -> Result<DataType, String> {
        Ok(DataType::BulkString(self.state.to_string()))
    }
}

struct ArrayCommandParser {
    size: usize,
    current_iter: usize,
    state: Vec<Parser>,
}

impl ArrayCommandParser {
    fn new(size: usize) -> ArrayCommandParser {
        ArrayCommandParser {
            size,
            current_iter: 0,
            state: Vec::with_capacity(size),
        }
    }
}

// Handling connection!
// Parsing Line: *3
// Parsing Line: $3
// Parsing Line: SET
// Parsing Line: $1
// Parsing Line: Y
// Parsing Line: $1
// Parsing Line: 1
// Closing connection!

impl CommandParser for ArrayCommandParser {
    fn next(
        &mut self,
        str: &str,
    ) -> Result<(ParseResult, Option<Box<dyn CommandParser + Send>>), String> {
        println!("Array parser {} {}", self.size, self.current_iter);
        if self.current_iter == self.state.len() {
            self.state.push(Parser::new());
        }
        let current_parser = self.state.get_mut(self.current_iter).expect(&format!(
            "Expected a parser to exist at index {}",
            self.current_iter
        ));
        let parse_result = current_parser.next(str)?;

        match parse_result {
            ParseResult::IsMore => Ok((ParseResult::IsMore, None)),
            ParseResult::Complete if self.current_iter + 1 == self.size => {
                Ok((ParseResult::Complete, None))
            }
            ParseResult::Complete => {
                self.current_iter += 1;
                self.state.push(Parser::new());
                Ok((ParseResult::IsMore, None))
            }
        }
    }

    fn to_datatype(&self) -> Result<DataType, String> {
        let arr = self
            .state
            .iter()
            .map(|x| x.to_datatype())
            .collect::<Result<Vec<DataType>, String>>()?;
        Ok(DataType::Array(arr))
    }
}

pub struct Parser {
    state: Box<dyn CommandParser + Send>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: Box::from(BaseCommandParser::new()),
        }
    }

    pub fn next(&mut self, str: &str) -> Result<ParseResult, String> {
        let (parse_result, next_parser) = self.state.next(str)?;
        if let Some(next_parser) = next_parser {
            self.state = next_parser;
        }

        Ok(parse_result)
    }

    pub fn to_datatype(&self) -> Result<DataType, String> {
        self.state.to_datatype()
    }
}
