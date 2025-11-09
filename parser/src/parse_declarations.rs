use crate::Parser;
use crate::ast::{Block, Declaration, FunctionDecl, StorageClass, VariableDecl};
use crate::parse_err::ParseErr;
use lexer::SpannedToken;
use lexer::token::Token;
use shared_context::Type;
use shared_context::{Identifier, Span, SpannedIdentifier};

impl<'src, 'ctx> Parser<'src, 'ctx> {
    /// Parses a declaration, determining whether it is a function or variable declaration.
    pub(crate) fn parse_declaration(&mut self) -> Result<Declaration, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let (decl_type, storage_class) = self.parse_type_and_storage_class_list()?;

        let token = self.peek_two()?.get_token();
        match token {
            Token::LeftParenthesis => Ok(Declaration::FunDecl(self.parse_function_decl(
                decl_type,
                storage_class,
                start,
                line,
            )?)),
            _ => Ok(Declaration::VarDecl(self.parse_variable_declaration(
                decl_type,
                storage_class,
                start,
                line,
            )?)),
        }
    }

    /// parse a specifier list to determine the type and storage class of a declaration
    pub(crate) fn parse_type_and_storage_class_list(
        &mut self,
    ) -> Result<(Type, StorageClass), ParseErr> {
        let (list, span) = self.collect_declaration_specifiers()?;
        let mut type_list = Vec::new();
        let mut storage_class_list = Vec::new();

        for specifier in list {
            if specifier.get_token().is_type() {
                type_list.push(specifier);
            } else {
                storage_class_list.push(specifier);
            }
        }

        let decl_type = self.parse_type(type_list, span)?;

        if storage_class_list.len() > 1 {
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

        Ok((decl_type, storage_class))
    }

    /// collect all the tokens that make up a declaration specifier into one vector,
    /// return the vector and span of the list of specifiers,
    /// this list will be used to parse the types and storage class of the declaration
    fn collect_declaration_specifiers(
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

    /// parse a specifier list that doesn't contain storage class specifiers
    ///
    /// return an err if a storage class specifier is found
    pub(crate) fn parse_type_list(&mut self) -> Result<Type, ParseErr> {
        let (list, span) = self.collect_declaration_specifiers()?;
        let mut type_list = Vec::new();

        for specifier in list {
            if specifier.get_token().is_type() {
                type_list.push(specifier);
            } else {
                return Err(ParseErr::new(
                    "Invalid use of storage class specifier",
                    span,
                    &self.source_map,
                ));
            }
        }

        self.parse_type(type_list, span)
    }

    /// parse types annotations in a specifier list
    fn parse_type(&mut self, token_list: Vec<SpannedToken>, span: Span) -> Result<Type, ParseErr> {
        let type_list: Vec<&str> = token_list.iter().map(|st| st.get_lexeme()).collect();
        match type_list.as_slice() {
            ["int"] => Ok(Type::Int),
            ["int", "long"] | ["long", "int"] | ["long"] => Ok(Type::Long),
            _ => Err(ParseErr::new(
                "invalid type specifier",
                span,
                &self.source_map,
            )),
        }
    }

    /// parse storage and linkage specifiers
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
    pub(crate) fn parse_variable_declaration(
        &mut self,
        var_type: Type,
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
        Ok(VariableDecl::new(name, var_type, init, storage_class, span))
    }

    /// Parses a function declaration:
    fn parse_function_decl(
        &mut self,
        ret_type: Type,
        storage_class: StorageClass,
        start: usize,
        line: usize,
    ) -> Result<FunctionDecl, ParseErr> {
        let name = self.parse_identifier()?;

        self.expect_token(Token::LeftParenthesis)?;
        let (params_types, params_iden) = self.parse_params_list()?;
        self.expect_token(Token::RightParenthesis)?;

        let body = self.parse_optional_block()?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);
        let type_id = self.ty_interner.intern(ret_type, &params_types);

        Ok(FunctionDecl::new(
            name,
            type_id,
            params_iden,
            body,
            storage_class,
            span,
        ))
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
    ///
    /// Returns two vectors
    /// - vector of params identifiers
    /// - vector for params types
    fn parse_params_list(&mut self) -> Result<(Vec<Type>, Vec<SpannedIdentifier>), ParseErr> {
        let mut params_iden = Vec::new();
        let mut params_type = Vec::new();

        if self.peek()?.get_token() == Token::Void {
            self.advance()?; // consume 'void'
            return Ok((params_type, params_iden));
        }

        let param_type = self.parse_type_list()?;
        params_iden.push(self.parse_identifier()?);
        params_type.push(param_type);

        while self.peek()?.get_token() != Token::RightParenthesis {
            self.expect_token(Token::Comma)?;
            let param_type = self.parse_type_list()?;
            params_iden.push(self.parse_identifier()?);
            params_type.push(param_type);
        }
        Ok((params_type, params_iden))
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
            let identifier = Identifier::new(self.sy_interner.intern(token.get_lexeme()), 0);
            Ok(SpannedIdentifier::new(identifier, span))
        } else {
            Err(ParseErr::expected("identifier", &token, &self.source_map))
        }
    }
}
