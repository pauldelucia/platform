use crate::errors::consensus::ConsensusError;
use crate::ProtocolError;
use std::fmt::Debug;

pub type SimpleConsensusValidationResult = ConsensusValidationResult<()>;

pub type SimpleValidationResult<E> = ValidationResult<(), E>;

pub type ConsensusValidationResult<TData: Clone> = ValidationResult<TData, ConsensusError>;

#[derive(Debug)]
pub struct ValidationResult<TData: Clone, E: Debug> {
    pub errors: Vec<E>,
    pub data: Option<TData>,
}

impl<T: Clone, E: Debug> Default for ValidationResult<T, E> {
    fn default() -> Self {
        ValidationResult {
            errors: Vec::new(),
            data: None,
        }
    }
}

impl<TData: Clone, E: Debug> ValidationResult<TData, E> {
    pub fn new_with_data(data: TData) -> Self {
        Self {
            errors: vec![],
            data: Some(data),
        }
    }

    pub fn new_with_data_and_errors(data: TData, errors: Vec<E>) -> Self {
        Self {
            errors,
            data: Some(data),
        }
    }

    pub fn new_with_errors(errors: Vec<E>) -> Self {
        Self { errors, data: None }
    }

    pub fn map<F, U: Clone>(self, f: F) -> ValidationResult<U, E>
    where
        F: FnOnce(TData) -> U,
    {
        ValidationResult {
            errors: self.errors,
            data: self.data.map(f),
        }
    }

    pub fn map_result<F, U: Clone, G>(self, f: F) -> Result<ValidationResult<U, E>, G>
    where
        F: FnOnce(TData) -> Result<U, G>,
    {
        Ok(ValidationResult {
            errors: self.errors,
            data: self.data.map(f).transpose()?,
        })
    }

    pub fn and_then_simple_validation<F>(
        self,
        f: F,
    ) -> Result<ValidationResult<TData, E>, ProtocolError>
    where
        F: FnOnce(&TData) -> Result<SimpleValidationResult<E>, ProtocolError>,
    {
        let new_errors = self.data.as_ref().map(f).transpose()?;
        let mut result = ValidationResult {
            errors: self.errors,
            data: self.data,
        };
        if let Some(new_errors) = new_errors {
            result.add_errors(new_errors.errors)
        }
        Ok(result)
    }

    pub fn and_then_validation<F, U: Clone, G>(self, f: F) -> Result<ValidationResult<U, E>, G>
    where
        F: FnOnce(TData) -> Result<ValidationResult<U, E>, G>,
    {
        if let Some(data) = self.data {
            let mut new_validation_result = f(data)?;
            new_validation_result.add_errors(self.errors);
            Ok(new_validation_result)
        } else {
            Ok(ValidationResult::<U, E>::new_with_errors(self.errors))
        }
    }

    pub fn and_then_borrowed_validation<F, U: Clone, G>(
        self,
        f: F,
    ) -> Result<ValidationResult<U, E>, G>
    where
        F: FnOnce(&TData) -> Result<ValidationResult<U, E>, G>,
    {
        if let Some(data) = self.data.as_ref() {
            let mut new_validation_result = f(data)?;
            new_validation_result.add_errors(self.errors);
            Ok(new_validation_result)
        } else {
            Ok(ValidationResult::<U, E>::new_with_errors(self.errors))
        }
    }

    pub fn add_error<T>(&mut self, error: T)
    where
        T: Into<E>,
    {
        self.errors.push(error.into())
    }

    pub fn add_errors(&mut self, mut errors: Vec<E>) {
        self.errors.append(&mut errors)
    }

    pub fn merge<TOtherData: Clone>(&mut self, mut other: ValidationResult<TOtherData, E>) {
        self.errors.append(&mut other.errors);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn first_error(&self) -> Option<&E> {
        self.errors.first()
    }

    pub fn into_result_without_data(self) -> ValidationResult<(), E> {
        ValidationResult {
            errors: self.errors,
            data: None,
        }
    }

    pub fn is_valid_with_data(&self) -> bool {
        self.is_valid() && self.data.is_some()
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn set_data(&mut self, data: TData) {
        self.data = Some(data)
    }

    pub fn into_data(self) -> Result<TData, ProtocolError> {
        self.data
            .ok_or(ProtocolError::CorruptedCodeExecution(format!(
                "trying to push validation result into data (errors are {:?})",
                self.errors
            )))
    }

    pub fn data_as_borrowed(&self) -> Result<&TData, ProtocolError> {
        self.data
            .as_ref()
            .ok_or(ProtocolError::CorruptedCodeExecution(format!(
                "trying to get validation result as data (errors are {:?})",
                self.errors
            )))
    }
}

impl<TData: Clone, E: Debug> From<TData> for ValidationResult<TData, E> {
    fn from(value: TData) -> Self {
        ValidationResult::new_with_data(value)
    }
}

impl<TData: Clone, E: Debug> From<Result<TData, E>> for ValidationResult<TData, E> {
    fn from(value: Result<TData, E>) -> Self {
        match value {
            Ok(data) => ValidationResult::new_with_data(data),
            Err(e) => ValidationResult::new_with_errors(vec![e]),
        }
    }
}
