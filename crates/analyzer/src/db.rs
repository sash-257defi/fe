use crate::context::{Analysis, Constant, FunctionBody};
use crate::errors::{ConstEvalError, TypeError};
use crate::namespace::items::{
    self, ContractFieldId, ContractId, DepGraphWrapper, EventId, FunctionId, GlobalId, IngotId,
    Item, ModuleConstantId, ModuleId, StructFieldId, StructId, TypeAliasId,
};
use crate::namespace::types;
use fe_common::Span;
use indexmap::map::IndexMap;
use smol_str::SmolStr;
use std::rc::Rc;
mod queries;

// from rust-analyzer {
#[macro_export]
macro_rules! impl_intern_key {
    ($name:ident) => {
        impl salsa::InternKey for $name {
            fn from_intern_id(v: salsa::InternId) -> Self {
                $name(v.as_u32())
            }
            fn as_intern_id(&self) -> salsa::InternId {
                salsa::InternId::from(self.0)
            }
        }
    };
}
// } from rust-analyzer

#[salsa::query_group(AnalyzerDbStorage)]
pub trait AnalyzerDb {
    #[salsa::interned]
    fn intern_global(&self, data: Rc<items::Global>) -> GlobalId;
    #[salsa::interned]
    fn intern_ingot(&self, data: Rc<items::Ingot>) -> IngotId;
    #[salsa::interned]
    fn intern_module(&self, data: Rc<items::Module>) -> ModuleId;
    #[salsa::interned]
    fn intern_module_const(&self, data: Rc<items::ModuleConstant>) -> ModuleConstantId;
    #[salsa::interned]
    fn intern_struct(&self, data: Rc<items::Struct>) -> StructId;
    #[salsa::interned]
    fn intern_struct_field(&self, data: Rc<items::StructField>) -> StructFieldId;
    #[salsa::interned]
    fn intern_type_alias(&self, data: Rc<items::TypeAlias>) -> TypeAliasId;
    #[salsa::interned]
    fn intern_contract(&self, data: Rc<items::Contract>) -> ContractId;
    #[salsa::interned]
    fn intern_contract_field(&self, data: Rc<items::ContractField>) -> ContractFieldId;
    #[salsa::interned]
    fn intern_function(&self, data: Rc<items::Function>) -> FunctionId;
    #[salsa::interned]
    fn intern_event(&self, data: Rc<items::Event>) -> EventId;

    // Ingot
    #[salsa::invoke(queries::ingots::ingot_all_modules)]
    fn ingot_all_modules(&self, ingot: IngotId) -> Rc<Vec<ModuleId>>;
    #[salsa::invoke(queries::ingots::ingot_main_module)]
    fn ingot_main_module(&self, ingot: IngotId) -> Analysis<Option<ModuleId>>;
    #[salsa::invoke(queries::ingots::ingot_lib_module)]
    fn ingot_lib_module(&self, ingot: IngotId) -> Analysis<Option<ModuleId>>;
    #[salsa::invoke(queries::ingots::ingot_root_module)]
    fn ingot_root_module(&self, ingot: IngotId) -> Option<ModuleId>;
    #[salsa::invoke(queries::ingots::ingot_root_sub_modules)]
    fn ingot_root_sub_modules(&self, ingot: IngotId) -> Rc<IndexMap<SmolStr, ModuleId>>;

    // Module
    #[salsa::invoke(queries::module::module_all_items)]
    fn module_all_items(&self, module: ModuleId) -> Rc<Vec<Item>>;
    #[salsa::invoke(queries::module::module_item_map)]
    fn module_item_map(&self, module: ModuleId) -> Analysis<Rc<IndexMap<SmolStr, Item>>>;
    #[salsa::invoke(queries::module::module_contracts)]
    fn module_contracts(&self, module: ModuleId) -> Rc<Vec<ContractId>>;
    #[salsa::invoke(queries::module::module_structs)]
    fn module_structs(&self, module: ModuleId) -> Rc<Vec<StructId>>;
    #[salsa::invoke(queries::module::module_constants)]
    fn module_constants(&self, module: ModuleId) -> Rc<Vec<ModuleConstantId>>;
    #[salsa::invoke(queries::module::module_used_item_map)]
    fn module_used_item_map(
        &self,
        module: ModuleId,
    ) -> Analysis<Rc<IndexMap<SmolStr, (Span, Item)>>>;
    #[salsa::invoke(queries::module::module_parent_module)]
    fn module_parent_module(&self, module: ModuleId) -> Option<ModuleId>;
    #[salsa::invoke(queries::module::module_sub_modules)]
    fn module_sub_modules(&self, module: ModuleId) -> Rc<IndexMap<SmolStr, ModuleId>>;

    // Module Constant
    #[salsa::cycle(queries::module::module_constant_type_cycle)]
    #[salsa::invoke(queries::module::module_constant_type)]
    fn module_constant_type(
        &self,
        id: ModuleConstantId,
    ) -> Analysis<Result<types::Type, TypeError>>;
    #[salsa::cycle(queries::module::module_constant_value_cycle)]
    #[salsa::invoke(queries::module::module_constant_value)]
    fn module_constant_value(
        &self,
        id: ModuleConstantId,
    ) -> Analysis<Result<Constant, ConstEvalError>>;

    // Contract
    #[salsa::invoke(queries::contracts::contract_all_functions)]
    fn contract_all_functions(&self, id: ContractId) -> Rc<Vec<FunctionId>>;
    #[salsa::invoke(queries::contracts::contract_function_map)]
    fn contract_function_map(&self, id: ContractId) -> Analysis<Rc<IndexMap<SmolStr, FunctionId>>>;
    #[salsa::invoke(queries::contracts::contract_public_function_map)]
    fn contract_public_function_map(&self, id: ContractId) -> Rc<IndexMap<SmolStr, FunctionId>>;
    #[salsa::invoke(queries::contracts::contract_init_function)]
    fn contract_init_function(&self, id: ContractId) -> Analysis<Option<FunctionId>>;
    #[salsa::invoke(queries::contracts::contract_call_function)]
    fn contract_call_function(&self, id: ContractId) -> Analysis<Option<FunctionId>>;

    #[salsa::invoke(queries::contracts::contract_all_events)]
    fn contract_all_events(&self, id: ContractId) -> Rc<Vec<EventId>>;
    #[salsa::invoke(queries::contracts::contract_event_map)]
    fn contract_event_map(&self, id: ContractId) -> Analysis<Rc<IndexMap<SmolStr, EventId>>>;

    #[salsa::invoke(queries::contracts::contract_all_fields)]
    fn contract_all_fields(&self, id: ContractId) -> Rc<Vec<ContractFieldId>>;
    #[salsa::invoke(queries::contracts::contract_field_map)]
    fn contract_field_map(
        &self,
        id: ContractId,
    ) -> Analysis<Rc<IndexMap<SmolStr, ContractFieldId>>>;
    #[salsa::invoke(queries::contracts::contract_field_type)]
    fn contract_field_type(
        &self,
        field: ContractFieldId,
    ) -> Analysis<Result<types::Type, TypeError>>;
    #[salsa::cycle(queries::contracts::contract_dependency_graph_cycle)]
    #[salsa::invoke(queries::contracts::contract_dependency_graph)]
    fn contract_dependency_graph(&self, id: ContractId) -> DepGraphWrapper;
    #[salsa::cycle(queries::contracts::contract_runtime_dependency_graph_cycle)]
    #[salsa::invoke(queries::contracts::contract_runtime_dependency_graph)]
    fn contract_runtime_dependency_graph(&self, id: ContractId) -> DepGraphWrapper;

    // Function
    #[salsa::invoke(queries::functions::function_signature)]
    fn function_signature(&self, id: FunctionId) -> Analysis<Rc<types::FunctionSignature>>;
    #[salsa::invoke(queries::functions::function_body)]
    fn function_body(&self, id: FunctionId) -> Analysis<Rc<FunctionBody>>;
    #[salsa::cycle(queries::functions::function_dependency_graph_cycle)]
    #[salsa::invoke(queries::functions::function_dependency_graph)]
    fn function_dependency_graph(&self, id: FunctionId) -> DepGraphWrapper;

    // Struct
    #[salsa::invoke(queries::structs::struct_type)]
    fn struct_type(&self, id: StructId) -> Rc<types::Struct>;
    #[salsa::invoke(queries::structs::struct_all_fields)]
    fn struct_all_fields(&self, id: StructId) -> Rc<Vec<StructFieldId>>;
    #[salsa::invoke(queries::structs::struct_field_map)]
    fn struct_field_map(&self, id: StructId) -> Analysis<Rc<IndexMap<SmolStr, StructFieldId>>>;
    #[salsa::invoke(queries::structs::struct_field_type)]
    fn struct_field_type(
        &self,
        field: StructFieldId,
    ) -> Analysis<Result<types::FixedSize, TypeError>>;
    #[salsa::invoke(queries::structs::struct_all_functions)]
    fn struct_all_functions(&self, id: StructId) -> Rc<Vec<FunctionId>>;
    #[salsa::invoke(queries::structs::struct_function_map)]
    fn struct_function_map(&self, id: StructId) -> Analysis<Rc<IndexMap<SmolStr, FunctionId>>>;
    #[salsa::invoke(queries::structs::struct_dependency_graph)]
    fn struct_dependency_graph(&self, id: StructId) -> DepGraphWrapper;

    // Event
    #[salsa::invoke(queries::events::event_type)]
    fn event_type(&self, event: EventId) -> Analysis<Rc<types::Event>>;

    // Type alias
    #[salsa::invoke(queries::types::type_alias_type)]
    #[salsa::cycle(queries::types::type_alias_type_cycle)]
    fn type_alias_type(&self, id: TypeAliasId) -> Analysis<Result<types::Type, TypeError>>;
}

#[salsa::database(AnalyzerDbStorage)]
#[derive(Default)]
pub struct TestDb {
    storage: salsa::Storage<TestDb>,
}
impl salsa::Database for TestDb {}
