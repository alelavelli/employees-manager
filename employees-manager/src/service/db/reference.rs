use std::marker::PhantomData;

use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{error::ServiceAppError, service::db::document::DatabaseDocument, DocumentId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReferenceField<T: DatabaseDocument> {
    Cascade(DocumentId, PhantomData<T>),
    NoCascade(DocumentId, PhantomData<T>),
}

impl<T: DatabaseDocument> ReferenceField<T> {
    fn get_id(&self) -> &DocumentId {
        match self {
            ReferenceField::Cascade(doc_id, _) => doc_id,
            ReferenceField::NoCascade(doc_id, _) => doc_id,
        }
    }

    async fn fetch(&self) -> Result<Option<T>, ServiceAppError> {
        let document_id = match self {
            ReferenceField::Cascade(doc_id, _) => doc_id,
            ReferenceField::NoCascade(doc_id, _) => doc_id,
        };
        T::find_one(doc! {"_id": document_id}).await
    }

    async fn on_delete(self) -> Result<(), ServiceAppError> {
        match self {
            ReferenceField::Cascade(doc_id, _) => {
                let document = T::find_one(doc! {"_id": doc_id})
                    .await?
                    .ok_or(ServiceAppError::EntityDoesNotExist(format!("")))?;

                document.delete().await;
            }
            ReferenceField::NoCascade(_, _) => {}
        };
        Ok(())
    }
}

pub trait ReferenceFieldTrait {
    fn as_any(&self) -> &dyn std::any::Any;
    fn get_id(&self) -> &DocumentId;
    /* async fn fetch(&self) -> Result<Option<impl DatabaseDocument>, ServiceAppError>;
    async fn on_delete(
        self,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> Result<(), ServiceAppError>; */
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CascadeReferenceField<T: DatabaseDocument> {
    document_id: DocumentId,
    phantom: PhantomData<T>,
}

impl<T: DatabaseDocument> CascadeReferenceField<T> {
    async fn fetch(&self) -> Result<Option<impl DatabaseDocument>, ServiceAppError> {
        T::find_one(doc! {"_id": self.document_id}).await
    }
    /*
    async fn on_delete(
        self,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> Result<(), ServiceAppError> {
        let document = T::find_one(doc! {"_id": self.document_id})
            .await?
            .ok_or(ServiceAppError::EntityDoesNotExist(format!("")))?;
        let reference_fields = document.get_reference_fields();
        todo!();
        document.delete(transaction).await?;
        Ok(())
    } */
}

impl<T: DatabaseDocument + 'static> ReferenceFieldTrait for CascadeReferenceField<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_id(&self) -> &DocumentId {
        &self.document_id
    }
}

impl<T: DatabaseDocument> NoCascadeReferenceField<T> {
    async fn fetch(&self) -> Result<Option<impl DatabaseDocument>, ServiceAppError> {
        T::find_one(doc! {"_id": self.document_id}).await
    }

    async fn on_delete(self) -> Result<(), ServiceAppError> {
        let document = T::find_one(doc! {"_id": self.document_id})
            .await?
            .ok_or(ServiceAppError::EntityDoesNotExist(format!("")))?;

        document.delete().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoCascadeReferenceField<T: DatabaseDocument> {
    document_id: DocumentId,
    phantom: PhantomData<T>,
}

impl<T: DatabaseDocument + 'static> ReferenceFieldTrait for NoCascadeReferenceField<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_id(&self) -> &DocumentId {
        &self.document_id
    }
}
