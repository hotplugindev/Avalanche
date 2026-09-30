use avalanche_doctor::DoctorService;
use avalanche_git::GitService;
use avalanche_graph::GraphService;
use avalanche_index::{IndexResult, IndexService, RepositoryIndex};
use avalanche_model::{MutationIntent, Repository, Transaction};
use avalanche_nix::NixService;
use avalanche_ownership::OwnershipService;
use avalanche_source::SourceService;
use avalanche_validate::ValidationService;
use avalanche_write::{WriteError, WriteService};

pub struct ConfigurationEngine {
    pub index_service: IndexService,
    pub write_service: WriteService,
    pub git_service: GitService,
    pub doctor_service: DoctorService,
    pub validation_service: ValidationService,
    pub ownership_service: OwnershipService,
    pub graph_service: GraphService,
    pub nix_service: NixService,
    pub source_service: SourceService,
}

impl ConfigurationEngine {
    pub fn new(repo_path: impl Into<String>) -> Self {
        let repo_path = repo_path.into();
        Self {
            index_service: IndexService::new(),
            write_service: WriteService::new(),
            git_service: GitService::new(repo_path.clone()),
            doctor_service: DoctorService::new(),
            validation_service: ValidationService::new(),
            ownership_service: OwnershipService::new(),
            graph_service: GraphService::new(),
            nix_service: NixService::new(repo_path.clone()),
            source_service: SourceService::new(),
        }
    }

    pub fn index(&self, repository: &Repository) -> IndexResult<RepositoryIndex> {
        self.index_service.build(repository, &self.nix_service)
    }

    pub fn plan(&self, intents: Vec<MutationIntent>) -> Result<Transaction, WriteError> {
        self.write_service.plan(intents)
    }

    pub fn apply(&mut self, transaction: &mut Transaction) -> Result<(), WriteError> {
        self.write_service.apply(transaction)
    }
}
