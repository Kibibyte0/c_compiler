use crate::Parser;
use crate::ast::{Block, Declaration, FunctionDecl, StorageClass, VariableDecl};
use crate::parse_err::ParseErr;
use lexer::SpannedToken;
use lexer::token::Token;
use shared_context::{Identifier, Span, SpannedIdentifier};

impl<'src, 'ctx> Parser<'src, 'ctx> {
    /// Parses a declaration, determining whether it is a function or variable declaration.
    pub(crate) fn parse_declaration(&mut self) -> Result<Declaration, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let (specifier_list, specifier_span) = self.collect_declaration_specifiers()?;
        let storage_class = self.parse_specifier_list(specifier_list, specifier_span)?;

        let token = self.peek_two()?.get_token();
        match token {
            Token::LeftParenthesis => Ok(Declaration::FunDecl(self.parse_function_decl(
                storage_class,
                start,
                line,
            )?)),
            _ => Ok(Declaration::VarDecl(self.parse_variable_declaration(
                storage_class,
                start,
                line,
            )?)),
        }
    }

    /// parse a specifier list to determine the type and storage class of a declaration
    ///
    /// currently it returns storage class only because int is the only valid type
    pub(crate) fn parse_specifier_list(
        &mut self,
        list: Vec<SpannedToken>,
        span: Span,
    ) -> Result<StorageClass, ParseErr> {
        let mut type_list = Vec::new();
        let mut storage_class_list = Vec::new();

        for specifier in list {
            match specifier.get_token() {
                Token::Int => type_list.push(specifier),
                _ => storage_class_list.push(specifier),
            }
        }

        if type_list.len() != 1 || storage_class_list.len() > 1 {
            return Err(ParseErr::new(
                "Invalid declaration specifier",
                span,
                &self.source_map,
            ));
        }

        let storage_class = match storage_class_list.get(0) {
            Some(class) => self.parse_storage_class(*class)?,
            None => StorageClass::None,
        };

        Ok(storage_class)
    }

    /// collect all the tokens that make up a declaration specifier into one vector,
    /// return the vector and span of the list of specifiers,
    /// this list will be used to parse the types and storage class of the declaration
    pub(crate) fn collect_declaration_specifiers(
        &mut self,
    ) -> Result<(Vec<SpannedToken<'src>>, Span), ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        let mut specifier_list = Vec::new();
        while self.peek()?.get_token().is_specifier() {
            self.advance()?; // consume the token
            specifier_list.push(self.current_token);
        }

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok((specifier_list, span))
    }

    fn parse_storage_class(&self, token: SpannedToken) -> Result<StorageClass, ParseErr> {
        match token.get_token() {
            Token::Static => Ok(StorageClass::Static),
            Token::Extern => Ok(StorageClass::Extern),
            _ => Err(ParseErr::expected(
                "storage specifier",
                &token,
                &self.source_map,
            )),
        }
    }

    /// Parses a variable declaration:
    ///
    /// ```text
    /// { <specifier> }+ <identifier> [= <expr>] ;
    /// ```
    pub(crate) fn parse_variable_declaration(
        &mut self,
        storage_class: StorageClass,
        start: usize,
        line: usize,
    ) -> Result<VariableDecl, ParseErr> {
        let name = self.parse_identifier()?;
        let init = match self.peek()?.get_token() {
            Token::Assignment => {
                self.advance()?; // consume '='
                Some(self.parse_expression(0)?)
            }
            _ => None,
        };

        self.expect_token(Token::Semicolon)?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);
        Ok(VariableDecl::new(name, init, storage_class, span))
    }

    /// Parses a function declaration:
    ///
    /// ```text
    /// int <identifier> ( <param-list> ) <block-or-semicolon>
    /// ```
    fn parse_function_decl(
        &mut self,
        storage_class: StorageClass,
        start: usize,
        line: usize,
    ) -> Result<FunctionDecl, ParseErr> {
        let name = self.parse_identifier()?;

        self.expect_token(Token::LeftParenthesis)?;
        let params = self.parse_params_list()?;
        self.expect_token(Token::RightParenthesis)?;

        let body = self.parse_optional_block()?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok(FunctionDecl::new(name, params, body, storage_class, span))
    }

    /// Parses an optional function body block.
    ///
    /// Either a `{ ... }` block or a terminating semicolon (`;`) for
    /// declarations without a body.
    fn parse_optional_block(&mut self) -> Result<Option<Block>, ParseErr> {
        match self.peek()?.get_token() {
            Token::LeftCurlyBracket => Ok(Some(self.parse_block()?)),
            _ => {
                self.expect_token(Token::Semicolon)?;
                Ok(None)
            }
        }
    }

    /// Parses a function parameter list.
    ///
    /// Accepts either:
    /// - `void` (no parameters), or
    /// - one or more `int <identifier>` pairs separated by commas.
    fn parse_params_list(&mut self) -> Result<Vec<SpannedIdentifier>, ParseErr> {
        let mut params = Vec::new();
        if self.peek()?.get_token() == Token::Void {
            self.advance()?; // consume 'void'
            return Ok(params);
        }

        self.expect_token(Token::Int)?;
        params.push(self.parse_identifier()?);

        while self.peek()?.get_token() != Token::RightParenthesis {
            self.expect_token(Token::Comma)?;
            self.expect_token(Token::Int)?;
            params.push(self.parse_identifier()?);
        }
        Ok(params)
    }

    /// Parses an identifier and returns it as a SpannedToken.
    ///
    /// Converts the lexeme into an interned identifier and attaches
    /// span information for error reporting.
    pub(crate) fn parse_identifier(&mut self) -> Result<SpannedIdentifier, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let token = self.advance()?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        if token.get_token() == Token::Identifier {
            let identifier = Identifier::new(self.interner.intern(token.get_lexeme()), 0);
            Ok(SpannedIdentifier::new(identifier, span))
        } else {
            Err(ParseErr::expected("identifier", &token, &self.source_map))
        }
    }
}
