//! Build-time linked models. Engineering fixtures never enter the official catalog.
use rsphysics_model_api::{ErrorKind, ModelAdapter, ModelDescriptor, ModelError, TypedAdapter};
use std::collections::BTreeMap;

pub struct Catalog {
    models: BTreeMap<String, Box<dyn ModelAdapter>>,
}

impl Catalog {
    pub fn new(models: Vec<Box<dyn ModelAdapter>>) -> Result<Self, ModelError> {
        let mut catalog = Self {
            models: BTreeMap::new(),
        };
        for model in models {
            model.descriptor().metadata.validate()?;
            let id = model.descriptor().metadata.id.as_str().to_owned();
            if catalog.models.contains_key(&id) {
                return Err(ModelError::new(
                    ErrorKind::InvalidParameter,
                    format!("duplicate model ID: {id}"),
                ));
            }
            catalog.models.insert(id, model);
        }
        Ok(catalog)
    }

    pub fn list_models(&self) -> Vec<&ModelDescriptor> {
        self.models
            .values()
            .map(|model| model.descriptor())
            .collect()
    }

    pub fn get(&self, id: &str) -> Result<&dyn ModelAdapter, ModelError> {
        self.models.get(id).map(Box::as_ref).ok_or_else(|| {
            ModelError::new(
                ErrorKind::UnknownModel,
                format!("model is not compiled into this build: {id}"),
            )
        })
    }
}

/// Scientific models compiled into this build. Templates/fixtures never enter this list.
pub fn official_catalog() -> Result<Catalog, ModelError> {
    Catalog::new(vec![
        Box::new(TypedAdapter::new(rsphysics_i2em::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_sort::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_sort::LambertModel::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_lmeb::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_gort_gpp::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_smac::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_water_cloud::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_tau_omega::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_prospect_d::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_prosail_d::Model::new()?)?),
        Box::new(TypedAdapter::new(rsphysics_ross_li::Model::new()?)?),
    ])
}
