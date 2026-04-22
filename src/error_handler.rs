use clap::builder::Str;

use crate::parser::Parser;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

pub fn handle_error<S: Into<String>>(msg: S, line: usize, file: S) -> ! {
    let mut msg = msg.into();
    let mut chars = msg.chars();
    msg = match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    };
    panic!(
        "{} at line {} in file {} \n -> ",
        msg,
        line + 1,
        file.into()
    );
}
pub fn handle_expected_error<S: Into<String>>(
    expected: S,
    found: S,
    after: S,
    line: usize,
    file: S,
) -> ! {
    panic!(
        "Expected {} found {} {} at line {} in file {}",
        expected.into(),
        found.into(),
        after.into(),
        line,
        file.into()
    );
}
pub fn handle_error_parser(error_message: CompilationError, parser: &Parser) -> ! {
    let error_msg = error_message.get_msg(&parser);
    let msg = error_msg.msg;
    let hint = error_msg.hint;
    let example = error_msg.example;
    panic!("\x1b[31mError:\x1b[0m {} \n\n-> {} \n\nFile: {}:{} \n\nHint: {} \n\nExample:\n {}",
        msg,
        parser.get_tok_around_colored(10),
        parser.get_file_path(),
        parser.get_line(),
        hint,
        example,
    );
}

struct ErrorMessage {
    msg: String,
    hint: String,
    example: String,
}
impl ErrorMessage {
    pub fn new<S: Into<String>>(msg: S, hint: S, example: S) -> Self {
        Self {
            msg: msg.into(),
            hint: hint.into(),
            example: example.into(),
        }
    }
}
pub enum CompilationError {
    InvalidFunc,
    InvalidTokenInPath,
    InvalidAfterFilePath,
    InvalidTokenInIncludePath,
    InvalidDefName,
    InvalidDefPlaceName,
    InvalidFinishTokWhen,
    Invalid2ndIdentWhen,
    InvalidComparissonTok,
    Invalid1stIdentWhen,
    Invalid2ndIdentDefWhere,
    InvalidAssignementDefWhere,
    Invalid1stIdentDefWhere,
    InvalidDefOption,
    InvalidVar,
    NoDDEndef,
    NLArrowVarName,
    NotMarkAfterArrowVar,
    NotArrow,
    InvalidArrowVarName,
    InvalidBodyCommand,
    BodyEOF,
    InvalidPlaceName,
    Invalid2ndPlaceVar,
    InvalidPlaceAssign,
    Invalid1stPlaceVar,
    EOFInQuotVar,
    NoDDAfterQuotVar,
    InvalidPlaceOption,
}
impl CompilationError {
    fn get_msg(&self, parser: &Parser) -> ErrorMessage {
        match self {
            CompilationError::InvalidFunc => ErrorMessage::new(
                format!("Invalid token after mark {:?}", parser.peek()),
                format!("Check documentation to see valid commands"),
                format!("//- {}def{} a:", YELLOW, RESET),
            ),
            CompilationError::InvalidTokenInPath => ErrorMessage::new(
                format!(
                    "Invalid token found in file path in create {:?}",
                    parser.peek()
                ),
                format!("Check if the path is valid, it must only contain valid characters"),
                format!("//- create {}folder/file.txt:{}", YELLOW, RESET),
            ),
            CompilationError::InvalidAfterFilePath => ErrorMessage::new(
                format!(
                    "Invalid command after file path in create {:?}",
                    parser.peek()
                ),
                format!("See documentation to see wich commands are supported inside create"),
                format!(
                    "//- create folder/file.txt {}place{} template:",
                    YELLOW, RESET
                ),
            ),
            CompilationError::InvalidTokenInIncludePath => ErrorMessage::new(
                format!("Invalid token found in path in include {:?}", parser.peek()),
                format!("Check if the path is valid, it must only contain valid characters"),
                format!("//- include {}folder/file.txt{}:", YELLOW, RESET),
            ),
            CompilationError::InvalidDefName => ErrorMessage::new(
                format!(
                    "Invalid token found in def declaration name {:?}",
                    parser.peek()
                ),
                format!("Make sure the template name contains only valid characters"),
                format!("//- def {}template{}:", YELLOW, RESET),
            ),
            CompilationError::InvalidDefPlaceName => ErrorMessage::new(
                format!(
                    "Invalid token found inde def place after place {:?}",
                    parser.peek()
                ),
                format!("Make sure the place name contains only valid characters"),
                format!("//- def templateA place {}templateB{}:", YELLOW, RESET),
            ),
            CompilationError::InvalidFinishTokWhen => ErrorMessage::new(
                format!("Invalid token at the end of def when {:?}", parser.peek()),
                format!("Check documentation to see wich commands are supported inside def"),
                format!("//- def template when var = val{}:{}", YELLOW, RESET),
            ),
            CompilationError::Invalid2ndIdentWhen => ErrorMessage::new(
                format!(
                    "Invalid token found in variable name after the comparison in def when {:?}",
                    parser.peek()
                ),
                format!("Make sure the variable name contains only valid characters"),
                format!("//- def template when var = {}val{}:", YELLOW, RESET),
            ),
            CompilationError::InvalidComparissonTok => ErrorMessage::new(
                format!(
                    "Invalid token found in condition in def when {:?}",
                    parser.peek()
                ),
                format!("Check documentation to see wich conditions are valid"),
                format!("//- def template when var {}={} val:", YELLOW, RESET),
            ),
            CompilationError::Invalid1stIdentWhen => ErrorMessage::new(
                format!(
                    "Invalid token in variable name before comparison in def when {:?}",
                    parser.peek()
                ),
                format!("Make sure the variable contains only valid characters"),
                format!("//- def template when {}var{} = val:", YELLOW, RESET),
            ),
            CompilationError::Invalid2ndIdentDefWhere => ErrorMessage::new(
                format!(
                    "Invalid token found after assignement simbol in def where {:?}",
                    parser.peek()
                ),
                format!("Make sure the value contains only valid characters"),
                format!("//- def template where var = {}val{}", YELLOW, RESET),
            ),
            CompilationError::InvalidAssignementDefWhere => ErrorMessage::new(
                format!(
                    "Invalid token in assignement in def where {:?}",
                    parser.peek()
                ),
                format!("Replace the current token with \"=\" "),
                format!("//- def template where var{}={}val:", YELLOW, RESET),
            ),
            CompilationError::Invalid1stIdentDefWhere => ErrorMessage::new(
                format!(
                    "Invalid token found before assignement simbol in def where {:?}",
                    parser.peek()
                ),
                format!("Make sure the variable name contains only valid characters"),
                format!("//- def template where {}var{}=val:", YELLOW, RESET),
            ),
            CompilationError::InvalidDefOption => ErrorMessage::new(
                format!("Invalid option for def {:?}", parser.peek()),
                format!("Check documentation to see the available option for def or add :"),
                format!("//- def template {}where{} var=val:", YELLOW, RESET),
            ),
            CompilationError::InvalidVar => ErrorMessage::new(
                format!("Invalid variable name {:?}", parser.peek()),
                format!("Make sure the variable name does not conflict with reserved keywords"),
                format!("$#{}var{}", YELLOW, RESET),
            ),
            CompilationError::NoDDEndef => ErrorMessage::new(
                format!("Expected : after endef found {:?}", parser.peek()),
                format!("Add : after endef"),
                format!("//- endef{}:{}", YELLOW, RESET),
            ),
            CompilationError::NLArrowVarName => ErrorMessage::new(
                format!("Newline not supported as arrow variable default"),
                format!("Remove the newline"),
                format!("/*- $#var -> -*/ {}default{}", YELLOW, RESET),
            ),
            Self::NotMarkAfterArrowVar => ErrorMessage::new(
                format!(
                    "Invalid token at the end of arrow var {:?} expected mark",
                    parser.peek()
                ),
                format!("Add a mark after the arrow"),
                format!("/*- $#varname -> {}-*/{} default", YELLOW, RESET),
            ),
            CompilationError::NotArrow => ErrorMessage::new(
                format!(
                    "Invalid token after arrow variable name {:?} expected -> ",
                    parser.peek()
                ),
                format!(
                    "Add -> after the variable name or don't use variable simbol right after a mark"
                ),
                format!("/*- $#var {}->{} -*/ default", YELLOW, RESET),
            ),
            CompilationError::InvalidArrowVarName => ErrorMessage::new(
                format!("Invalid arrow var name {:?}", parser.peek()),
                format!(
                    "Make sure the name of the arrow variable contains only valid characters and no reserved words"
                ),
                format!("/*- $#{}var{} -> -*/ default", YELLOW, RESET),
            ),
            CompilationError::InvalidBodyCommand => ErrorMessage::new(
                format!("Invalid command inside body {:?}", parser.peek()),
                format!("Check documentation to see wich commads can be used inside a body"),
                format!(
                    "
                 //- def templateA: 
                    //- {}place{} templateB:
                 //- endef:
                 ",
                    YELLOW, RESET
                ),
            ),
            Self::BodyEOF => ErrorMessage::new(
                format!("Found EOF inside body"),
                format!("The file may have been incorrecly overriten or the code is incomplete"),
                format!(
                    "
                //- def template:
                    {}body{}
                //- endef:
                ",
                    YELLOW, RESET
                ),
            ),
            Self::InvalidPlaceName => ErrorMessage::new(
                format!("Invalid template name in place {:?}", parser.peek()),
                format!(
                    "Make sure the template name contains only valid characters and no reserved keywords"
                ),
                format!("//- place {}template{}:", YELLOW, RESET),
            ),
            CompilationError::Invalid2ndPlaceVar => ErrorMessage::new(
                format!("Invalid value after assignement in place where {:?}", parser.peek()),
                format!(
                    "Make sure the value contains only valid characters and not reserved keywords"
                ),
                format!("//- place template where var={}val{}:", YELLOW, RESET),
            ),
            CompilationError::InvalidPlaceAssign => ErrorMessage::new(
                format!("Invalid assignement token in place were {:?}", parser.peek()), 
                format!("Add or replace your token for = in the variable assignement"), 
                format!("//- place name where var{}={}val",YELLOW,RESET)
            ),
            CompilationError::Invalid1stPlaceVar => ErrorMessage::new(
                format!("Invalid variable name before assignement in place where {:?}", parser.peek()), 
                format!("Make sure the value contains only valid characters and not reserved keywords"), 
                format!("//- palce template where {}var{}=val:",YELLOW,RESET)
            ),
            Self::EOFInQuotVar => ErrorMessage::new(
                format!("Found EOF inside of quotation variable"),
                format!("The file may have been incorrecly overriten or the code is incomplete"),
                format!("//- place template where var=\"{}val{}\"", YELLOW, RESET),
            ),
            Self::NoDDAfterQuotVar => ErrorMessage::new(
                format!("Invalid token after quote variable in place where {:?}",parser.peek()), 
                format!("Add : or , after quote variable"), 
                format!("//- place name where var=\"val\"{}:{}",YELLOW,RESET)
            ),
            Self::InvalidPlaceOption => ErrorMessage::new(
                format!("Invalid option for place {:?}",parser.peek()), 
                format!("Check documentation for place options or add : at the end"), 
                format!("//- place template {}where{} var=val:",YELLOW,RESET)
            )
        }
    }
}
